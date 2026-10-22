//! A small persistent flock and finite fleece transport, shared by both cameras.
use crate::simulation::{STOCKPILE, Settlement, Worker};
use bevy::prelude::*;

pub const SHEEP_COUNT: usize = 2;
pub const WOOL_PER_SHEEP: usize = 2;
pub const WOOL_UNITS: usize = SHEEP_COUNT * WOOL_PER_SHEEP;
pub const PASTURE_GATE: Vec2 = Vec2::new(6.5, -1.8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WoolLocation {
    Fleece,
    Ground,
    Carried,
    Stored,
    Trader,
}
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Sheep {
    #[serde(with = "crate::session::vector")]
    pub position: Vec2,
    pub yaw: f32,
    pub ordered: bool,
    pub shorn: bool,
    grazing_seconds: f64,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Flock {
    pub sheep: [Sheep; SHEEP_COUNT],
    pub wool: [WoolLocation; WOOL_UNITS],
    #[serde(with = "crate::session::vectors")]
    pub ground: [Vec2; WOOL_UNITS],
}
fn home(i: usize) -> Vec2 {
    Vec2::new(6.1 + i as f32 * 1.4, -3.7)
}
impl Default for Flock {
    fn default() -> Self {
        Self {
            sheep: std::array::from_fn(|i| Sheep {
                position: home(i),
                yaw: std::f32::consts::FRAC_PI_2,
                ordered: false,
                shorn: false,
                grazing_seconds: 0.0,
            }),
            wool: [WoolLocation::Fleece; WOOL_UNITS],
            ground: [Vec2::ZERO; WOOL_UNITS],
        }
    }
}
impl Flock {
    pub fn request_shear(&mut self) -> bool {
        let mut accepted = false;
        for sheep in &mut self.sheep {
            if !sheep.ordered && !sheep.shorn {
                sheep.ordered = true;
                accepted = true;
            }
        }
        accepted
    }
    pub fn graze(&mut self, seconds: f64) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        for (i, sheep) in self.sheep.iter_mut().enumerate() {
            // An ordered sheep holds still for the worker; after shearing it can
            // graze again. Ground wool keeps its own fixed drop position.
            if sheep.ordered && !sheep.shorn {
                continue;
            }
            sheep.grazing_seconds += seconds;
            let phase = (sheep.grazing_seconds * (0.25 + i as f64 * 0.05)) as f32;
            sheep.position = home(i) + Vec2::new(phase.sin() * 0.18, (phase.cos() - 1.0) * 0.12);
            sheep.yaw = (phase.cos() * 0.18).atan2(-phase.sin() * 0.12);
        }
    }
    pub fn clip(&mut self, i: usize) -> bool {
        let Some(sheep) = self.sheep.get_mut(i) else {
            return false;
        };
        if !sheep.ordered || sheep.shorn {
            return false;
        }
        sheep.shorn = true;
        for offset in 0..WOOL_PER_SHEEP {
            let index = i * WOOL_PER_SHEEP + offset;
            self.wool[index] = WoolLocation::Ground;
            self.ground[index] = sheep.position + Vec2::new(offset as f32 * 0.4 - 0.2, 0.75);
        }
        true
    }
    pub fn count(&self, location: WoolLocation) -> usize {
        self.wool.iter().filter(|&&w| w == location).count()
    }
}
/// Three visible fence sides; the entire southern approach remains open.
pub fn fence_blocks(p: Vec2) -> bool {
    ((p.x - 5.2).abs() < 0.25 || (p.x - 8.4).abs() < 0.25) && (-5.1..=-2.2).contains(&p.y)
        || (p.y + 4.9).abs() < 0.25 && (4.95..=8.65).contains(&p.x)
}
pub struct LivestockPlugin;
impl Plugin for LivestockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, sync);
    }
}
#[derive(Component)]
enum LivestockVisual {
    Animal(usize),
    Coat(usize),
    Head(usize),
    Wool(usize),
}
#[derive(Resource)]
struct CoatColors {
    fleece: Handle<StandardMaterial>,
    shorn: Handle<StandardMaterial>,
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let timber = materials.add(Color::srgb(0.34, 0.24, 0.14));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    for x in [5.2, 8.4] {
        for z in [-4.9, -3.6, -2.4] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(x, 0.65, z).with_scale(Vec3::new(0.09, 0.9, 0.09)),
            ));
        }
        for y in [0.55, 0.9] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(x, y, -3.65).with_scale(Vec3::new(0.06, 0.055, 2.5)),
            ));
        }
    }
    for y in [0.55, 0.9] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(timber.clone()),
            Transform::from_xyz(6.8, y, -4.9).with_scale(Vec3::new(3.2, 0.055, 0.06)),
        ));
    }
    let fleece = materials.add(Color::srgb(0.77, 0.75, 0.65));
    let shorn = materials.add(Color::srgb(0.49, 0.46, 0.39));
    let dark = materials.add(Color::srgb(0.15, 0.14, 0.12));
    let sphere = meshes.add(Sphere::new(1.0).mesh().ico(2).expect("sheep mesh"));
    for i in 0..SHEEP_COUNT {
        commands
            .spawn((
                Transform::default(),
                Visibility::Inherited,
                LivestockVisual::Animal(i),
            ))
            .with_children(|animal| {
                animal.spawn((
                    Mesh3d(sphere.clone()),
                    MeshMaterial3d(fleece.clone()),
                    Transform::from_xyz(0.0, 0.62, 0.0).with_scale(Vec3::new(0.42, 0.38, 0.62)),
                    LivestockVisual::Coat(i),
                ));
                for x in [-0.23, 0.23] {
                    for z in [-0.32, 0.32] {
                        animal.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(dark.clone()),
                            Transform::from_xyz(x, 0.27, z)
                                .with_scale(Vec3::new(0.075, 0.50, 0.075)),
                        ));
                    }
                }
                animal
                    .spawn((
                        Transform::from_xyz(0.0, 0.65, 0.58),
                        Visibility::Inherited,
                        LivestockVisual::Head(i),
                    ))
                    .with_children(|head| {
                        head.spawn((
                            Mesh3d(sphere.clone()),
                            MeshMaterial3d(dark.clone()),
                            Transform::default().with_scale(Vec3::new(0.19, 0.22, 0.27)),
                        ));
                        for x in [-0.22, 0.22] {
                            head.spawn((
                                Mesh3d(sphere.clone()),
                                MeshMaterial3d(dark.clone()),
                                Transform::from_xyz(x, 0.12, -0.01)
                                    .with_scale(Vec3::new(0.15, 0.045, 0.06)),
                            ));
                        }
                    });
            });
    }
    for i in 0..WOOL_UNITS {
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(fleece.clone()),
            Transform::default(),
            Visibility::Hidden,
            LivestockVisual::Wool(i),
            Name::new(format!("Wool {}", i)),
        ));
    }
    commands.insert_resource(CoatColors { fleece, shorn });
}
type LivestockParts<'w, 's> = Query<
    'w,
    's,
    (
        &'static LivestockVisual,
        &'static mut Transform,
        &'static mut Visibility,
        Option<&'static mut MeshMaterial3d<StandardMaterial>>,
    ),
