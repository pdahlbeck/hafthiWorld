//! Finite coastal catch, physical salting and preserved fish for the existing home.
use crate::{
    simulation::{STOCKPILE, Settlement, Worker},
    trade::SaltLocation,
};
use bevy::prelude::*;
pub const FISH_UNITS: usize = 4;
pub const BATCH_SIZE: usize = 2;
pub const FISHING_SPOT: Vec2 = Vec2::new(-5.7, 6.0);
pub const SALTING_TABLE: Vec2 = Vec2::new(-3.9, 4.75);
pub const SALTING_WORK: Vec2 = Vec2::new(-3.9, 4.1);
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FishLocation {
    Uncaught,
    FreshAtShore,
    CarriedRaw,
    RawStored,
    RawAtTable,
    PreservedAtTable,
    CarriedPreserved,
    PreservedStored,
    Home(u32),
    Eaten(u32),
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Fishery {
    pub fishing_ordered: bool,
    pub salting_ordered: bool,
    pub fish: [FishLocation; FISH_UNITS],
    pub batches: u64,
}
impl Default for Fishery {
    fn default() -> Self {
        Self {
            fishing_ordered: false,
            salting_ordered: false,
            fish: [FishLocation::Uncaught; FISH_UNITS],
            batches: 0,
        }
    }
}
impl Fishery {
    pub fn request_fishing(&mut self) -> bool {
        if self.fishing_ordered || !self.fish.contains(&FishLocation::Uncaught) {
            return false;
        }
        self.fishing_ordered = true;
        true
    }
    pub fn request_salting(&mut self) -> bool {
        if self.salting_ordered {
            return false;
        }
        self.salting_ordered = true;
        true
    }
    pub fn count(&self, location: FishLocation) -> usize {
        self.fish.iter().filter(|&&f| f == location).count()
    }
    pub fn first(&self, location: FishLocation) -> Option<usize> {
        self.fish.iter().position(|&f| f == location)
    }
    pub fn catch(&mut self, index: usize) -> bool {
        if !self.fishing_ordered || self.fish.get(index) != Some(&FishLocation::Uncaught) {
            return false;
        }
        self.fish[index] = FishLocation::FreshAtShore;
        true
    }
    pub fn salting_salt(&self, salt: &[SaltLocation]) -> Option<usize> {
        if !self.salting_ordered
            || self.count(FishLocation::RawStored) < BATCH_SIZE
            || self.count(FishLocation::RawAtTable) + self.count(FishLocation::PreservedAtTable) > 0
            || salt.contains(&SaltLocation::AtSalting)
        {
            return None;
        }
        salt.iter().position(|&s| s == SaltLocation::Stored)
    }
    pub fn preserve(&mut self, salt: &mut [SaltLocation]) -> bool {
        if !self.salting_ordered
            || self.count(FishLocation::RawAtTable) != BATCH_SIZE
            || salt
                .iter()
                .filter(|&&s| s == SaltLocation::AtSalting)
                .count()
                != 1
        {
            return false;
        }
        for fish in &mut self.fish {
            if *fish == FishLocation::RawAtTable {
                *fish = FishLocation::PreservedAtTable;
            }
        }
        *salt
            .iter_mut()
            .find(|s| **s == SaltLocation::AtSalting)
            .unwrap() = SaltLocation::Consumed;
        self.batches += 1;
        true
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CoastalTask {
    ToFishingSpot,
    Fishing,
    LoadingFresh,
    ReturningFresh,
    StoringFresh,
    ToBrineStore,
    LoadingBrineSalt,
    ToTableWithSalt,
    DepositSalt,
    BackForRaw,
    LoadingRaw,
    ToTableWithRaw,
    DepositRaw,
    Salting,
    LoadingPreserved,
    BackWithPreserved,
    StoringPreserved,
    ToPickPreserved,
    ToMealStore,
    LoadingMeal,
    ToFishHome,
    UnloadingMeal,
}
pub fn table_position(index: usize) -> Vec2 {
    SALTING_TABLE + Vec2::new(-0.35 + index as f32 * 0.24, 0.0)
}
fn fish_position(location: FishLocation, index: usize, carrier: Vec2) -> (Vec2, f32) {
    match location {
        FishLocation::FreshAtShore => (FISHING_SPOT + Vec2::new(-0.3, 0.35), 0.4),
        FishLocation::RawStored => (STOCKPILE + Vec2::new(-0.8 + index as f32 * 0.26, 1.7), 0.36),
        FishLocation::RawAtTable | FishLocation::PreservedAtTable => (table_position(index), 0.92),
        FishLocation::CarriedRaw | FishLocation::CarriedPreserved => {
            (carrier + Vec2::new(0.3, 0.0), 1.0)
        }
        FishLocation::PreservedStored => {
            (STOCKPILE + Vec2::new(-0.8 + index as f32 * 0.26, 2.1), 0.36)
        }
        FishLocation::Home(_) => (
            crate::household::PANTRY + Vec2::new(-0.5 + index as f32 * 0.3, 0.42),
            0.4,
        ),
        _ => (Vec2::ZERO, 0.0),
    }
}
pub struct FishingPlugin;
impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, sync);
    }
}
#[derive(Component)]
struct FishVisual(usize);
#[derive(Resource)]
struct FishColors {
    fresh: Handle<StandardMaterial>,
    preserved: Handle<StandardMaterial>,
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wood = materials.add(Color::srgb(0.37, 0.25, 0.15));
    let rope = materials.add(Color::srgb(0.64, 0.58, 0.39));
    let fresh = materials.add(Color::srgb(0.47, 0.63, 0.64));
    let preserved = materials.add(Color::srgb(0.65, 0.48, 0.29));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands
        .spawn((
            Name::new("Salting work table"),
            Transform::from_xyz(SALTING_TABLE.x, 0.0, SALTING_TABLE.y),
            Visibility::Inherited,
        ))
        .with_children(|table| {
            table.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(0.0, 0.78, 0.0).with_scale(Vec3::new(1.35, 0.12, 0.75)),
            ));
            for x in [-0.5, 0.5] {
                table.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(wood.clone()),
                    Transform::from_xyz(x, 0.49, 0.0).with_scale(Vec3::new(0.12, 0.6, 0.6)),
                ));
            }
        });
    // A simple original net frame signals the authored fishing spot, not a reconstructed fishing technology.
    for x in [-0.5, 0.5] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(wood.clone()),
            Transform::from_xyz(FISHING_SPOT.x + x, 0.8, FISHING_SPOT.y + 0.6)
                .with_scale(Vec3::new(0.06, 1.2, 0.06)),
        ));
    }
    for height in [0.5, 0.8, 1.1] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(rope.clone()),
            Transform::from_xyz(FISHING_SPOT.x, height, FISHING_SPOT.y + 0.6)
                .with_scale(Vec3::new(1.0, 0.02, 0.02)),
        ));
    }
    for x in [-0.3, 0.0, 0.3] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(rope.clone()),
            Transform::from_xyz(FISHING_SPOT.x + x, 0.8, FISHING_SPOT.y + 0.6)
                .with_scale(Vec3::new(0.02, 0.6, 0.02)),
        ));
    }
    let body = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("valid fish mesh"));
    let tail = meshes.add(Cone::new(0.65, 0.7));
    for i in 0..FISH_UNITS {
        commands
            .spawn((
                FishVisual(i),
                Name::new(format!("Fish {}", i)),
                Mesh3d(body.clone()),
                MeshMaterial3d(fresh.clone()),
                Transform::default(),
                Visibility::Hidden,
            ))
            .with_children(|fish| {
                fish.spawn((
                    Mesh3d(tail.clone()),
                    MeshMaterial3d(fresh.clone()),
                    Transform::from_xyz(0.0, 0.0, -0.95)
                        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ));
            });
    }
    commands.insert_resource(FishColors { fresh, preserved });
}
fn sync(
    s: Res<Settlement>,
    workers: Query<&Worker>,
    colors: Res<FishColors>,
    mut visuals: Query<(
        &FishVisual,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (visual, mut transform, mut visibility, mut material) in &mut visuals {
        let location = s.fishery.fish[visual.0];
        if matches!(location, FishLocation::Uncaught | FishLocation::Eaten(_)) {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        let (p, y) = fish_position(
            location,
            visual.0,
            workers
                .iter()
                .find(|w| w.carrying_fish == Some(visual.0))
                .map_or(STOCKPILE, |w| w.position),
        );
        transform.translation = Vec3::new(p.x, y, p.y);
        transform.scale = Vec3::new(0.12, 0.07, 0.30);
        let handle = if matches!(
            location,
            FishLocation::PreservedAtTable
                | FishLocation::CarriedPreserved
                | FishLocation::PreservedStored
                | FishLocation::Home(_)
        ) {
            &colors.preserved
        } else {
            &colors.fresh
        };
        if material.0 != *handle {
            material.0 = handle.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        farming::{CropPhase, GRAIN_UNITS, GrainLocation},
        household::{Household, RATION_SECONDS},
        livestock::{LivestockPlugin, WOOL_UNITS, WoolLocation},
        simulation::{BUILD_COST, LogLocation, SimulationPlugin, TOTAL_LOGS, Task},
        trade::{SALT_UNITS, TradePlugin},
    };
    fn order_chain(s: &mut Settlement) {
        s.fishery.request_fishing();
        s.fishery.request_salting();
        s.household.request_supply();
        s.flock.request_shear();
        s.barter.request();
    }
    fn balance(s: &Settlement, w: &Worker) {
        let f = &s.fishery;
        assert_eq!(
            [
                FishLocation::Uncaught,
                FishLocation::FreshAtShore,
                FishLocation::CarriedRaw,
                FishLocation::RawStored,
                FishLocation::RawAtTable,
                FishLocation::PreservedAtTable,
                FishLocation::CarriedPreserved,
                FishLocation::PreservedStored,
                FishLocation::Home(1),
                FishLocation::Eaten(1)
            ]
            .iter()
            .map(|&l| f.count(l))
            .sum::<usize>(),
            FISH_UNITS
        );
        assert_eq!(
            f.count(FishLocation::CarriedRaw) + f.count(FishLocation::CarriedPreserved),
            usize::from(w.carrying_fish.is_some())
        );
        assert_eq!(
            s.barter.count(SaltLocation::Carried),
            usize::from(w.carrying_salt.is_some())
        );
        assert_eq!(
            [
                SaltLocation::Offered,
                SaltLocation::Paid,
                SaltLocation::Carried,
                SaltLocation::Stored,
                SaltLocation::AtSalting,
                SaltLocation::Consumed
            ]
            .iter()
            .map(|&l| s.barter.count(l))
            .sum::<usize>(),
            SALT_UNITS
        );
        assert_eq!(s.barter.count(SaltLocation::Consumed) as u64, f.batches);
        assert_eq!(
            f.count(FishLocation::PreservedAtTable)
                + f.count(FishLocation::CarriedPreserved)
                + f.count(FishLocation::PreservedStored)
                + f.count(FishLocation::Home(1))
                + f.count(FishLocation::Eaten(1)),
            f.batches as usize * BATCH_SIZE
        );
        assert_eq!(
            s.household.fish_used as usize,
            f.count(FishLocation::Eaten(1))
        );
        assert!(
            usize::from(w.carrying.is_some())
                + usize::from(w.carrying_grain.is_some())
                + usize::from(w.carrying_wool.is_some())
                + usize::from(w.carrying_salt.is_some())
                + usize::from(w.carrying_fish.is_some())
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
    fn catch_is_ordered_finite_and_duplicate_safe() {
        let mut f = Fishery::default();
        assert!(!f.catch(0));
        assert!(f.request_fishing());
        assert!(!f.request_fishing());
        assert!(!f.catch(usize::MAX));
        for i in 0..FISH_UNITS {
            assert!(f.catch(i));
            assert!(!f.catch(i));
        }
        assert!(!f.request_fishing());
        assert_eq!(f.count(FishLocation::FreshAtShore), 4);
    }
    #[test]
    fn preservation_requires_order_and_both_raw_fish_and_actual_table_salt() {
        let mut f = Fishery::default();
        let mut salt = [SaltLocation::Stored; SALT_UNITS];
        f.fish[0] = FishLocation::RawStored;
        f.fish[1] = FishLocation::RawStored;
        assert_eq!(f.salting_salt(&salt), None);
        assert!(f.request_salting());
        assert!(!f.request_salting());
        assert_eq!(f.salting_salt(&salt), Some(0));
        for unavailable in [
            SaltLocation::Offered,
            SaltLocation::Paid,
            SaltLocation::Carried,
            SaltLocation::Consumed,
        ] {
            salt.fill(unavailable);
            assert_eq!(f.salting_salt(&salt), None);
            assert!(!f.preserve(&mut salt));
        }
        salt[0] = SaltLocation::AtSalting;
        assert!(!f.preserve(&mut salt)); // Stored raw fish are not at the table.
        f.fish[0] = FishLocation::RawAtTable;
        assert!(!f.preserve(&mut salt));
        f.fish[1] = FishLocation::RawAtTable;
        assert!(f.preserve(&mut salt));
        assert!(!f.preserve(&mut salt));
        assert_eq!(f.batches, 1);
        assert_eq!(f.count(FishLocation::PreservedAtTable), BATCH_SIZE);
        assert_eq!(salt[0], SaltLocation::Consumed);
    }
    #[test]
    fn fishing_salt_and_home_orders_wait_for_their_own_physical_inputs() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.fishery.request_fishing();
        for _ in 0..3000 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
        }
        assert_eq!(s.fishery.count(FishLocation::RawStored), 4);
        s.fishery.request_salting();
        s.household.request_supply();
        w.advance(100.0, &mut s);
        assert_eq!(s.household.food_used, 0);
        assert!(!s.household.started);
        assert_eq!(s.fishery.count(FishLocation::RawStored), 4);
        s.household.supply_ordered = false; // Isolate output storage before a later delivery order.
        s.barter.salt.fill(SaltLocation::Stored);
        for _ in 0..1500 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
        }
        assert_eq!(s.fishery.count(FishLocation::PreservedStored), 4);
        assert_eq!(s.household.food_used, 0);
        s.household.request_supply();
        w.advance(250.0, &mut s);
        assert_eq!(s.household.food_used, 4);
        assert_eq!(s.household.fish_used, 4);
        assert_eq!(s.fishery.count(FishLocation::Eaten(1)), 4);
        assert!(!s.fishery.request_fishing());
    }
    #[test]
    fn ingredients_are_carried_separately_and_spent_only_after_actual_processing() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.fishery.fish[0] = FishLocation::RawStored;
        s.fishery.fish[1] = FishLocation::RawStored;
        s.barter.salt[0] = SaltLocation::Stored;
        s.fishery.request_salting();
        w.advance(0.1, &mut s);
        assert_eq!(s.barter.salt[0], SaltLocation::Stored);
        w.advance(1.0, &mut s);
        assert_eq!(s.barter.salt[0], SaltLocation::Carried);
        assert_eq!(s.fishery.count(FishLocation::RawStored), 2);
        let mut carried_raw = false;
        for _ in 0..1000 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            carried_raw |=
                s.barter.count(SaltLocation::AtSalting) == 1 && w.carrying_fish.is_some();
            if w.task == Task::Coastal(CoastalTask::Salting) {
                break;
            }
        }
        assert!(carried_raw);
        assert_eq!(w.task, Task::Coastal(CoastalTask::Salting));
        assert_eq!(s.fishery.count(FishLocation::RawAtTable), 2);
        assert_eq!(s.barter.count(SaltLocation::Consumed), 0);
        w.advance(5.0, &mut s);
        assert_eq!(s.barter.count(SaltLocation::Consumed), 0);
        w.advance(1.2, &mut s);
        assert_eq!(s.barter.count(SaltLocation::Consumed), 1);
        assert_eq!(s.fishery.batches, 1);
    }
    #[test]
    fn grain_and_owned_fish_share_one_ration_clock_without_eating_raw_or_other_owned_stock() {
        let mut h = Household::default();
        let mut grain = [GrainLocation::Stored];
        let mut fish = [
            FishLocation::RawStored,
            FishLocation::CarriedPreserved,
            FishLocation::PreservedStored,
            FishLocation::Home(2),
        ];
        h.advance_food(100.0, &mut grain, &mut fish);
        assert!(!h.started);
        grain[0] = GrainLocation::Home(1);
        fish[1] = FishLocation::Home(1);
        fish[2] = FishLocation::Home(1);
        h.advance_food(RATION_SECONDS, &mut grain, &mut fish);
        assert_eq!(h.food_used, 1);
        assert_eq!(h.fish_used, 0);
        h.advance_food(2.0 * RATION_SECONDS, &mut grain, &mut fish);
        assert_eq!(h.food_used, 3);
        assert_eq!(h.fish_used, 2);
        h.advance_food(RATION_SECONDS, &mut grain, &mut fish);
        assert_eq!(h.missed_rations, 1);
        assert_eq!(fish[0], FishLocation::RawStored);
        assert_eq!(fish[3], FishLocation::Home(2));
    }
    #[test]
    fn coastal_work_is_frame_partition_stable_and_home_supply_respects_membership() {
        let mut a = Settlement::default();
        let mut b = Settlement::default();
        for s in [&mut a, &mut b] {
            s.barter.salt.fill(SaltLocation::Stored);
            s.fishery.request_fishing();
            s.fishery.request_salting();
            s.household.request_supply();
        }
        let mut wa = Worker::default();
        let mut wb = Worker::default();
        wa.advance(600.0, &mut a);
        for _ in 0..6000 {
            wb.advance(0.1, &mut b);
            balance(&b, &wb);
        }
        assert_eq!(a.fishery.fish, b.fishery.fish);
        assert_eq!(a.barter.salt, b.barter.salt);
        assert_eq!(a.household.food_used, 4);
        assert_eq!(b.household.food_used, 4);
        assert_eq!(a.household.missed_rations, b.household.missed_rations);
        assert!(wa.position.distance(wb.position) < 0.001);
        let mut other = Settlement::default();
        other.household.request_supply();
        other.fishery.fish.fill(FishLocation::PreservedStored);
        let mut outsider = Worker::for_home(2);
        outsider.advance(100.0, &mut other);
        assert_eq!(other.fishery.count(FishLocation::PreservedStored), 4);
        assert_eq!(other.household.food_used, 0);
    }
    #[test]
    fn full_real_trade_catch_salting_and_meals_conserve_every_chain_over_two_winters() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        order_chain(&mut s);
        s.household.hearth.request();
        s.request_build();
        s.farm.request_reserve();
        for i in 0..3 {
            s.request_harvest(i);
        }
        for _ in 0..15000 {
            if s.farm.harvests < 2 {
                if matches!(s.farm.phase, CropPhase::Fallow | CropPhase::Harvested) {
                    s.farm.request_sow();
                }
                s.farm.request_reap();
            }
            w.advance(0.1, &mut s);
            balance(&s, &w);
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
                live as u64 + s.household.food_used - s.household.fish_used
                    + s.farm.seed_batches_saved
            );
            assert_eq!(
                [
                    LogLocation::InTree,
                    LogLocation::Ground,
                    LogLocation::Carried,
                    LogLocation::Stored,
                    LogLocation::Installed,
                    LogLocation::Home(1),
                    LogLocation::Burned(1)
                ]
                .iter()
                .map(|&l| s.count(l))
                .sum::<usize>(),
                TOTAL_LOGS
            );
            assert_eq!(
                s.flock.count(WoolLocation::Trader) / 2,
                SALT_UNITS - s.barter.count(SaltLocation::Offered)
            );
        }
        assert_eq!(s.installed(), BUILD_COST);
        assert!(s.construction.complete);
        assert_eq!(s.household.hearth.burned as usize, TOTAL_LOGS - BUILD_COST);
        assert_eq!(s.flock.count(WoolLocation::Trader), WOOL_UNITS);
        assert_eq!(s.barter.count(SaltLocation::Consumed), SALT_UNITS);
        assert_eq!(s.fishery.batches, 2);
        assert_eq!(s.household.fish_used, 4);
        assert_eq!(s.farm.harvests, 2);
        assert_eq!(s.household.food_used, 10);
        assert_eq!(w.task, Task::Idle);
    }
    #[test]
    fn fish_only_home_rations_are_stable_across_time_partitions() {
        let mut a = Household::default();
        let mut b = Household::default();
        let mut fa = [FishLocation::Home(1); 4];
        let mut fb = fa;
        a.advance_food(200.0, &mut [], &mut fa);
        for _ in 0..800 {
            b.advance_food(0.25, &mut [], &mut fb);
        }
        assert_eq!(fa, fb);
        assert_eq!(a.food_used, 4);
        assert_eq!(a.fish_used, 4);
        assert_eq!(a.food_used, b.food_used);
        assert_eq!(a.missed_rations, b.missed_rations);
    }
    #[test]
    fn actual_fish_salt_worker_and_visitor_entities_persist_through_both_views_and_meals() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((
                SimulationPlugin,
                FishingPlugin,
                LivestockPlugin,
                TradePlugin,
            ));
        app.update();
        let fish: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<FishVisual>>()
            .iter(app.world())
            .collect();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        let salt: Vec<_> = app
            .world_mut()
            .query::<(Entity, &Name)>()
            .iter(app.world())
            .filter_map(|(e, n)| n.as_str().starts_with("Salt pouch ").then_some(e))
            .collect();
        assert_eq!(fish.len(), FISH_UNITS);
        assert_eq!(salt.len(), SALT_UNITS);
        let count = app.world().entities().len();
        order_chain(&mut app.world_mut().resource_mut::<Settlement>());
        for frame in 0..6500 {
            if frame % 11 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            balance(
                app.world().resource::<Settlement>(),
                app.world().get::<Worker>(worker).unwrap(),
            );
        }
        assert_eq!(app.world().resource::<Settlement>().household.fish_used, 4);
        assert_eq!(
            app.world()
                .resource::<Settlement>()
                .barter
                .count(SaltLocation::Consumed),
            SALT_UNITS
        );
        assert_eq!(app.world().entities().len(), count);
        for id in fish.into_iter().chain(salt) {
            assert_eq!(
                *app.world().get::<Visibility>(id).unwrap(),
                Visibility::Hidden
            );
        }
    }
}
