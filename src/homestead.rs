//! A fixed second farmstead: material-backed construction and owned household demand.
use crate::{
    farming::GrainLocation,
    fishing::FishLocation,
    household::Household,
    simulation::{LogLocation, Settlement, Worker},
};
use bevy::prelude::*;
pub const CENTER: Vec2 = Vec2::new(-11.0, -4.0);
pub const DOOR: Vec2 = Vec2::new(-11.0, -1.3);
pub const GATE: Vec2 = Vec2::new(-10.0, 3.5);
pub const TIMBER_COST: usize = 2;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Homestead {
    pub ordered: bool,
    pub complete: bool,
    pub invited: bool,
    pub arrived: bool,
    pub household: Household,
}
impl Default for Homestead {
    fn default() -> Self {
        let mut household = Household::default();
        household.id = 2;
        Self {
            ordered: false,
            complete: false,
            invited: false,
            arrived: false,
            household,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Job {
    Build,
    Grain,
    Fish,
    Fuel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    FetchGate,
    ToStore,
    Load,
    ToGate,
    ToDoor,
    Unload,
    ReturnGate,
    ReturnStore,
    Assemble,
}
impl Phase {
    pub fn walking(self) -> bool {
        matches!(
            self,
            Self::FetchGate
                | Self::ToStore
                | Self::ToGate
                | Self::ToDoor
                | Self::ReturnGate
                | Self::ReturnStore
        )
    }
}
pub fn blocks(p: Vec2) -> bool {
    (p.x - CENTER.x).abs() < 1.6 && (p.y - CENTER.y).abs() < 2.1
}
pub fn pantry(slot: usize) -> Vec2 {
    DOOR + Vec2::new((slot % 4) as f32 * 0.25 - 0.4, -0.25)
}
pub fn fuel(slot: usize) -> Vec3 {
    let p = DOOR
        + Vec2::new(
            0.65 + (slot % 3) as f32 * 0.25,
            -0.45 + (slot / 3) as f32 * 0.4,
        );
    Vec3::new(p.x, 0.43, p.y)
}
pub fn food(s: &Settlement, id: u32) -> usize {
    s.farm.count(GrainLocation::Home(id)) + s.fishery.count(FishLocation::Home(id))
}
impl Settlement {
    pub fn homestead_timbers(&self) -> usize {
        self.count(LogLocation::Homestead)
    }
    pub fn first_home_food_room(&self) -> bool {
        !self.homestead.arrived || food(self, 1) == 0
    }
    pub fn first_home_fuel_limit(&self) -> usize {
        if self.homestead.arrived && self.count(LogLocation::Home(2)) == 0 {
            1
        } else {
            crate::hearth::FUEL_TARGET
        }
    }
    pub fn order_homestead(&mut self) -> &'static str {
        if !self.homestead.ordered {
            self.homestead.ordered = true;
            return "Second farmstead ordered: needs two stored timber units and real building work. G opens its inspector.";
        }
        if !self.homestead.complete {
            return "The second farmstead still needs delivered timber and assembly. Harvest trees and leave timber in shared storage.";
        }
        if !self.homestead.invited {
            self.homestead.invited = true;
            return "A new resident is invited to household 2. Food and firewood must be carried from actual shared storage.";
        }
        "The second household already has an invitation. Its resident supplies that household only."
    }
    pub fn homestead_valid(&self) -> bool {
        let h = &self.homestead;
        h.household.id == 2
            && self.homestead_timbers() <= TIMBER_COST
            && (!h.complete || (h.ordered && self.homestead_timbers() == TIMBER_COST))
            && (!h.invited || h.complete)
            && (!h.arrived || h.invited)
            && h.household.fish_used <= h.household.food_used
            && h.household.next_use().is_finite()
            && h.household.hearth.next_use().is_finite()
    }
}
pub struct HomesteadPlugin;
impl Plugin for HomesteadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, sync);
    }
}
#[derive(Component)]
struct Part(usize);
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    let cube = meshes.add(crate::art::softened_box(Vec3::ONE, 0.035));
    let stone = materials.add(crate::coast::material(&server, "limestone"));
    let wood = materials.add(crate::coast::material(&server, "wood"));
    let roof = materials.add(StandardMaterial {
        base_color_texture: Some(crate::coast::texture(&server, "thatch-color", true)),
        normal_map_texture: Some(crate::coast::texture(&server, "thatch-normal", false)),
        double_sided: true,
        cull_mode: None,
        perceptual_roughness: 1.,
        ..default()
    });
    // Shared authored roof/materials, scaled for a modest outlying timber house.
    for (stage, offset, size, mat) in [
        (0, Vec3::new(0., 0.24, 0.), Vec3::new(2.8, 0.10, 3.8), stone),
        (
            1,
            Vec3::new(-1.25, 1.05, 0.),
            Vec3::new(0.15, 1.55, 3.5),
            wood.clone(),
        ),
        (
            2,
            Vec3::new(1.25, 1.05, 0.),
            Vec3::new(0.15, 1.55, 3.5),
            wood.clone(),
        ),
        (
            3,
            Vec3::new(0., 1.05, -1.7),
            Vec3::new(2.6, 1.55, 0.15),
            wood.clone(),
        ),
        (
            3,
            Vec3::new(-0.88, 1.05, 1.7),
            Vec3::new(0.8, 1.55, 0.15),
            wood.clone(),
        ),
        (
            3,
            Vec3::new(0.88, 1.05, 1.7),
            Vec3::new(0.8, 1.55, 0.15),
            wood.clone(),
        ),
        (
            3,
            Vec3::new(0., 1.65, 1.7),
            Vec3::new(0.95, 0.35, 0.15),
            wood.clone(),
        ),
    ] {
        commands.spawn((
            Part(stage),
            Name::new("Second farmstead construction"),
            Mesh3d(cube.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(CENTER.x + offset.x, offset.y, CENTER.y + offset.z)
                .with_scale(size),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        Part(3),
        Name::new("Second farmstead thatch"),
        Mesh3d(meshes.add(crate::art::thatch_roof())),
        MeshMaterial3d(roof),
        Transform::from_xyz(CENTER.x, 0., CENTER.y).with_scale(Vec3::new(0.78, 0.79, 0.72)),
        Visibility::Hidden,
    ));
    let mut gable = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-1.3, 0., 0.], [1.3, 0., 0.], [0., 0.94, 0.]],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.], [1., 0.], [0.5, 1.]]);
    gable.generate_tangents().expect("gable UVs");
    let gable = meshes.add(gable);
    for (z, rotation) in [(1.72, 0.), (-1.72, std::f32::consts::PI)] {
        commands.spawn((
            Part(3),
            Name::new("Second farmstead timber gable"),
            Mesh3d(gable.clone()),
            MeshMaterial3d(wood.clone()),
            Transform::from_xyz(CENTER.x, 1.825, CENTER.y + z)
                .with_rotation(Quat::from_rotation_y(rotation)),
            Visibility::Hidden,
        ));
    }
    for x in [-1.26, 1.26] {
        for z in [-1.72, 1.72] {
            commands.spawn((
                Part(1),
                Name::new("Second farmstead timber post"),
                Mesh3d(cube.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(CENTER.x + x, 1.06, CENTER.y + z)
                    .with_scale(Vec3::new(0.16, 1.58, 0.16)),
                Visibility::Hidden,
            ));
        }
    }
    let door = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.17, 0.10),
        perceptual_roughness: 0.96,
        ..crate::coast::material(&server, "wood")
    });
    commands.spawn((
        Part(3),
        Name::new("Second farmstead closed plank door"),
        Mesh3d(cube.clone()),
        MeshMaterial3d(door),
        Transform::from_xyz(CENTER.x, 0.85, CENTER.y + 1.79).with_scale(Vec3::new(0.88, 1.2, 0.07)),
        Visibility::Hidden,
    ));
    let shelf = DOOR + Vec2::new(-0.025, -0.25);
    commands.spawn((
        Part(3),
        Name::new("Household 2 pantry shelf"),
        Mesh3d(cube.clone()),
        MeshMaterial3d(wood.clone()),
        Transform::from_xyz(shelf.x, 0.26, shelf.y).with_scale(Vec3::new(1.35, 0.06, 0.7)),
        Visibility::Hidden,
    ));
    let cradle = DOOR + Vec2::new(0.9, -0.25);
    commands.spawn((
        Part(3),
        Name::new("Household 2 firewood cradle"),
        Mesh3d(cube.clone()),
        MeshMaterial3d(wood.clone()),
        Transform::from_xyz(cradle.x, 0.29, cradle.y).with_scale(Vec3::new(0.95, 0.18, 1.1)),
        Visibility::Hidden,
    ));
    // The reserved plot is visible before work; its fixed collision keeps a player
    // from standing inside future walls when construction completes.
    for x in [-1.5, 1.5] {
        for z in [-2., 2.] {
            commands.spawn((
                Name::new("Reserved farmstead plot peg"),
                Mesh3d(cube.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(CENTER.x + x, 0.35, CENTER.y + z)
                    .with_scale(Vec3::new(0.06, 0.3, 0.06)),
            ));
        }
    }
}
fn sync(s: Res<Settlement>, mut parts: Query<(&Part, &mut Visibility)>) {
    for (part, mut visible) in &mut parts {
        *visible = if match part.0 {
            0 => s.homestead.ordered,
            1 => s.homestead_timbers() >= 1,
            2 => s.homestead_timbers() >= TIMBER_COST,
            _ => s.homestead.complete,
        } {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
pub(crate) fn spawn_resident(mut commands: Commands, mut s: ResMut<Settlement>) {
    if s.homestead.invited && !s.homestead.arrived {
        let mut w = Worker::for_home(2);
        w.id = 3;
        w.position = DOOR;
        commands.spawn(w);
        s.homestead.arrived = true;
        s.homestead.household.supply_ordered = true;
        s.homestead.household.hearth.ordered = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{SimulationPlugin, Task, Worker};
    use bevy::time::TimeUpdateStrategy;
    fn built() -> (Settlement, Worker) {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.order_homestead();
        w.advance(20., &mut s);
        assert!(!s.homestead.complete && s.homestead_timbers() == 0);
        assert!(!s.homestead.invited);
        assert_eq!(s.count(LogLocation::InTree), crate::simulation::TOTAL_LOGS);
        for i in 0..3 {
            s.request_harvest(i);
        }
        for _ in 0..1200 {
            w.advance(0.25, &mut s);
            assert!(
                crate::world::walkable(w.position),
                "{:?} {:?}",
                w.task,
                w.position
            );
        }
        assert!(s.homestead.complete);
        assert!(s.homestead_valid());
        assert_eq!(s.homestead_timbers(), TIMBER_COST);
        assert_eq!(s.stored(), crate::simulation::TOTAL_LOGS - TIMBER_COST);
        assert_eq!(w.task, Task::Idle);
        (s, w)
    }
    #[test]
    fn real_construction_preserves_timber_and_clear_approaches() {
        let (mut s, mut w) = built();
        let before = s.logs.clone();
        s.order_homestead();
        assert!(s.homestead.invited);
        for _ in 0..10 {
            s.order_homestead();
            w.advance(1., &mut s);
        }
        assert_eq!(s.logs, before);
        assert!(!crate::world::walkable(CENTER));
        assert!(crate::world::walkable(DOOR));
        for (a, b) in [(crate::simulation::STOCKPILE, GATE), (GATE, DOOR)] {
            for i in 0..100 {
                assert!(crate::world::walkable(a.lerp(b, i as f32 / 99.)));
            }
        }
    }
    #[test]
    fn three_persistent_workers_supply_two_owned_households_without_duplicating_goods() {
        let (mut s, w) = built();
        s.order_homestead();
        s.request_helper();
        s.household.supply_ordered = true;
        s.household.hearth.ordered = true;
        s.farm.grain.fill(GrainLocation::Stored);
        s.fishery.fish.fill(FishLocation::PreservedStored);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(250),
            ))
            .insert_resource(Time::<Fixed>::from_seconds(0.25))
            .insert_resource(s)
            .init_resource::<crate::camera::CameraRig>()
            .add_plugins(SimulationPlugin);
        app.update();
        let founder = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .iter(app.world())
            .find(|&id| app.world().get::<Worker>(id).unwrap().id == 1)
            .unwrap();
        app.world_mut().entity_mut(founder).insert(w);
        app.update();
        let ids: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .iter(app.world())
            .collect();
        assert_eq!(ids.len(), 3);
        let mut saw_cargo = false;
        for frame in 0..1600 {
            if frame % 31 == 0 {
                app.world_mut()
                    .resource_mut::<crate::camera::CameraRig>()
                    .toggle();
            }
            app.update();
            let s = app.world().resource::<Settlement>();
            let workers: Vec<&Worker> = ids
                .iter()
                .map(|&id| app.world().get::<Worker>(id).unwrap())
                .collect();
            for worker in &workers {
                assert!(
                    crate::world::walkable(worker.position),
                    "{:?} {:?}",
                    worker.task,
                    worker.position
                );
            }
            let settler = workers.iter().find(|w| w.id == 3).unwrap();
            saw_cargo |= settler.carrying_grain.is_some() || settler.carrying_fish.is_some();
            assert_eq!(
                s.farm.count(GrainLocation::Carried),
                workers
                    .iter()
                    .filter(|w| w.carrying_grain.is_some())
                    .count()
            );
            assert_eq!(
                s.count(LogLocation::Carried),
                workers.iter().filter(|w| w.carrying.is_some()).count()
            );
            assert_eq!(
                s.fishery.count(FishLocation::CarriedPreserved),
                workers.iter().filter(|w| w.carrying_fish.is_some()).count()
            );
            assert!(s.household.food_used + s.homestead.household.food_used <= 8);
        }
        let s = app.world().resource::<Settlement>();
        assert!(saw_cargo);
        assert!(s.household.food_used > 0 && s.homestead.household.food_used > 0);
        assert_eq!(s.household.food_used + s.homestead.household.food_used, 8);
        assert!(s.count(LogLocation::Burned(1)) > 0 && s.count(LogLocation::Burned(2)) > 0);
        assert_eq!(
            s.homestead_timbers()
                + s.count(LogLocation::Burned(1))
                + s.count(LogLocation::Burned(2))
                + s.stored()
                + s.count(LogLocation::Home(1))
                + s.count(LogLocation::Home(2)),
            crate::simulation::TOTAL_LOGS
        );
    }
}
