//! One outbound collection of the shore visitor's already-paid wool.
use crate::{
    livestock::{WOOL_UNITS, WoolLocation},
    simulation::Settlement,
    trade::SaltLocation,
};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

pub const LANDING_WORK: Vec2 = Vec2::new(-7.0, 6.6);
pub const BERTH: Vec2 = Vec2::new(-8.0, 9.1);
pub const OFFSHORE: Vec2 = Vec2::new(-18.0, 13.0);
const SAIL_SECONDS: f64 = 20.0;
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum VoyagePhase {
    #[default]
    NotOrdered,
    WaitingForTrade,
    Arriving,
    Loading,
    Departing,
    Completed,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct Voyage {
    pub phase: VoyagePhase,
    elapsed: f64,
}
impl Voyage {
    pub fn request(&mut self) -> bool {
        if self.phase != VoyagePhase::NotOrdered {
            return false;
        }
        self.phase = VoyagePhase::WaitingForTrade;
        true
    }
    pub fn position(&self) -> Vec2 {
        let progress = (self.elapsed / SAIL_SECONDS).clamp(0.0, 1.0) as f32;
        match self.phase {
            VoyagePhase::Arriving => OFFSHORE.lerp(BERTH, progress),
            VoyagePhase::Departing => BERTH.lerp(OFFSHORE, progress),
            _ => BERTH,
        }
    }
    /// Presentation is derived from saved simulation time, never render time.
    fn sail_fraction(&self) -> f32 {
        match self.phase {
            VoyagePhase::Arriving => ((SAIL_SECONDS - self.elapsed) / 3.0).clamp(0.0, 1.0) as f32,
            VoyagePhase::Departing => (self.elapsed / 3.0).clamp(0.0, 1.0) as f32,
            _ => 0.0,
        }
    }
    pub fn valid(&self, wool: &[WoolLocation]) -> bool {
        let aboard = wool.iter().filter(|&&w| w == WoolLocation::Aboard).count();
        let exported = wool
            .iter()
            .filter(|&&w| w == WoolLocation::Exported)
            .count();
        self.elapsed.is_finite()
            && (0.0..SAIL_SECONDS).contains(&self.elapsed)
            && (aboard == 0 || matches!(self.phase, VoyagePhase::Loading | VoyagePhase::Departing))
            && (self.phase != VoyagePhase::Departing || aboard == WOOL_UNITS)
            && (if self.phase == VoyagePhase::Completed {
                exported == WOOL_UNITS
            } else {
                exported == 0
            })
    }
    pub fn visible(&self) -> bool {
        matches!(
            self.phase,
            VoyagePhase::Arriving | VoyagePhase::Loading | VoyagePhase::Departing
        )
    }
    pub fn label(&self) -> &'static str {
        match self.phase {
            VoyagePhase::NotOrdered => "U: request wool collection",
            VoyagePhase::WaitingForTrade => "waiting for both salt trades and returns",
            VoyagePhase::Arriving => "approaching landing",
            VoyagePhase::Loading => "loading paid wool from visitor's pile",
            VoyagePhase::Departing => "departing with wool",
            VoyagePhase::Completed => "shipment completed",
        }
    }
    pub fn advance(
        &mut self,
        mut seconds: f64,
        wool: &mut [WoolLocation; WOOL_UNITS],
        salt: &[SaltLocation],
    ) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        loop {
            match self.phase {
                VoyagePhase::WaitingForTrade => {
                    if !wool.iter().all(|&w| w == WoolLocation::Trader)
                        || salt.iter().any(|s| {
                            matches!(
                                s,
                                SaltLocation::Offered | SaltLocation::Paid | SaltLocation::Carried
                            )
                        })
                    {
                        return;
                    }
                    self.phase = VoyagePhase::Arriving;
                }
                VoyagePhase::Loading => {
                    if !wool.iter().all(|&w| w == WoolLocation::Aboard) {
                        return;
                    }
                    self.phase = VoyagePhase::Departing;
                    self.elapsed = 0.0;
                }
                VoyagePhase::Arriving | VoyagePhase::Departing => {
                    let used = seconds.min(SAIL_SECONDS - self.elapsed);
                    self.elapsed += used;
                    seconds -= used;
                    if self.elapsed < SAIL_SECONDS {
                        return;
                    }
                    self.elapsed = 0.0;
                    if self.phase == VoyagePhase::Arriving {
                        self.phase = VoyagePhase::Loading;
                    } else {
                        wool.fill(WoolLocation::Exported);
                        self.phase = VoyagePhase::Completed;
                        return;
                    }
                }
                _ => return,
            }
        }
    }
}
pub fn cargo_position(voyage: &Voyage, index: usize) -> Vec3 {
    let offset = Vec2::new(
        (index % 2) as f32 * 0.36 - 0.18,
        (index / 2) as f32 * 0.42 - 0.2,
    );
    let p = voyage.position() + Vec2::new(offset.y, -offset.x);
    Vec3::new(p.x, 0.15, p.y)
}
pub struct ShippingPlugin;
impl Plugin for ShippingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (sync, sync_rigging));
    }
}
#[derive(Component)]
struct Boat;
#[derive(Component)]
struct ShippingStatus;
#[derive(Component)]
struct RaisedSail;
#[derive(Component)]
struct FurledSail;
#[derive(Component)]
struct BoatCrew;

