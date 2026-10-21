//! First household: an owned pantry, physical supply and abstract food use.
use crate::{farming::GrainLocation, simulation::Settlement};
use bevy::prelude::*;

pub const HOME_ID: u32 = 1;
pub const HOME_DELIVERY: Vec2 = Vec2::new(2.0, 3.35);
pub const PANTRY: Vec2 = Vec2::new(1.4, 3.05);
pub const RATION_SECONDS: f64 = 40.0;
#[derive(Debug)]
pub struct Household {
    pub id: u32,
    pub supply_ordered: bool,
    pub started: bool,
    pub food_used: u64,
    pub missed_rations: u64,
    pub hungry: bool,
    elapsed: f64,
}
impl Default for Household {
    fn default() -> Self {
        Self {
            id: HOME_ID,
            supply_ordered: false,
            started: false,
            food_used: 0,
            missed_rations: 0,
            hungry: false,
            elapsed: 0.0,
        }
    }
}
impl Household {
    pub fn request_supply(&mut self) -> bool {
        if self.supply_ordered {
            return false;
        }
        self.supply_ordered = true;
        true
    }
    pub fn advance(&mut self, seconds: f64, grain: &mut [GrainLocation]) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        if !self.started {
            if !grain.contains(&GrainLocation::Home(self.id)) {
                return;
            }
            self.started = true;
        }
        self.elapsed += seconds;
        let due = (self.elapsed / RATION_SECONDS).floor() as u64;
        self.elapsed %= RATION_SECONDS;
        if due == 0 {
            return;
        }
        let mut used = 0;
        for unit in grain {
            if used < due && *unit == GrainLocation::Home(self.id) {
                *unit = GrainLocation::Consumed(self.id);
                used += 1;
            }
        }
        self.food_used = self.food_used.saturating_add(used);
        self.missed_rations = self.missed_rations.saturating_add(due - used);
        self.hungry = used < due;
    }
    pub fn next_use(&self) -> f64 {
        RATION_SECONDS - self.elapsed
    }
    pub fn status(&self, pantry: usize) -> &'static str {
        if !self.started {
            "awaiting first delivery"
        } else if pantry > 0 {
            "food available"
        } else if self.hungry {
            "needs food"
        } else {
            "supplied"
        }
    }
}
pub fn pantry_position(i: usize) -> Vec2 {
    PANTRY + Vec2::new((i as f32 - 1.5) * 0.32, 0.0)
}

