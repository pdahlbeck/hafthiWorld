//! Explicit, material-safe annual coastal work. Never overwrite unfinished cargo.
use crate::{
    fishing::FishLocation,
    livestock::WoolLocation,
    shipping::{Voyage, VoyagePhase},
    simulation::Settlement,
    trade::SaltLocation,
};
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShoreCycles {
    pub last_started_year: u64,
    /// Retired batches; the current batch is still represented by actual slots.
    pub retired_rounds: u64,
}
impl Default for ShoreCycles {
    fn default() -> Self {
        Self {
            last_started_year: 1,
            retired_rounds: 0,
        }
    }
}
impl ShoreCycles {
    pub fn valid(&self, year: u64) -> bool {
        self.last_started_year > 0
            && self.last_started_year <= year
            && self.retired_rounds < self.last_started_year
    }
}
impl Settlement {
    pub fn next_shore_round_blocker(&self) -> Option<&'static str> {
        if self.voyage.phase != VoyagePhase::Completed {
            return Some("Finish the current wool shipment before starting another round.");
        }
        if self.calendar.year <= self.shore_cycles.last_started_year {
            return Some("The next coastal round opens in a later year. No goods have been reset.");
        }
        if !self.calendar.can_sow() {
            return Some("The next coastal round waits for spring or summer.");
        }
        if self.flock.regrowth_remaining.iter().any(|&v| v > 0.0) {
            return Some(
                "The sheep need a full gameplay year after shearing to regrow their fleece.",
            );
        }
        if self.coastal_owner.is_some() {
            return Some("The coastal worker must finish the current delivery and return first.");
        }
        if !self.flock.wool.iter().all(|&w| w == WoolLocation::Exported)
            || !self
                .barter
                .salt
                .iter()
                .all(|&s| s == SaltLocation::Consumed)
            || !self
                .fishery
                .fish
                .iter()
                .all(|&f| f == FishLocation::Eaten(self.household.id))
        {
            return Some(
                "Finish preserving, supplying and eating the current fish; stored goods are never overwritten.",
            );
        }
        None
    }
    /// U first requests the existing shipment, then explicitly authorizes a later
    /// complete production round. The caller gets feedback rather than lost stock.
    pub fn request_shipment(&mut self) -> Result<bool, &'static str> {
        if self.voyage.phase == VoyagePhase::NotOrdered {
            self.voyage.request();
            return Ok(false);
        }
        if let Some(reason) = self.next_shore_round_blocker() {
            return Err(reason);
        }
        self.shore_cycles.retired_rounds += 1;
        self.shore_cycles.last_started_year = self.calendar.year;
        self.flock.wool.fill(WoolLocation::Fleece);
        for sheep in &mut self.flock.sheep {
            sheep.shorn = false;
            sheep.ordered = true;
        }
        self.barter.salt.fill(SaltLocation::Offered);
        self.barter.ordered = true;
        self.fishery.fish.fill(FishLocation::Uncaught);
        self.fishery.fishing_ordered = true;
        self.fishery.salting_ordered = true;
        self.voyage = Voyage::default();
        self.voyage.request();
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{calendar::YEAR_SECONDS, simulation::Worker};
    fn order_first(s: &mut Settlement) {
        s.flock.request_shear();
        s.barter.request();
        s.fishery.request_fishing();
        s.fishery.request_salting();
        s.household.request_supply();
        assert_eq!(s.request_shipment(), Ok(false));
    }
    fn finish(s: &mut Settlement, w: &mut Worker) {
        for _ in 0..1800 {
            w.advance(1.0, s);
            if s.voyage.phase == VoyagePhase::Completed
                && s.household.fish_used % 4 == 0
                && s.fishery.fish.iter().all(|&f| f == FishLocation::Eaten(1))
                && s.coastal_owner.is_none()
            {
                return;
            }
        }
        panic!(
            "unfinished {:?}, fish {:?}, worker {:?}",
            s.voyage.phase, s.fishery.fish, w.task
        );
    }
    fn wait_for_renewal(s: &mut Settlement, w: &mut Worker) {
        for _ in 0..1800 {
            if s.next_shore_round_blocker().is_none() {
                return;
            }
            w.advance(1.0, s);
        }
        panic!("renewal blocked: {:?}", s.next_shore_round_blocker());
    }
    #[test]
    fn renewed_work_uses_the_same_two_workers_in_both_views() {
        use crate::{camera::CameraRig, simulation::SimulationPlugin};
        use bevy::{prelude::*, time::TimeUpdateStrategy};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(250),
            ))
            .insert_resource(Time::<Fixed>::from_seconds(0.25))
            .init_resource::<CameraRig>()
            .add_plugins(SimulationPlugin);
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            order_first(&mut s);
            s.request_helper();
        }
        app.update();
        app.update();
        let ids: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .iter(app.world())
            .collect();
        assert_eq!(ids.len(), 2);
        let mut renewed = false;
        for frame in 0..14000 {
            if frame % 31 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            {
                let mut s = app.world_mut().resource_mut::<Settlement>();
                if !renewed && s.next_shore_round_blocker().is_none() {
                    assert_eq!(s.request_shipment(), Ok(true));
                    renewed = true;
                }
                if renewed && s.voyage.phase == VoyagePhase::Completed && s.household.fish_used == 8
                {
                    assert_eq!(s.shore_cycles.retired_rounds, 1);
                    assert!(
                        ids.iter()
                            .all(|&id| app.world().get::<Worker>(id).is_some())
                    );
                    return;
                }
            }
            app.update();
        }
        panic!(
            "two-worker round failed: {:?}",
            app.world().resource::<Settlement>()
        );
    }
    #[test]
    fn three_real_rounds_require_labor_and_keep_lifetime_material_totals() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        order_first(&mut s);
        for round in 1..=3 {
            finish(&mut s, &mut w);
            assert_eq!(s.household.fish_used, round * 4);
            assert_eq!(s.fishery.batches, round * 2);
            assert_eq!(s.shore_cycles.retired_rounds, round - 1);
            assert_eq!(s.flock.wool, [WoolLocation::Exported; 4]);
            assert_eq!(s.barter.salt, [SaltLocation::Consumed; 2]);
            assert!(s.voyage.valid(&s.flock.wool));
            if round < 3 {
                wait_for_renewal(&mut s, &mut w);
                let before = s.household.fish_used;
                assert_eq!(s.request_shipment(), Ok(true));
                assert_eq!(s.household.fish_used, before);
                assert_eq!(s.fishery.fish, [FishLocation::Uncaught; 4]);
                assert_eq!(s.barter.salt, [SaltLocation::Offered; 2]);
                assert_eq!(s.flock.wool, [WoolLocation::Fleece; 4]);
                assert!(s.request_shipment().is_err());
            }
        }
    }
    #[test]
    fn pending_stock_and_fleece_cooldown_cannot_be_overwritten() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        order_first(&mut s);
        finish(&mut s, &mut w);
        wait_for_renewal(&mut s, &mut w);
        for location in [
            FishLocation::RawStored,
            FishLocation::Home(1),
            FishLocation::CarriedRaw,
        ] {
            s.fishery.fish[0] = location;
            let before = serde_json::to_string(&s).unwrap();
            assert!(s.request_shipment().is_err());
            assert_eq!(serde_json::to_string(&s).unwrap(), before);
        }
        s.fishery.fish[0] = FishLocation::Eaten(1);
        s.flock.regrowth_remaining[0] = 1.0;
        assert!(s.request_shipment().is_err());
        s.flock.graze(1.0);
        assert_eq!(s.request_shipment(), Ok(true));
        assert_eq!(s.flock.regrowth_remaining, [0.0; 2]);
    }
    #[test]
    fn regrowth_uses_world_time_and_legacy_state_defaults_safely() {
        let mut a = crate::livestock::Flock::default();
        a.request_shear();
        a.clip(0);
        let mut b = a.clone();
        a.graze(YEAR_SECONDS);
        for _ in 0..1920 {
            b.graze(YEAR_SECONDS / 1920.0);
        }
        assert_eq!(a.regrowth_remaining, b.regrowth_remaining);
        assert_eq!(a.wool, b.wool); // Growing fleece never deletes the cut wool.
        let s = Settlement::default();
        let mut json = serde_json::to_value(s).unwrap();
        json.as_object_mut().unwrap().remove("shore_cycles");
        json["flock"]
            .as_object_mut()
            .unwrap()
            .remove("regrowth_remaining");
        let old: Settlement = serde_json::from_value(json).unwrap();
        assert!(old.shore_cycles.valid(1));
        assert_eq!(old.flock.regrowth_remaining, [0.0; 2]);
    }
}
