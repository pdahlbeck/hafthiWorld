//! Owned firewood and winter-only demand; timings/units are gameplay abstractions.
use crate::{
    calendar::Season,
    simulation::{LogLocation, Settlement},
};
use bevy::prelude::*;
pub const FUEL_SECONDS: f64 = 40.0;
pub const FUEL_TARGET: usize = 3;
pub const FUEL_STORE: Vec2 = Vec2::new(3.45, 3.1);
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct Hearth {
    pub ordered: bool,
    pub burned: u64,
    pub missed: u64,
    elapsed: f64,
}
impl Hearth {
    pub fn request(&mut self) -> bool {
        if self.ordered {
            return false;
        }
        self.ordered = true;
        true
    }
    pub fn advance(&mut self, winter_seconds: f64, home: u32, logs: &mut [LogLocation]) {
        if !winter_seconds.is_finite() || winter_seconds <= 0.0 {
            return;
        }
        self.elapsed += winter_seconds;
        let due = (self.elapsed / FUEL_SECONDS).floor() as u64;
        self.elapsed %= FUEL_SECONDS;
        let mut burned = 0;
        for log in logs {
            if burned < due && *log == LogLocation::Home(home) {
                *log = LogLocation::Burned(home);
                burned += 1;
            }
        }
        self.burned = self.burned.saturating_add(burned);
        self.missed = self.missed.saturating_add(due - burned);
    }
    pub fn next_use(&self) -> f64 {
        FUEL_SECONDS - self.elapsed
    }
    pub fn status(&self, season: Season, available: usize) -> &'static str {
        if season != Season::Winter {
            "winter demand paused"
        } else if available == 0 {
            "needs firewood"
        } else {
            "fuel ready"
        }
    }
}
pub fn log_position(slot: usize) -> Vec3 {
    Vec3::new(
        FUEL_STORE.x + (slot % 3) as f32 * 0.4 - 0.4,
        0.43 + (slot / 3) as f32 * 0.23,
        FUEL_STORE.y,
    )
}
pub struct HearthPlugin;
impl Plugin for HearthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}
#[derive(Component)]
struct FuelRack;
fn setup(
    mut commands: Commands,
    settlement: Res<Settlement>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wood = materials.add(Color::srgb(0.34, 0.24, 0.14));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands
        .spawn((
            FuelRack,
            Name::new(format!("Home {} firewood", settlement.household.id)),
            Transform::from_xyz(FUEL_STORE.x, 0.2, FUEL_STORE.y),
            Visibility::Inherited,
        ))
        .with_children(|rack| {
            rack.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(0.0, 0.06, 0.0).with_scale(Vec3::new(1.65, 0.08, 0.95)),
            ));
            for x in [-0.8, 0.8] {
                rack.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(wood.clone()),
                    Transform::from_xyz(x, 0.3, 0.0).with_scale(Vec3::new(0.06, 0.6, 0.8)),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        farming::{CropPhase, GRAIN_UNITS, GrainLocation},
        livestock::{WOOL_UNITS, WoolLocation},
        simulation::{BUILD_COST, TOTAL_LOGS, Task, Worker},
        trade::{SALT_UNITS, SaltLocation},
    };
    fn balance(s: &Settlement, w: &Worker) {
        let home = s.household.id;
        assert_eq!(
            [
                LogLocation::InTree,
                LogLocation::Ground,
                LogLocation::Carried,
                LogLocation::Stored,
                LogLocation::Installed,
                LogLocation::Home(home),
                LogLocation::Burned(home)
            ]
            .iter()
            .map(|&l| s.count(l))
            .sum::<usize>(),
            TOTAL_LOGS
        );
        assert_eq!(
            s.count(LogLocation::Carried),
            usize::from(w.carrying.is_some())
        );
        assert_eq!(
            s.count(LogLocation::Burned(home)) as u64,
            s.household.hearth.burned
        );
        assert!(s.count(LogLocation::Home(home)) <= FUEL_TARGET);
        assert!(
            usize::from(w.carrying.is_some())
                + usize::from(w.carrying_grain.is_some())
                + usize::from(w.carrying_wool.is_some())
                + usize::from(w.carrying_salt.is_some())
                <= 1
        );
        assert!(
            crate::world::walkable(w.position),
            "blocked {:?} at {:?}",
            w.task,
            w.position
        );
    }
    #[test]
    fn fuel_clock_consumes_only_owned_delivered_logs_and_never_underflows() {
        let mut h = Hearth::default();
        let mut logs = [
            LogLocation::Stored,
            LogLocation::Carried,
            LogLocation::Installed,
            LogLocation::Home(2),
            LogLocation::Home(1),
            LogLocation::Home(1),
        ];
        let before = logs;
        for dt in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
            h.advance(dt, 1, &mut logs);
        }
        assert_eq!(logs, before);
        h.advance(39.0, 1, &mut logs);
        assert_eq!(logs, before);
        h.advance(1.0, 1, &mut logs);
        assert_eq!(h.burned, 1);
        h.advance(120.0, 1, &mut logs);
        assert_eq!(h.burned, 2);
        assert_eq!(h.missed, 2);
        assert_eq!(&logs[..4], &before[..4]);
        assert_eq!(h.status(Season::Winter, 0), "needs firewood");
        assert_eq!(h.status(Season::Summer, 0), "winter demand paused");
    }
    #[test]
    fn fuel_uses_only_actual_winter_time_across_years_and_frame_partitions() {
        let mut a = Settlement::default();
        let mut b = Settlement::default();
        a.logs.fill(LogLocation::Home(1));
        b.logs = a.logs;
        let mut wa = Worker::default();
        let mut wb = Worker::default();
        wa.advance(360.0, &mut a);
        assert_eq!(a.household.hearth.burned, 0);
        assert_eq!(a.household.hearth.missed, 0);
        wa.advance(40.0, &mut a);
        assert_eq!(a.household.hearth.burned, 1);
        wa.advance(1040.0, &mut a);
        for _ in 0..5760 {
            wb.advance(0.25, &mut b);
        }
        assert_eq!(a.logs, b.logs);
        assert_eq!(a.household.hearth.burned, 6);
        assert_eq!(a.household.hearth.missed, 3);
        assert_eq!(a.household.hearth.burned, b.household.hearth.burned);
        assert_eq!(a.household.hearth.missed, b.household.hearth.missed);
        assert_eq!(a.household.hearth.next_use(), b.household.hearth.next_use());
    }
    #[test]
    fn supply_waits_for_real_stored_logs_and_caps_the_owned_stack() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        assert!(s.household.hearth.request());
        assert!(!s.household.hearth.request());
        w.advance(100.0, &mut s);
        assert_eq!(s.count(LogLocation::InTree), TOTAL_LOGS);
        assert_eq!(w.task, Task::Idle);
        s.logs.fill(LogLocation::Stored);
        w.advance(0.1, &mut s);
        assert_eq!(w.task, Task::LoadingFuel);
        assert_eq!(s.stored(), TOTAL_LOGS);
        w.advance(1.0, &mut s);
        assert_eq!(w.task, Task::ToFuelHome);
        assert_eq!(s.count(LogLocation::Home(1)), 0);
        for _ in 0..900 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
        }
        assert_eq!(s.count(LogLocation::Home(1)), FUEL_TARGET);
        assert_eq!(s.stored(), TOTAL_LOGS - FUEL_TARGET);
        assert_eq!(s.household.hearth.burned, 0); // Still summer.
        assert_eq!(w.task, Task::Idle);
    }
    #[test]
    fn no_supply_order_and_nonmembers_cannot_transfer_shared_fuel() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.logs.fill(LogLocation::Stored);
        w.advance(100.0, &mut s);
        assert_eq!(s.stored(), TOTAL_LOGS);
        s.household.hearth.request();
        let mut outsider = Worker::for_home(2);
        outsider.advance(100.0, &mut s);
        assert_eq!(s.stored(), TOTAL_LOGS);
        assert_eq!(s.count(LogLocation::Home(1)), 0);
        assert_eq!(s.count(LogLocation::Home(2)), 0);
    }
    #[test]
    fn two_winters_share_six_logs_with_construction_and_all_other_material_chains() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.household.hearth.request();
        s.household.request_supply();
        s.request_build();
        s.farm.request_reserve();
        s.flock.request_shear();
        s.barter.request();
        for i in 0..3 {
            s.request_harvest(i);
        }
        let mut carried_home = false;
        for _ in 0..12000 {
            if s.farm.harvests < 2 {
                if matches!(s.farm.phase, CropPhase::Fallow | CropPhase::Harvested) {
                    s.farm.request_sow();
                }
                s.farm.request_reap();
            }
            w.advance(0.1, &mut s);
            balance(&s, &w);
            carried_home |= w.task == Task::ToFuelHome && w.carrying.is_some();
            let live = s
                .farm
                .grain
                .iter()
                .filter(|g| {
                    matches!(
                        g,
                        GrainLocation::Ground
                            | GrainLocation::Carried
                            | GrainLocation::Stored
                            | GrainLocation::Home(_)
                    )
                })
                .count();
            assert_eq!(
                s.farm.harvests * GRAIN_UNITS as u64,
                live as u64 + s.household.food_used + s.farm.seed_batches_saved
            );
            assert_eq!(
                s.flock.count(WoolLocation::Trader) / 2,
                SALT_UNITS - s.barter.count(SaltLocation::Offered)
            );
        }
        assert!(carried_home && s.construction.complete);
        assert_eq!(s.installed(), BUILD_COST);
        assert_eq!(s.household.hearth.burned as usize, TOTAL_LOGS - BUILD_COST);
        assert_eq!(s.household.hearth.missed, 2);
        assert_eq!(s.count(LogLocation::Home(1)), 0);
        assert_eq!(s.farm.harvests, 2);
        assert_eq!(s.household.food_used, 6);
        assert_eq!(s.barter.count(SaltLocation::Stored), SALT_UNITS);
        assert_eq!(s.flock.count(WoolLocation::Trader), WOOL_UNITS);
    }
}