pub struct HouseholdPlugin;
impl Plugin for HouseholdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}
/// A persistent home marker, separate from the worker's household membership.
#[derive(Component)]
pub struct HouseholdHome(pub u32);
fn setup(
    mut commands: Commands,
    settlement: Res<Settlement>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wood = materials.add(Color::srgb(0.36, 0.25, 0.14));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let home = HouseholdHome(settlement.household.id);
    let name = Name::new(format!("Home {} pantry", home.0));
    commands
        .spawn((
            Transform::from_xyz(PANTRY.x, 0.2, PANTRY.y),
            Visibility::Inherited,
            home,
            name,
        ))
        .with_children(|home| {
            home.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(0.0, 0.04, 0.0).with_scale(Vec3::new(1.6, 0.08, 0.65)),
            ));
            for x in [-0.8, 0.8] {
                home.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(wood.clone()),
                    Transform::from_xyz(x, 0.15, 0.0).with_scale(Vec3::new(0.06, 0.3, 0.65)),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        farming::{CropPhase, FarmVisual, FarmingPlugin, GRAIN_UNITS},
        livestock::{WOOL_UNITS, WoolLocation},
        simulation::{BUILD_COST, LogLocation, SimulationPlugin, TOTAL_LOGS, Task, Worker},
    };
    fn balance(s: &Settlement, w: &Worker) {
        assert_eq!(
            s.farm.count(GrainLocation::Carried),
            usize::from(w.carrying_grain.is_some())
        );
        assert!(
            usize::from(w.carrying.is_some())
                + usize::from(w.carrying_grain.is_some())
                + usize::from(w.carrying_wool.is_some())
                <= 1
        );
        let states = [
            GrainLocation::Unproduced,
            GrainLocation::Ground,
            GrainLocation::Carried,
            GrainLocation::Stored,
            GrainLocation::Home(HOME_ID),
            GrainLocation::Consumed(HOME_ID),
        ];
        assert_eq!(
            states.iter().map(|&g| s.farm.count(g)).sum::<usize>(),
            GRAIN_UNITS
        );
        assert_eq!(
            s.household.food_used as usize,
            s.farm.count(GrainLocation::Consumed(HOME_ID))
        );
        assert!(
            crate::world::walkable(w.position),
            "blocked food route {:?} {:?}",
            w.task,
            w.position
        );
    }
    #[test]
    fn ration_clock_uses_world_time_even_without_workers_or_with_extra_idle_workers() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .add_plugins(SimulationPlugin);
        app.update();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        app.world_mut().despawn(worker);
        app.world_mut()
            .resource_mut::<Settlement>()
            .farm
            .grain
            .fill(GrainLocation::Home(HOME_ID));
        for _ in 0..420 {
            app.update();
        }
        assert_eq!(app.world().resource::<Settlement>().household.food_used, 1);
        app.world_mut().spawn(Worker::for_home(HOME_ID));
        app.world_mut().spawn(Worker::for_home(2));
        for _ in 0..400 {
            app.update();
        }
        assert_eq!(app.world().resource::<Settlement>().household.food_used, 2);
    }
    #[test]
    fn food_clock_starts_on_delivery_and_never_consumes_stored_carried_or_other_owned_grain() {
        let mut home = Household::default();
        let mut grain = [
            GrainLocation::Stored,
            GrainLocation::Carried,
            GrainLocation::Home(2),
            GrainLocation::Unproduced,
        ];
        home.advance(1000.0, &mut grain);
        assert!(!home.started);
        assert_eq!(home.food_used, 0);
        grain[0] = GrainLocation::Home(HOME_ID);
        grain[1] = GrainLocation::Home(HOME_ID);
        home.advance(RATION_SECONDS, &mut grain);
        assert_eq!(home.food_used, 1);
        assert_eq!(home.missed_rations, 0);
        home.advance(RATION_SECONDS * 2.0, &mut grain);
        assert_eq!(home.food_used, 2);
        assert_eq!(home.missed_rations, 1);
        assert!(home.hungry);
        assert_eq!(home.status(0), "needs food");
        assert_eq!(home.status(1), "food available");
        assert_eq!(grain[2], GrainLocation::Home(2));
        assert_eq!(grain[3], GrainLocation::Unproduced);
        let before = grain;
        let remaining = home.next_use();
        for invalid in [f64::NAN, f64::INFINITY, -3.0] {
            home.advance(invalid, &mut grain);
        }
        assert_eq!(grain, before);
        assert_eq!(home.next_use(), remaining);
    }
    #[test]
    fn food_use_is_stable_across_frame_partitions_and_counts_empty_rations_without_underflow() {
        let mut a = Household::default();
        let mut b = Household::default();
        let mut ga = [GrainLocation::Home(HOME_ID); GRAIN_UNITS];
        let mut gb = ga;
        a.advance(400.0, &mut ga);
        for _ in 0..4000 {
            b.advance(0.1, &mut gb);
        }
        assert_eq!(ga, gb);
        assert_eq!(a.food_used, 4);
        assert_eq!(a.food_used, b.food_used);
        assert_eq!(a.missed_rations, 6);
        assert_eq!(a.missed_rations, b.missed_rations);
        assert!((a.next_use() - b.next_use()).abs() < 0.0001);
    }
    #[test]
    fn queued_supply_waits_for_real_harvest_then_transports_and_consumes_exactly_four_units() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        assert!(s.household.request_supply());
        assert!(!s.household.request_supply());
        w.advance(100.0, &mut s);
        assert_eq!(s.household.food_used, 0);
        assert!(!s.household.started);
        assert_eq!(s.farm.count(GrainLocation::Home(HOME_ID)), 0);
        s.farm.request_sow();
        w.advance(80.0, &mut s);
        s.farm.request_reap();
        let mut carried_home = false;
        let mut in_pantry = false;
        let mut shared_stock = false;
        for _ in 0..3500 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            carried_home |= w.task == Task::ToHome && w.carrying_grain.is_some();
            in_pantry |= s.farm.count(GrainLocation::Home(HOME_ID)) > 0;
            shared_stock |= s.farm.count(GrainLocation::Stored) > 0;
        }
        assert!(carried_home && in_pantry && shared_stock);
        assert_eq!(s.household.food_used, GRAIN_UNITS as u64);
        assert_eq!(s.farm.count(GrainLocation::Stored), 0);
        assert_eq!(s.farm.count(GrainLocation::Home(HOME_ID)), 0);
        assert!(s.household.hungry);
        assert!(!s.household.request_supply());
        assert_eq!(s.count(LogLocation::InTree), TOTAL_LOGS);
        assert_eq!(s.flock.count(WoolLocation::Fleece), WOOL_UNITS);
    }
    #[test]
    fn available_food_has_priority_but_a_nonmember_cannot_supply_another_home() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.household.request_supply();
        s.farm.grain[0] = GrainLocation::Stored;
        s.request_harvest(0);
        w.advance(0.1, &mut s);
        assert_eq!(w.task, Task::LoadingFood);
        assert_eq!(s.farm.grain[0], GrainLocation::Stored);
        w.advance(1.0, &mut s);
        assert_eq!(w.task, Task::ToHome);
        assert_eq!(s.farm.grain[0], GrainLocation::Carried);
        assert_eq!(s.household.food_used, 0);
        let mut other = Settlement::default();
        other.household.request_supply();
        other.farm.grain.fill(GrainLocation::Stored);
        let mut visitor = Worker::for_home(2);
        visitor.advance(100.0, &mut other);
        assert_eq!(other.farm.count(GrainLocation::Stored), GRAIN_UNITS);
        assert!(!other.household.started);
    }
    #[test]
    fn food_timber_crops_and_sheep_share_clear_routes_and_keep_separate_accounts() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.household.request_supply();
        s.farm.request_sow();
        s.flock.request_shear();
        s.request_build();
        for i in 0..3 {
            s.request_harvest(i);
        }
        for _ in 0..8000 {
            if s.farm.phase == CropPhase::Ripe {
                s.farm.request_reap();
            }
            w.advance(0.1, &mut s);
            balance(&s, &w);
        }
        assert!(s.construction.complete);
        assert_eq!(s.installed(), BUILD_COST);
        assert_eq!(s.stored(), TOTAL_LOGS - BUILD_COST);
        assert_eq!(s.flock.count(WoolLocation::Stored), WOOL_UNITS);
        assert_eq!(s.household.food_used, GRAIN_UNITS as u64);
    }
    #[test]
    fn home_membership_and_grain_entities_persist_when_cameras_change_and_food_is_used() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((SimulationPlugin, FarmingPlugin, HouseholdPlugin));
        app.update();
        let home = app
            .world_mut()
            .query_filtered::<Entity, With<HouseholdHome>>()
            .single(app.world())
            .unwrap();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        let grain: Vec<_> = app
            .world_mut()
            .query::<(Entity, &FarmVisual)>()
            .iter(app.world())
            .filter_map(|(e, v)| matches!(v, FarmVisual::Grain(_)).then_some(e))
            .collect();
        assert_eq!(grain.len(), GRAIN_UNITS);
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.household.request_supply();
            s.farm.request_sow();
        }
        for frame in 0..3500 {
            if frame % 7 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            if app.world().resource::<Settlement>().farm.phase == CropPhase::Ripe {
                app.world_mut()
                    .resource_mut::<Settlement>()
                    .farm
                    .request_reap();
            }
            app.update();
        }
        assert_eq!(app.world().get::<Worker>(worker).unwrap().home_id, HOME_ID);
        assert_eq!(app.world().get::<HouseholdHome>(home).unwrap().0, HOME_ID);
        assert_eq!(
            app.world().resource::<Settlement>().household.food_used,
            GRAIN_UNITS as u64
        );
        for id in grain {
            assert!(app.world().get::<FarmVisual>(id).is_some());
            assert_eq!(
                *app.world().get::<Visibility>(id).unwrap(),
                Visibility::Hidden
            );
        }
    }
}