fn sail_mesh() -> Mesh {
    let mut vertices = Vec::new();
    for x in 0..8 {
        for y in 0..6 {
            for (u, v) in [(0., 0.), (1., 0.), (1., 1.), (0., 0.), (1., 1.), (0., 1.)] {
                let u = (x as f32 + u) / 8.0;
                let v = (y as f32 + v) / 6.0;
                vertices.push([
                    (u - 0.5) * (1.55 + 0.16 * v),
                    -1.15 * v,
                    0.18 * (u * std::f32::consts::PI).sin() * (v * std::f32::consts::PI).sin(),
                ]);
            }
        }
    }
    let count = vertices.len();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count]);
    mesh.compute_flat_normals();
    mesh
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let timber = materials.add(Color::srgb(0.35, 0.19, 0.085));
    let edge = materials.add(Color::srgb(0.58, 0.35, 0.14));
    let cloth = materials.add(Color::srgb(0.78, 0.67, 0.46));
    let sail_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.84, 0.72, 0.49),
        cull_mode: None,
        perceptual_roughness: 0.95,
        ..default()
    });
    let sail = meshes.add(sail_mesh());
    let skin = materials.add(Color::srgb(0.67, 0.45, 0.31));
    let rust_tunic = materials.add(Color::srgb(0.58, 0.25, 0.13));
    let blue_tunic = materials.add(Color::srgb(0.22, 0.36, 0.43));
    let head = meshes.add(Sphere::new(0.14));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    // Original tapered clinker-inspired hull: visible ribs/plank strakes, no copied vessel model.
    commands
        .spawn((
            Boat,
            Name::new("Coastal wool collection boat"),
            Transform::default(),
            Visibility::Hidden,
        ))
        .with_children(|boat| {
            for i in 0..13 {
                let z = -1.9 + i as f32 * (3.8 / 12.0);
                let width = 0.12 + 0.65 * (1.0 - (z / 2.05).powi(2)).max(0.0);
                boat.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(timber.clone()),
                    Transform::from_xyz(0.0, 0.14, z).with_scale(Vec3::new(
                        width * 1.7,
                        0.14,
                        0.33,
                    )),
                ));
                for side in [-1.0, 1.0] {
                    for plank in 0..3 {
                        boat.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(if plank == 2 {
                                edge.clone()
                            } else {
                                timber.clone()
                            }),
                            Transform::from_xyz(
                                side * (width - 0.10 + plank as f32 * 0.06),
                                0.02 + plank as f32 * 0.13,
                                z,
                            )
                            .with_scale(Vec3::new(0.10, 0.14, 0.34)),
                        ));
                    }
                }
            }
            for z in [-1.92, 1.92] {
                boat.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(edge.clone()),
                    Transform::from_xyz(0.0, 0.25, z).with_scale(Vec3::new(0.13, 0.85, 0.13)),
                ));
            }
            boat.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(0.0, 1.2, 0.55).with_scale(Vec3::new(0.10, 2.4, 0.10)),
            ));
            boat.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_xyz(0.0, 2.15, 0.55).with_scale(Vec3::new(1.65, 0.08, 0.08)),
            ));
            // The two visiting sailors stay aboard, separate from household workers.
            for (index, z) in [-1.25, 1.35].into_iter().enumerate() {
                boat.spawn((
                    BoatCrew,
                    Name::new(format!("Visiting boat crew {}", index + 1)),
                    Transform::from_xyz(0.0, 0.21, z),
                    Visibility::Inherited,
                ))
                .with_children(|crew| {
                    let tunic = if index == 0 {
                        rust_tunic.clone()
                    } else {
                        blue_tunic.clone()
                    };
                    crew.spawn((
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(tunic.clone()),
                        Transform::from_xyz(0.0, 0.36, 0.0).with_scale(Vec3::new(0.30, 0.48, 0.23)),
                    ));
                    crew.spawn((
                        Mesh3d(head.clone()),
                        MeshMaterial3d(skin.clone()),
                        Transform::from_xyz(0.0, 0.74, 0.0),
                    ));
                    for x in [-0.09, 0.09] {
                        crew.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(timber.clone()),
                            Transform::from_xyz(x, 0.09, 0.0)
                                .with_scale(Vec3::new(0.09, 0.18, 0.12)),
                        ));
                        crew.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(tunic.clone()),
                            Transform::from_xyz(x * 2.0, 0.36, 0.07)
                                .with_rotation(Quat::from_rotation_x(-0.45))
                                .with_scale(Vec3::new(0.08, 0.34, 0.08)),
                        ));
                    }
                });
            }
            // A visible steering oar belongs to the visiting helmsman.
            boat.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_xyz(0.49, 0.27, -1.25)
                    .with_rotation(Quat::from_rotation_z(0.55))
                    .with_scale(Vec3::new(0.07, 1.25, 0.07)),
            ));
            boat.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_xyz(0.76, -0.22, -1.25)
                    .with_rotation(Quat::from_rotation_z(0.55))
                    .with_scale(Vec3::new(0.18, 0.38, 0.06)),
            ));
            boat.spawn((
                RaisedSail,
                Name::new("Boat square sail"),
                Mesh3d(sail.clone()),
                MeshMaterial3d(sail_material.clone()),
                Transform::from_xyz(0.0, 2.06, 0.55),
                Visibility::Hidden,
            ));
            // Sail geometry has its origin at the yard so reefing keeps its top attached.
            boat.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(cloth),
                FurledSail,
                Visibility::Inherited,
                Transform::from_xyz(0.0, 2.02, 0.55).with_scale(Vec3::new(1.55, 0.20, 0.14)),
            ));
        });
    // A small existing shore handover plank, not a player-built/costed harbour.
    commands.spawn((
        Name::new("Shore handover plank"),
        Mesh3d(cube.clone()),
        MeshMaterial3d(timber),
        Transform::from_xyz(-7.0, 0.20, 7.6).with_scale(Vec3::new(0.65, 0.12, 1.8)),
    ));
    for x in [-7.4, -6.6] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(edge.clone()),
            Transform::from_xyz(x, 0.40, 7.3).with_scale(Vec3::new(0.12, 0.8, 0.12)),
        ));
    }
    commands.spawn((
        ShippingStatus,
        Text::new(""),
        TextFont {
            font_size: px(16).into(),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.06, 0.10, 0.10, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            bottom: px(125),
            padding: UiRect::all(px(8)),
            ..default()
        },
    ));
}
fn sync(
    s: Res<Settlement>,
    mut boats: Query<(&mut Transform, &mut Visibility), With<Boat>>,
    mut status: Query<&mut Text, With<ShippingStatus>>,
) {
    for (mut t, mut visible) in &mut boats {
        let p = s.voyage.position();
        t.translation = Vec3::new(p.x, -0.24, p.y);
        t.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        *visible = if s.voyage.visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text in &mut status {
        text.0 = format!(
            "Boat: {}\nWool: aboard {}/4 | exported {}/4",
            s.voyage.label(),
            s.flock.count(WoolLocation::Aboard),
            s.flock.count(WoolLocation::Exported)
        );
    }
}

type RaisedSailQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<RaisedSail>, Without<FurledSail>),
>;
type FurledSailQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<FurledSail>, Without<RaisedSail>),
>;
fn sync_rigging(s: Res<Settlement>, mut sails: RaisedSailQuery, mut furled: FurledSailQuery) {
    let fraction = s.voyage.sail_fraction();
    for (mut t, mut visible) in &mut sails {
        t.scale.y = fraction.max(0.001);
        *visible = if fraction > 0.001 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (mut t, mut visible) in &mut furled {
        t.scale.y = 0.20 * (1.0 - fraction).max(0.001);
        *visible = if fraction < 0.999 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        orders::OrderState,
        simulation::{SimulationPlugin, Worker},
    };
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;
    #[test]
    fn rigging_restores_sailing_pose_and_keeps_two_visitors_on_the_same_boat() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((SimulationPlugin, ShippingPlugin));
        app.world_mut()
            .resource_mut::<crate::session::Session>()
            .paused = true;
        app.update();
        let boat = app
            .world_mut()
            .query_filtered::<Entity, With<Boat>>()
            .single(app.world())
            .unwrap();
        let sail = app
            .world_mut()
            .query_filtered::<Entity, With<RaisedSail>>()
            .single(app.world())
            .unwrap();
        let furled = app
            .world_mut()
            .query_filtered::<Entity, With<FurledSail>>()
            .single(app.world())
            .unwrap();
        let crew: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<BoatCrew>>()
            .iter(app.world())
            .collect();
        assert_eq!(crew.len(), 2);
        let meshes: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<Mesh3d>>()
            .iter(app.world())
            .collect();
        let clock = app
            .world()
            .resource::<crate::simulation::SimulationClock>()
            .0;
        for (phase, elapsed, fraction) in [
            (VoyagePhase::NotOrdered, 0., 0.),
            (VoyagePhase::WaitingForTrade, 0., 0.),
            (VoyagePhase::Arriving, 0., 1.),
            (VoyagePhase::Arriving, 18.5, 0.5),
            (VoyagePhase::Loading, 0., 0.),
            (VoyagePhase::Departing, 0., 0.),
            (VoyagePhase::Departing, 1.5, 0.5),
            (VoyagePhase::Departing, 3., 1.),
            (VoyagePhase::Departing, 19.5, 1.),
            (VoyagePhase::Completed, 0., 0.),
        ] {
            // The same serialized voyage used by saves reconstructs the visual pose.
            let saved = serde_json::to_vec(&Voyage { phase, elapsed }).unwrap();
            app.world_mut().resource_mut::<Settlement>().voyage =
                serde_json::from_slice(&saved).unwrap();
            let before = serde_json::to_vec(app.world().resource::<Settlement>()).unwrap();
            for _ in 0..3 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
                app.update();
            }
            assert_eq!(
                serde_json::to_vec(app.world().resource::<Settlement>()).unwrap(),
                before
            );
            assert_eq!(
                app.world()
                    .resource::<crate::simulation::SimulationClock>()
                    .0,
                clock
            );
            assert_eq!(
                app.world().get::<Transform>(sail).unwrap().scale.y,
                f32::max(fraction, 0.001)
            );
            assert_eq!(
                *app.world().get::<Visibility>(sail).unwrap() != Visibility::Hidden,
                fraction > 0.0
            );
            assert_eq!(
                *app.world().get::<Visibility>(furled).unwrap() != Visibility::Hidden,
                fraction < 1.0
            );
            assert_eq!(
                *app.world().get::<Visibility>(boat).unwrap() != Visibility::Hidden,
                matches!(
                    phase,
                    VoyagePhase::Arriving | VoyagePhase::Loading | VoyagePhase::Departing
                )
            );
            for &resident in &crew {
                assert_eq!(app.world().get::<ChildOf>(resident).unwrap().parent(), boat);
            }
            assert!(
                meshes
                    .iter()
                    .all(|&id| app.world().get::<Mesh3d>(id).is_some())
            );
            assert_eq!(
                app.world_mut().query::<&Worker>().iter(app.world()).count(),
                1
            );
        }
    }
    #[test]
    fn request_waits_for_completed_real_trade_and_full_load_before_one_departure() {
        let mut voyage = Voyage::default();
        let mut wool = [WoolLocation::Stored; WOOL_UNITS];
        let mut salt = [SaltLocation::Offered; 2];
        assert!(voyage.request());
        assert!(!voyage.request());
        voyage.advance(1000.0, &mut wool, &salt);
        assert_eq!(voyage.phase, VoyagePhase::WaitingForTrade);
        wool.fill(WoolLocation::Trader);
        salt.fill(SaltLocation::Paid);
        voyage.advance(1000.0, &mut wool, &salt);
        assert_eq!(voyage.phase, VoyagePhase::WaitingForTrade);
        salt.fill(SaltLocation::Stored);
        let mut partitioned = voyage.clone();
        let mut split_wool = wool;
        voyage.advance(20.0, &mut wool, &salt);
        for _ in 0..80 {
            partitioned.advance(0.25, &mut split_wool, &salt);
        }
        assert_eq!(voyage.phase, VoyagePhase::Loading);
        assert_eq!(voyage.position(), partitioned.position());
        wool[0] = WoolLocation::Aboard;
        voyage.advance(1000.0, &mut wool, &salt);
        assert_eq!(voyage.phase, VoyagePhase::Loading);
        wool.fill(WoolLocation::Aboard);
        voyage.advance(10.0, &mut wool, &salt);
        assert_eq!(voyage.phase, VoyagePhase::Departing);
        assert!(voyage.position().distance(BERTH) > 0.0);
        voyage.advance(10.0, &mut wool, &salt);
        assert_eq!(voyage.phase, VoyagePhase::Completed);
        assert_eq!(wool, [WoolLocation::Exported; 4]);
        assert!(!voyage.visible());
        assert!(!voyage.request());
        voyage.advance(1000.0, &mut wool, &salt);
        assert_eq!(wool, [WoolLocation::Exported; 4]);
    }
    #[test]
    fn actual_solo_and_two_resident_trade_exports_the_same_four_original_bundles() {
        for helper in [false, true] {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                    100,
                )))
                .insert_resource(Time::<Fixed>::from_seconds(0.1))
                .init_resource::<Assets<Mesh>>()
                .init_resource::<Assets<StandardMaterial>>()
                .init_resource::<CameraRig>()
                .init_resource::<OrderState>()
                .add_plugins((
                    SimulationPlugin,
                    crate::livestock::LivestockPlugin,
                    crate::trade::TradePlugin,
                    crate::fishing::FishingPlugin,
                    ShippingPlugin,
                ));
            app.update();
            let ids: Vec<_> = app
                .world_mut()
                .query_filtered::<Entity, With<Mesh3d>>()
                .iter(app.world())
                .collect();
            {
                let mut s = app.world_mut().resource_mut::<Settlement>();
                if helper {
                    s.request_helper();
                }
                s.flock.request_shear();
                s.barter.request();
                s.voyage.request();
                s.fishery.request_fishing();
                s.fishery.request_salting();
                s.household.request_supply();
            }
            let mut paused_phases = Vec::new();
            let mut saw_cargo = false;
            let mut saw_aboard = false;
            let mut saw_departure = false;
            for tick in 0..7500 {
                if tick % 11 == 0 {
                    app.world_mut().resource_mut::<CameraRig>().toggle();
                }
                app.update();
                let phase = app.world().resource::<Settlement>().voyage.phase;
                if matches!(phase, VoyagePhase::Arriving | VoyagePhase::Departing)
                    && !paused_phases.contains(&phase)
                {
                    let before = serde_json::to_vec(app.world().resource::<Settlement>()).unwrap();
                    let clock = app
                        .world()
                        .resource::<crate::simulation::SimulationClock>()
                        .0;
                    app.world_mut()
                        .resource_mut::<crate::session::Session>()
                        .paused = true;
                    for _ in 0..10 {
                        app.world_mut().resource_mut::<CameraRig>().toggle();
                        app.update();
                    }
                    assert_eq!(
                        serde_json::to_vec(app.world().resource::<Settlement>()).unwrap(),
                        before
                    );
                    assert_eq!(
                        app.world()
                            .resource::<crate::simulation::SimulationClock>()
                            .0,
                        clock
                    );
                    app.world_mut()
                        .resource_mut::<crate::session::Session>()
                        .paused = false;
                    paused_phases.push(phase);
                }
                let s = app.world().resource::<Settlement>();
                let workers: Vec<_> = app
                    .world()
                    .iter_entities()
                    .filter_map(|e| e.get::<Worker>())
                    .collect();
                assert_eq!(
                    s.flock.count(WoolLocation::Carried),
                    workers.iter().filter(|w| w.carrying_wool.is_some()).count()
                );
                for worker in &workers {
                    assert!(
                        crate::world::walkable(worker.position),
                        "{:?} at {:?}",
                        worker.task,
                        worker.position
                    );
                }
                for e in app.world().iter_entities() {
                    let Some(name) = e.get::<Name>() else {
                        continue;
                    };
                    let Some(slot) = name
                        .as_str()
                        .strip_prefix("Wool ")
                        .and_then(|n| n.parse::<usize>().ok())
                    else {
                        continue;
                    };
                    let actual = e.get::<Transform>().unwrap().translation;
                    if s.flock.wool[slot] == WoolLocation::Aboard {
                        assert_eq!(actual, cargo_position(&s.voyage, slot));
                        saw_aboard = true;
                    }
                    if let Some(worker) = workers.iter().find(|w| w.carrying_wool == Some(slot)) {
                        assert_eq!(
                            actual,
                            Vec3::new(worker.position.x + 0.3, 1.0, worker.position.y)
                        );
                        if matches!(
                            worker.task,
                            crate::simulation::Task::Coastal(
                                crate::fishing::CoastalTask::ToBoat
                                    | crate::fishing::CoastalTask::UnloadingExportWool
                            )
                        ) {
                            saw_cargo = true;
                        }
                    }
                }
                if s.voyage.phase == VoyagePhase::Departing {
                    saw_departure = true;
                    assert_eq!(s.flock.count(WoolLocation::Aboard), 4);
                }
            }
            let s = app.world().resource::<Settlement>();
            assert_eq!(s.voyage.phase, VoyagePhase::Completed);
            assert_eq!(s.flock.wool, [WoolLocation::Exported; 4]);
            assert_eq!(s.household.fish_used, 4);
            assert_eq!(s.barter.count(SaltLocation::Consumed), 2);
            assert!(saw_cargo && saw_aboard && saw_departure);
            assert_eq!(paused_phases.len(), 2);
            assert!(
                ids.iter()
                    .all(|&id| app.world().get::<Mesh3d>(id).is_some())
            );
        }
    }
}
