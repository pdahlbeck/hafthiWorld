//! Repeatable crop cycles with physical seed reservation and bounded sack slots. The field reads the same worker and settlement in both views.
use crate::simulation::{STOCKPILE, Settlement, Worker};
use bevy::prelude::*;

pub const FIELD_CENTER: Vec2 = Vec2::new(6.3, 4.6);
pub const FIELD_WORK: Vec2 = Vec2::new(5.0, 3.8);
pub const SEED_UNITS: usize = 4;
pub const GRAIN_UNITS: usize = 4;
pub const GROW_SECONDS: f64 = 30.0;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CropPhase {
    #[default]
    Fallow,
    SowOrdered,
    Growing,
    Ripe,
    Harvested,
}
impl CropPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fallow => "unplanted",
            Self::SowOrdered => "sowing ordered",
            Self::Growing => "growing",
            Self::Ripe => "ripe",
            Self::Harvested => "harvested",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SeedLocation {
    #[default]
    Stored,
    Carried,
    Sown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GrainLocation {
    Unproduced,
    Ground,
    Carried,
    Stored,
    Home(u32),
    Consumed(u32),
    SavedSeed,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Farm {
    pub phase: CropPhase,
    pub seeds: SeedLocation,
    pub growth: f64,
    pub harvest_ordered: bool,
    pub grain: [GrainLocation; GRAIN_UNITS],
    pub reserve_ordered: bool,
    pub harvests: u64,
    pub seed_batches_saved: u64,
    seed_saved_this_harvest: bool,
}
impl Default for Farm {
    fn default() -> Self {
        Self {
            phase: CropPhase::Fallow,
            seeds: SeedLocation::Stored,
            growth: 0.0,
            harvest_ordered: false,
            grain: [GrainLocation::Unproduced; GRAIN_UNITS],
            reserve_ordered: false,
            harvests: 0,
            seed_batches_saved: 0,
            seed_saved_this_harvest: false,
        }
    }
}
impl Farm {
    pub fn request_sow(&mut self) -> bool {
        if !matches!(self.phase, CropPhase::Fallow | CropPhase::Harvested)
            || self.seeds != SeedLocation::Stored
        {
            return false;
        }
        self.phase = CropPhase::SowOrdered;
        self.growth = 0.0;
        self.harvest_ordered = false;
        true
    }
    pub fn request_reap(&mut self) -> bool {
        if self.phase != CropPhase::Ripe || self.harvest_ordered || !self.previous_grain_resolved()
        {
            return false;
        }
        self.harvest_ordered = true;
        true
    }
    // Reuse only spent sack slots. Live stocks remain intact during the next sow/growth.
    pub fn previous_grain_resolved(&self) -> bool {
        self.grain.iter().all(|g| {
            matches!(
                g,
                GrainLocation::Unproduced | GrainLocation::Consumed(_) | GrainLocation::SavedSeed
            )
        })
    }
    pub fn request_reserve(&mut self) -> bool {
        if self.reserve_ordered {
            return false;
        }
        self.reserve_ordered = true;
        true
    }
    pub fn seed_candidate(&self) -> Option<usize> {
        if !self.reserve_ordered
            || self.harvests == 0
            || self.seed_saved_this_harvest
            || self.seeds != SeedLocation::Sown
        {
            return None;
        }
        self.grain.iter().position(|&g| g == GrainLocation::Stored)
    }
    pub fn save_seed(&mut self, index: usize) -> bool {
        if self.seed_candidate() != Some(index) {
            return false;
        }
        self.grain[index] = GrainLocation::SavedSeed;
        self.seeds = SeedLocation::Stored;
        self.seed_saved_this_harvest = true;
        self.seed_batches_saved += 1;
        true
    }
    pub fn reap(&mut self) -> bool {
        if self.phase != CropPhase::Ripe || !self.harvest_ordered || !self.previous_grain_resolved()
        {
            return false;
        }
        self.grain.fill(GrainLocation::Ground);
        self.phase = CropPhase::Harvested;
        self.harvest_ordered = false;
        self.harvests += 1;
        self.seed_saved_this_harvest = false;
        true
    }
    pub fn grow(&mut self, seconds: f64) {
        if self.phase == CropPhase::Growing && seconds.is_finite() && seconds > 0.0 {
            self.growth = (self.growth + seconds).min(GROW_SECONDS);
            if self.growth >= GROW_SECONDS {
                self.phase = CropPhase::Ripe;
            }
        }
    }
    pub fn count(&self, location: GrainLocation) -> usize {
        self.grain.iter().filter(|&&g| g == location).count()
    }
    pub fn progress(&self) -> f32 {
        (self.growth / GROW_SECONDS).clamp(0.0, 1.0) as f32
    }
}
pub fn grain_position(i: usize) -> Vec2 {
    FIELD_WORK + Vec2::new(i as f32 * 0.45, 0.35)
}

pub struct FarmingPlugin;
impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, sync);
    }
}
#[derive(Component)]
pub(crate) enum FarmVisual {
    Crop,
    Grain(usize),
    Seeds,
}
#[derive(Resource)]
struct CropColors {
    green: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let soil = materials.add(Color::srgb(0.28, 0.19, 0.11));
    let furrow = materials.add(Color::srgb(0.39, 0.28, 0.15));
    let cube = meshes.add(crate::art::softened_box(Vec3::ONE, 0.09));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(soil),
        Transform::from_xyz(FIELD_CENTER.x, 0.22, FIELD_CENTER.y)
            .with_scale(Vec3::new(3.0, 0.04, 1.8)),
    ));
    for row in 0..6 {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(furrow.clone()),
            Transform::from_xyz(
                FIELD_CENTER.x,
                0.25,
                FIELD_CENTER.y - 0.7 + row as f32 * 0.28,
            )
            .with_scale(Vec3::new(2.8, 0.025, 0.06)),
        ));
    }
    let green = materials.add(Color::srgb(0.32, 0.48, 0.16));
    let gold = materials.add(Color::srgb(0.78, 0.60, 0.24));
    let stalk = meshes.add(Cuboid::new(0.025, 0.60, 0.025));
    let ear = meshes.add(Sphere::new(0.06).mesh().ico(1).expect("crop ear mesh"));
    for row in 0..6 {
        for col in 0..12 {
            commands
                .spawn((
                    Transform::from_xyz(
                        FIELD_CENTER.x - 1.3 + col as f32 * 0.23,
                        0.26,
                        FIELD_CENTER.y - 0.7 + row as f32 * 0.28,
                    ),
                    Visibility::Hidden,
                    FarmVisual::Crop,
                ))
                .with_children(|p| {
                    p.spawn((
                        Mesh3d(stalk.clone()),
                        MeshMaterial3d(green.clone()),
                        Transform::from_xyz(0.0, 0.3, 0.0),
                    ));
                    p.spawn((
                        Mesh3d(ear.clone()),
                        MeshMaterial3d(green.clone()),
                        Transform::from_xyz(0.0, 0.62, 0.0).with_scale(Vec3::new(0.7, 2.1, 0.7)),
                    ));
                });
        }
    }
    commands.insert_resource(CropColors { green, gold });
    let sack = meshes.add(Sphere::new(0.20).mesh().ico(2).expect("sack mesh"));
    let cloth = materials.add(Color::srgb(0.66, 0.52, 0.32));
    for i in 0..GRAIN_UNITS {
        commands.spawn((
            Mesh3d(sack.clone()),
            MeshMaterial3d(cloth.clone()),
            Transform::default(),
            Visibility::Hidden,
            FarmVisual::Grain(i),
            Name::new(format!("Grain {}", i)),
        ));
    }
    commands.spawn((
        Mesh3d(sack),
        MeshMaterial3d(cloth),
        Transform::default(),
        Visibility::Visible,
        FarmVisual::Seeds,
    ));
}
fn sync(
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    colors: Res<CropColors>,
    mut visuals: Query<(
        &FarmVisual,
        &mut Transform,
        &mut Visibility,
        Option<&Children>,
    )>,
    mut materials: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let farm = &settlement.farm;

    for (visual, mut transform, mut visible, children) in &mut visuals {
        match *visual {
            FarmVisual::Crop => {
                *visible = if matches!(farm.phase, CropPhase::Growing | CropPhase::Ripe) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                transform.scale.y = 0.15 + farm.progress() * 0.85;
                if let Some(children) = children {
                    let color = if farm.progress() >= 0.8 {
                        &colors.gold
                    } else {
                        &colors.green
                    };
                    for child in children.iter() {
                        if let Ok(mut material) = materials.get_mut(child)
                            && material.0 != *color
                        {
                            material.0 = color.clone();
                        }
                    }
                }
            }
            FarmVisual::Grain(i) => {
                let p = match farm.grain[i] {
                    GrainLocation::Unproduced
                    | GrainLocation::Consumed(_)
                    | GrainLocation::SavedSeed => {
                        *visible = Visibility::Hidden;
                        continue;
                    }
                    GrainLocation::Home(_) => crate::household::pantry_position(i),
                    GrainLocation::Ground => grain_position(i),
                    GrainLocation::Carried => {
                        let Some(worker) = workers.iter().find(|w| w.carrying_grain == Some(i))
                        else {
                            *visible = Visibility::Hidden;
                            continue;
                        };
                        worker.position + Vec2::new(0.3, 0.0)
                    }
                    GrainLocation::Stored => STOCKPILE + Vec2::new(-0.6 + i as f32 * 0.35, 0.8),
                };
                *visible = Visibility::Visible;
                transform.translation = Vec3::new(
                    p.x,
                    if farm.grain[i] == GrainLocation::Carried {
                        1.0
                    } else {
                        0.43
                    },
                    p.y,
                );
                transform.scale = Vec3::new(0.8, 1.2, 0.8);
            }
            FarmVisual::Seeds => {
                *visible = if farm.seeds == SeedLocation::Sown {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
                let (p, y) = if farm.seeds == SeedLocation::Carried {
                    let Some(worker) = workers.iter().find(|w| w.id == 1) else {
                        *visible = Visibility::Hidden;
                        continue;
                    };
                    (worker.position + Vec2::new(0.3, 0.0), 1.0)
                } else {
                    (STOCKPILE + Vec2::new(-0.7, -0.5), 0.43)
                };
                transform.translation = Vec3::new(p.x, y, p.y);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        simulation::{BUILD_COST, LogLocation, SimulationPlugin, TOTAL_LOGS, Task},
    };
    fn balance(s: &Settlement, w: &Worker) {
        let f = &s.farm;
        assert_eq!(
            f.count(GrainLocation::Carried),
            usize::from(w.carrying_grain.is_some())
        );
        assert!(!(w.carrying.is_some() && w.carrying_grain.is_some()));
        assert_eq!(
            f.count(GrainLocation::Unproduced),
            if f.phase == CropPhase::Harvested {
                0
            } else {
                GRAIN_UNITS
            }
        );
        assert!(
            crate::world::walkable(w.position),
            "worker crossed obstacle at {:?} in {:?}",
            w.position,
            w.task
        );
    }
    #[test]
    fn seed_reservation_needs_real_stored_grain_and_saves_only_once_per_harvest() {
        let mut f = Farm::default();
        assert!(f.request_reserve());
        assert!(!f.request_reserve());
        assert_eq!(f.seed_candidate(), None);
        assert!(!f.save_seed(usize::MAX));
        assert!(f.request_sow());
        f.seeds = SeedLocation::Sown;
        f.phase = CropPhase::Ripe;
        assert!(f.request_reap());
        assert!(f.reap());
        assert_eq!(f.seed_candidate(), None); // Grain on the ground cannot become seed.
        f.grain[0] = GrainLocation::Stored;
        assert!(!f.save_seed(1));
        assert!(f.save_seed(0));
        assert_eq!(f.count(GrainLocation::SavedSeed), 1);
        assert_eq!(f.seeds, SeedLocation::Stored);
        assert_eq!(f.seed_batches_saved, 1);
        assert!(f.request_sow());
        assert_eq!(f.growth, 0.0);
        f.seeds = SeedLocation::Sown;
        f.grain[1] = GrainLocation::Stored;
        assert!(!f.save_seed(1)); // New sowing must not save a second sack of the old crop.
        assert_eq!(f.seed_batches_saved, 1);
        assert_eq!(f.grain[1], GrainLocation::Stored);
    }
    #[test]
    fn reaping_never_overwrites_live_grain_from_the_previous_crop() {
        for location in [
            GrainLocation::Ground,
            GrainLocation::Carried,
            GrainLocation::Stored,
            GrainLocation::Home(1),
        ] {
            let mut f = Farm::default();
            f.phase = CropPhase::Ripe;
            f.grain[0] = location;
            assert!(!f.request_reap());
            f.harvest_ordered = true; // Even an obsolete order cannot bypass the safeguard.
            let before = f.grain;
            assert!(!f.reap());
            assert_eq!(f.grain, before);
            assert_eq!(f.harvests, 0);
            f.harvest_ordered = false;
            f.grain[0] = GrainLocation::Consumed(1);
            assert!(f.request_reap());
            assert!(f.reap());
            assert_eq!(f.count(GrainLocation::Ground), GRAIN_UNITS);
            assert!(!f.reap());
        }
    }
    fn repeat_orders(s: &mut Settlement, max_harvests: u64) {
        if s.farm.harvests < max_harvests {
            if matches!(s.farm.phase, CropPhase::Fallow | CropPhase::Harvested) {
                s.farm.request_sow();
            }
            if s.farm.phase == CropPhase::Ripe {
                s.farm.request_reap();
            }
        }
    }
    fn repeat_balance(s: &Settlement, w: &Worker) {
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
            s.farm.count(GrainLocation::Carried),
            usize::from(w.carrying_grain.is_some())
        );
        assert!(
            usize::from(w.carrying_grain.is_some())
                + usize::from(w.carrying.is_some())
                + usize::from(w.carrying_wool.is_some())
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
    fn three_real_harvests_supply_nine_meals_and_three_seed_batches_without_losing_material() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.farm.request_reserve();
        s.household.request_supply();
        s.request_build();
        for i in 0..3 {
            s.request_harvest(i);
        }
        s.flock.request_shear();
        let mut saved_before_food = false;
        let mut growing_with_old_food = false;
        let mut waiting_for_old_food = false;
        for _ in 0..14000 {
            repeat_orders(&mut s, 3);
            w.advance(0.1, &mut s);
            repeat_balance(&s, &w);
            if s.farm.seed_batches_saved == 1 && s.household.food_used == 0 {
                saved_before_food = true;
            }
            growing_with_old_food |=
                s.farm.phase == CropPhase::Growing && !s.farm.previous_grain_resolved();
            waiting_for_old_food |= s.farm.phase == CropPhase::Ripe
                && !s.farm.previous_grain_resolved()
                && !s.farm.harvest_ordered;
        }
        assert!(saved_before_food && growing_with_old_food && waiting_for_old_food);
        assert_eq!(s.farm.harvests, 3);
        assert_eq!(s.farm.seed_batches_saved, 3);
        assert_eq!(s.household.food_used, 9);
        assert_eq!(s.farm.seeds, SeedLocation::Stored);
        assert!(s.farm.previous_grain_resolved());
        assert_eq!(w.task, Task::Idle);
        assert!(s.construction.complete);
        assert_eq!(s.installed(), BUILD_COST);
        assert_eq!(s.stored(), TOTAL_LOGS - BUILD_COST);
        assert_eq!(
            s.flock.count(crate::livestock::WoolLocation::Stored),
            crate::livestock::WOOL_UNITS
        );
    }
    #[test]
    fn no_reserve_order_leaves_all_four_sacks_for_food_and_no_seed_to_replant() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.household.request_supply();
        for _ in 0..4000 {
            repeat_orders(&mut s, 2);
            w.advance(0.1, &mut s);
            repeat_balance(&s, &w);
        }
        assert_eq!(s.household.food_used, 4);
        assert_eq!(s.farm.harvests, 1);
        assert_eq!(s.farm.seed_batches_saved, 0);
        assert!(!s.farm.request_sow());
        assert!(s.farm.request_reserve());
        w.advance(100.0, &mut s);
        assert!(!s.farm.request_sow()); // Consumed food cannot be recovered as seed.
    }
    #[test]
    fn repeated_crops_preserve_worker_field_and_sack_entities_in_both_views() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((SimulationPlugin, FarmingPlugin));
        app.update();
        let ids: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<FarmVisual>>()
            .iter(app.world())
            .collect();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.farm.request_reserve();
            s.household.request_supply();
        }
        let mut saw_new_growth = false;
        for frame in 0..5500 {
            repeat_orders(&mut app.world_mut().resource_mut::<Settlement>(), 2);
            if frame % 13 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            let s = app.world().resource::<Settlement>();
            repeat_balance(s, app.world().get::<Worker>(worker).unwrap());
            if s.farm.harvests == 1 && s.farm.phase == CropPhase::Growing {
                saw_new_growth = true;
                for &id in &ids {
                    if matches!(app.world().get::<FarmVisual>(id), Some(FarmVisual::Crop)) {
                        assert_eq!(
                            *app.world().get::<Visibility>(id).unwrap(),
                            Visibility::Visible
                        );
                        assert!(app.world().get::<Transform>(id).unwrap().scale.y <= 1.0);
                    }
                }
            }
        }
        assert!(saw_new_growth);
        let s = app.world().resource::<Settlement>();
        assert_eq!(s.farm.harvests, 2);
        assert_eq!(s.household.food_used, 6);
        assert_eq!(s.farm.seed_batches_saved, 2);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<FarmVisual>>()
                .iter(app.world())
                .count(),
            ids.len()
        );
        for id in ids {
            assert!(app.world().get::<FarmVisual>(id).is_some());
            let expected = if matches!(app.world().get::<FarmVisual>(id), Some(FarmVisual::Seeds)) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            assert_eq!(*app.world().get::<Visibility>(id).unwrap(), expected);
        }
    }
    #[test]
    fn seed_is_carried_then_planted_and_ripe_crop_requires_a_new_order() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        w.advance(100.0, &mut s);
        assert_eq!(s.farm.phase, CropPhase::Fallow);
        assert!(!s.farm.request_reap());
        assert!(s.farm.request_sow());
        assert!(!s.farm.request_sow());
        let mut saw_seeds = false;
        let mut saw_growth = false;
        for _ in 0..800 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            saw_seeds |= s.farm.seeds == SeedLocation::Carried;
            saw_growth |= s.farm.phase == CropPhase::Growing;
            if s.farm.phase == CropPhase::Growing {
                assert!(!s.farm.request_reap());
            }
        }
        assert!(saw_seeds && saw_growth);
        assert_eq!(s.farm.seeds, SeedLocation::Sown);
        assert_eq!(s.farm.phase, CropPhase::Ripe);
        assert_eq!(s.farm.count(GrainLocation::Stored), 0);
        assert_eq!(w.task, Task::Idle);
        assert!(!s.farm.request_sow());
    }
    #[test]
    fn reaping_delivers_exactly_four_sacks_and_repeated_orders_cannot_regrow_them() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.farm.request_sow();
        w.advance(80.0, &mut s);
        assert!(s.farm.request_reap());
        assert!(!s.farm.request_reap());
        let mut carried = false;
        let mut loose = false;
        for _ in 0..1500 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            carried |= w.carrying_grain.is_some();
            loose |= s.farm.count(GrainLocation::Ground) > 0;
        }
        assert!(carried && loose);
        assert_eq!(s.farm.count(GrainLocation::Stored), GRAIN_UNITS);
        assert_eq!(w.task, Task::Idle);
        for _ in 0..10 {
            assert!(!s.farm.request_reap());
            assert!(!s.farm.request_sow());
            w.advance(40.0, &mut s);
        }
        assert_eq!(s.farm.count(GrainLocation::Stored), GRAIN_UNITS);
        assert_eq!(s.count(LogLocation::InTree), TOTAL_LOGS);
    }
    #[test]
    fn farming_and_timber_work_share_safe_routes_and_preserve_both_stocks() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.farm.request_sow();
        // Start new timber work while the worker is still at the field. It must
        // finish its return through the south gateway before accepting that job.
        for _ in 0..300 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            if s.farm.phase == CropPhase::Growing {
                break;
            }
        }
        assert_eq!(s.farm.phase, CropPhase::Growing);
        s.request_build();
        for i in 0..3 {
            s.request_harvest(i);
        }
        let mut requested = false;
        for _ in 0..4000 {
            if s.farm.phase == CropPhase::Ripe && !requested {
                assert!(s.farm.request_reap());
                requested = true;
            }
            w.advance(0.1, &mut s);
            balance(&s, &w);
        }
        assert!(requested && s.construction.complete);
        assert_eq!(s.installed(), BUILD_COST);
        assert_eq!(s.stored(), TOTAL_LOGS - BUILD_COST);
        assert_eq!(s.farm.count(GrainLocation::Stored), GRAIN_UNITS);
    }
    #[test]
    fn growth_rejects_invalid_time_and_work_is_stable_across_frame_partitions() {
        let mut a = Settlement::default();
        let mut b = Settlement::default();
        let mut wa = Worker::default();
        let mut wb = Worker::default();
        a.farm.request_sow();
        b.farm.request_sow();
        wa.advance(20.0, &mut a);
        for _ in 0..200 {
            wb.advance(0.1, &mut b);
        }
        assert_eq!(a.farm.phase, b.farm.phase);
        assert_eq!(a.farm.seeds, b.farm.seeds);
        assert!(wa.position.distance(wb.position) < 0.001);
        assert!((a.farm.growth - b.farm.growth).abs() < 0.03);
        let before = a.farm.growth;
        for dt in [f64::NAN, f64::INFINITY, -3.0] {
            a.farm.grow(dt);
        }
        assert_eq!(a.farm.growth, before);
    }
    #[test]
    fn crop_and_cargo_entities_persist_through_camera_switches_and_complete_delivery() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((SimulationPlugin, FarmingPlugin));
        app.update();
        let visuals: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<FarmVisual>>()
            .iter(app.world())
            .collect();
        assert_eq!(visuals.len(), 72 + GRAIN_UNITS + 1);
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .resource_mut::<Settlement>()
            .farm
            .request_sow();
        for frame in 0..1500 {
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
        assert!(app.world().get::<Worker>(worker).is_some());
        assert!(
            visuals
                .iter()
                .all(|&e| app.world().get::<FarmVisual>(e).is_some())
        );
        assert_eq!(
            app.world()
                .resource::<Settlement>()
                .farm
                .count(GrainLocation::Stored),
            GRAIN_UNITS
        );
        for id in visuals {
            match app.world().get::<FarmVisual>(id).unwrap() {
                FarmVisual::Crop | FarmVisual::Seeds => assert_eq!(
                    *app.world().get::<Visibility>(id).unwrap(),
                    Visibility::Hidden
                ),
                FarmVisual::Grain(_) => assert_eq!(
                    *app.world().get::<Visibility>(id).unwrap(),
                    Visibility::Visible
                ),
            }
        }
    }
}