>;
fn sync(
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    colors: Res<CoatColors>,
    mut parts: LivestockParts,
) {
    let flock = &settlement.flock;

    for (part, mut transform, mut visible, material) in &mut parts {
        match *part {
            LivestockVisual::Animal(i) => {
                let sheep = &flock.sheep[i];
                transform.translation = Vec3::new(sheep.position.x, 0.2, sheep.position.y);
                transform.rotation = Quat::from_rotation_y(sheep.yaw);
            }
            LivestockVisual::Coat(i) => {
                let shorn = flock.sheep[i].shorn;
                transform.scale = if shorn {
                    Vec3::new(0.34, 0.28, 0.52)
                } else {
                    Vec3::new(0.42, 0.38, 0.62)
                };
                let color = if shorn { &colors.shorn } else { &colors.fleece };
                if let Some(mut material) = material
                    && material.0 != *color
                {
                    material.0 = color.clone();
                }
            }
            LivestockVisual::Head(i) => {
                let sheep = &flock.sheep[i];
                transform.rotation = Quat::from_rotation_x(if sheep.ordered && !sheep.shorn {
                    0.0
                } else {
                    0.35 + (sheep.grazing_seconds as f32 * 1.5).sin() * 0.12
                });
            }
            LivestockVisual::Wool(i) => {
                let (p, y) = match flock.wool[i] {
                    WoolLocation::Fleece => {
                        *visible = Visibility::Hidden;
                        continue;
                    }
                    WoolLocation::Ground => (flock.ground[i], 0.4),
                    WoolLocation::Carried => {
                        let Some(worker) = workers.iter().find(|w| w.carrying_wool == Some(i))
                        else {
                            *visible = Visibility::Hidden;
                            continue;
                        };
                        (worker.position + Vec2::new(0.3, 0.0), 1.0)
                    }
                    WoolLocation::Trader => (crate::trade::wool_position(i), 0.4),
                    WoolLocation::Stored => {
                        (STOCKPILE + Vec2::new(0.6 + i as f32 * 0.28, -0.6), 0.4)
                    }
                };
                *visible = Visibility::Visible;
                transform.translation = Vec3::new(p.x, y, p.y);
                transform.scale = Vec3::new(0.24, 0.18, 0.24);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        farming::{CropPhase, GRAIN_UNITS, GrainLocation},
        simulation::{BUILD_COST, LogLocation, SimulationPlugin, TOTAL_LOGS, Task},
    };
    fn balance(s: &Settlement, w: &Worker) {
        let flock = &s.flock;
        assert_eq!(
            flock.count(WoolLocation::Carried),
            usize::from(w.carrying_wool.is_some())
        );
        assert!(
            usize::from(w.carrying.is_some())
                + usize::from(w.carrying_grain.is_some())
                + usize::from(w.carrying_wool.is_some())
                <= 1
        );
        assert_eq!(
            [
                WoolLocation::Fleece,
                WoolLocation::Ground,
                WoolLocation::Carried,
                WoolLocation::Stored
            ]
            .iter()
            .map(|&v| flock.count(v))
            .sum::<usize>(),
            WOOL_UNITS
        );
        for (i, sheep) in flock.sheep.iter().enumerate() {
            assert!(crate::world::walkable(sheep.position));
            assert_eq!(
                flock.wool[i * WOOL_PER_SHEEP..(i + 1) * WOOL_PER_SHEEP]
                    .iter()
                    .filter(|&&v| v == WoolLocation::Fleece)
                    .count(),
                if sheep.shorn { 0 } else { WOOL_PER_SHEEP }
            );
        }
        assert!(
            crate::world::walkable(w.position),
            "blocked route {:?} {:?}",
            w.task,
            w.position
        );
    }
    #[test]
    fn grazing_is_deterministic_and_ordered_sheep_hold_still_without_moving_dropped_wool() {
        let mut a = Flock::default();
        let mut b = Flock::default();
        a.graze(10.0);
        for _ in 0..100 {
            b.graze(0.1);
        }
        for i in 0..SHEEP_COUNT {
            assert!(a.sheep[i].position.distance(b.sheep[i].position) < 0.0001);
        }
        assert!(!a.clip(0));
        assert!(!a.clip(99));
        let held = a.sheep[0].position;
        assert!(a.request_shear());
        a.graze(10.0);
        assert_eq!(a.sheep[0].position, held);
        assert!(a.clip(0));
        assert!(!a.clip(0));
        let ground = a.ground;
        a.graze(10.0);
        assert_eq!(a.ground, ground);
        assert_ne!(a.sheep[0].position, held);
        let before = a.sheep[0].position;
        for bad in [f64::NAN, f64::INFINITY, -1.0] {
            a.graze(bad);
        }
        assert_eq!(a.sheep[0].position, before);
    }
    #[test]
    fn shearing_requires_orders_and_transports_exactly_four_finite_bundles() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        w.advance(100.0, &mut s);
        assert_eq!(s.flock.count(WoolLocation::Fleece), WOOL_UNITS);
        assert!(s.flock.request_shear());
        assert!(!s.flock.request_shear());
        let mut saw_shear = false;
        let mut saw_carry = false;
        let mut saw_ground = false;
        for _ in 0..4000 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            saw_shear |= w.task == Task::Shearing;
            saw_carry |= w.carrying_wool.is_some();
            saw_ground |= s.flock.count(WoolLocation::Ground) > 0;
        }
        assert!(saw_shear && saw_carry && saw_ground);
        assert!(s.flock.sheep.iter().all(|s| s.shorn));
        assert_eq!(s.flock.count(WoolLocation::Stored), WOOL_UNITS);
        assert_eq!(w.task, Task::Idle);
        for _ in 0..10 {
            assert!(!s.flock.request_shear());
            w.advance(100.0, &mut s);
        }
        assert_eq!(s.flock.count(WoolLocation::Stored), WOOL_UNITS);
        assert_eq!(s.count(LogLocation::InTree), TOTAL_LOGS);
        assert_eq!(s.farm.count(GrainLocation::Stored), 0);
    }
    #[test]
    fn queued_farm_and_timber_jobs_finish_before_shearing_and_all_routes_stay_clear() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.farm.request_sow();
        s.flock.request_shear();
        s.request_build();
        for i in 0..3 {
            s.request_harvest(i);
        }
        for _ in 0..7000 {
            if s.farm.phase == CropPhase::Ripe {
                s.farm.request_reap();
            }
            w.advance(0.1, &mut s);
            balance(&s, &w);
        }
        assert!(s.construction.complete);
        assert_eq!(s.stored(), TOTAL_LOGS - BUILD_COST);
        assert_eq!(s.farm.count(GrainLocation::Stored), GRAIN_UNITS);
        assert_eq!(s.flock.count(WoolLocation::Stored), WOOL_UNITS);
    }
    #[test]
    fn fence_has_three_blocking_sides_and_a_walkable_southern_approach() {
        for p in [
            Vec2::new(5.2, -3.6),
            Vec2::new(8.4, -3.6),
            Vec2::new(6.8, -4.9),
        ] {
            assert!(fence_blocks(p));
            assert!(!crate::world::walkable(p));
        }
        for p in [PASTURE_GATE, Vec2::new(6.5, -2.4), home(0), home(1)] {
            assert!(crate::world::walkable(p));
        }
        assert!(crate::world::slide(Vec2::new(6.8, -4.0), Vec2::new(0.0, -3.0)).y > -4.9);
    }
    #[test]
    fn the_same_animals_and_wool_entities_survive_camera_switches_and_deliveries() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((SimulationPlugin, LivestockPlugin));
        app.update();
        let ids: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<LivestockVisual>>()
            .iter(app.world())
            .collect();
        assert_eq!(ids.len(), SHEEP_COUNT * 3 + WOOL_UNITS);
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .resource_mut::<Settlement>()
            .flock
            .request_shear();
        for frame in 0..3000 {
            if frame % 7 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
        }
        assert!(app.world().get::<Worker>(worker).is_some());
        assert!(
            ids.iter()
                .all(|&e| app.world().get::<LivestockVisual>(e).is_some())
        );
        assert_eq!(
            app.world()
                .resource::<Settlement>()
                .flock
                .count(WoolLocation::Stored),
            WOOL_UNITS
        );
        for id in ids {
            if matches!(
                app.world().get::<LivestockVisual>(id).unwrap(),
                LivestockVisual::Wool(_)
            ) {
                assert_eq!(
                    *app.world().get::<Visibility>(id).unwrap(),
                    Visibility::Visible
                );
            }
        }
    }
}
