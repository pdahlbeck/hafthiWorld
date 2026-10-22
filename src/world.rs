//! Seeded coastal scenery. Presentation never advances the settlement simulation.
use crate::{calendar::Season, performance::VisualQuality, simulation::Settlement};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Daylight>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (animate_meadow, animate_shore, daylight, seasonal_palette),
            );
    }
}

/// A deliberately slow presentation clock, unrelated to work durations.
#[derive(Resource, serde::Serialize, serde::Deserialize, Clone)]
pub struct Daylight {
    pub hour: f32,
    pub running: bool,
}
impl Default for Daylight {
    fn default() -> Self {
        Self {
            hour: 16.0,
            running: true,
        }
    }
}
#[derive(Component)]
struct Sun;
#[derive(Component)]
struct GrassPatch(f32);
#[derive(Component)]
struct ShoreRipple(f32);

// Small deterministic hash: identical scenery on every platform, no RNG dependency.
fn noise(index: u32) -> f32 {
    let mut n = index.wrapping_add(0x9e3779b9);
    n = (n ^ (n >> 16)).wrapping_mul(0x85ebca6b);
    n = (n ^ (n >> 13)).wrapping_mul(0xc2b2ae35);
    (n ^ (n >> 16)) as f32 / u32::MAX as f32
}

/// Flat playable meadow enclosed by an irregular limestone skirt. The existing
/// original clearing and authored worker paths stay on the level top at y=.2.
fn coast_point(i: usize, scale: f32) -> Vec3 {
    let angle = i as f32 * std::f32::consts::TAU / 64.0;
    let c = angle.cos();
    let s = angle.sin();
    let edge = 1.0 + 0.045 * (angle * 5.0).sin() + 0.025 * (angle * 11.0).cos();
    Vec3::new(
        c.signum() * c.abs().sqrt() * 11.2 * edge * scale,
        0.2,
        s.signum() * s.abs().sqrt() * 8.3 * edge * scale,
    )
}
fn fan_mesh(scale: f32, height: f32) -> Mesh {
    let mut points = Vec::with_capacity(64 * 3);
    for i in 0..64 {
        let a = coast_point(i, scale);
        let b = coast_point((i + 1) % 64, scale);
        points.extend([[0.0, height, 0.0], [b.x, height, b.z], [a.x, height, a.z]]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 64 * 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 64 * 3])
}
fn grass_mesh() -> Mesh {
    let mut points = Vec::new();
    // One mesh for a patch, shared by all patch entities; crossed tapered blades.
    for i in 0..28 {
        let x = (noise(i * 7) - 0.5) * 1.1;
        let z = (noise(i * 7 + 1) - 0.5) * 1.1;
        let h = 0.16 + noise(i * 7 + 2) * 0.26;
        points.extend([
            [x - 0.025, 0.0, z],
            [x + 0.025, 0.0, z],
            [x + 0.07, h, z],
            [x, 0.0, z - 0.025],
            [x, 0.0, z + 0.025],
            [x + 0.07, h, z],
        ]);
    }
    let count = points.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
}
fn ripple_mesh(scale: f32) -> Mesh {
    let mut points = Vec::new();
    for i in 0..64 {
        if noise(i as u32 + 500) < 0.35 {
            continue;
        }
        let a = coast_point(i, scale);
        let b = coast_point((i + 1) % 64, scale);
        let c = coast_point(i, scale + 0.005);
        let d = coast_point((i + 1) % 64, scale + 0.005);
        for p in [a, b, c, b, d, c] {
            points.push([p.x, 0.0, p.z]);
        }
    }
    let count = points.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
}
fn animate_shore(time: Res<Time>, mut ripples: Query<(&ShoreRipple, &mut Transform)>) {
    for (ripple, mut transform) in &mut ripples {
        let wave = (time.elapsed_secs() * 0.55 + ripple.0).sin();
        transform.scale = Vec3::new(1.0 + wave * 0.008, 1.0, 1.0 + wave * 0.008);
        transform.translation.y = -0.32 + wave * 0.002;
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    quality: Res<VisualQuality>,
) {
    spawn_coast(&mut commands, &mut meshes, &mut materials);
    spawn_farmstead(&mut commands, &mut meshes, &mut materials, *quality);
    spawn_pines(&mut commands, &mut meshes, &mut materials, *quality);
    spawn_grass(&mut commands, &mut meshes, &mut materials, *quality);
    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadow_maps_enabled: quality.shadows(),
            ..default()
        },
        Transform::default(),
        Sun,
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.73, 0.82, 0.94),
        brightness: 260.0,
        affects_lightmapped_meshes: true,
    });
}
fn spawn_coast(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let meadow = materials.add(Color::srgb(0.39, 0.49, 0.20));
    let limestone = materials.add(Color::srgb(0.72, 0.70, 0.56));
    let pale_stone = materials.add(Color::srgb(0.85, 0.82, 0.68));
    let sea = materials.add(StandardMaterial {
        base_color: Color::srgb(0.055, 0.24, 0.31),
        perceptual_roughness: 0.35,
        ..default()
    });
    let shallows = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.44, 0.45),
        perceptual_roughness: 0.42,
        ..default()
    });
    let nearshore = materials.add(Color::srgb(0.28, 0.54, 0.48));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1200.0, 1200.0))),
        MeshMaterial3d(sea),
        Transform::from_xyz(0.0, -0.38, 0.0),
    ));
    for (scale, height, material) in [
        (1.65, -0.36, shallows),
        (1.25, -0.34, nearshore),
        (1.07, -0.29, limestone.clone()),
        (1.0, 0.2, meadow),
    ] {
        let mut surface = commands.spawn((
            Mesh3d(meshes.add(fan_mesh(scale, height))),
            MeshMaterial3d(material),
            Transform::default(),
        ));
        if scale == 1.0 {
            surface.insert(SeasonalSurface::Meadow);
        }
    }
    let foam = materials.add(StandardMaterial {
        base_color: Color::srgb(0.46, 0.63, 0.57),
        cull_mode: None,
        perceptual_roughness: 0.6,
        ..default()
    });
    for i in 0..3 {
        commands.spawn((
            Mesh3d(meshes.add(ripple_mesh(1.12 + i as f32 * 0.16))),
            MeshMaterial3d(foam.clone()),
            Transform::from_xyz(0.0, -0.32, 0.0),
            ShoreRipple(i as f32 * 1.8),
            bevy::light::NotShadowCaster,
        ));
    }
    // Shared meshes/materials let Bevy batch repeated rocks and vegetation.
    let rock = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("valid icosphere"));
    for i in 0..64 {
        let p = coast_point(i, 1.01);
        commands.spawn((
            Mesh3d(rock.clone()),
            MeshMaterial3d(if i % 3 == 0 {
                pale_stone.clone()
            } else {
                limestone.clone()
            }),
            Transform::from_xyz(p.x, -0.10, p.z).with_scale(Vec3::new(
                0.65 + noise(i as u32),
                0.25 + noise(i as u32 + 80) * 0.2,
                0.5,
            )),
        ));
    }
    // Existing limestone obstacle retains its collision footprint.
    commands.spawn((
        Mesh3d(rock.clone()),
        MeshMaterial3d(pale_stone),
        Transform::from_xyz(-7.0, 0.4, 1.0).with_scale(Vec3::new(1.5, 0.45, 0.6)),
    ));
}

fn spawn_farmstead(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    quality: VisualQuality,
) {
    let timber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.27, 0.17, 0.10),
        perceptual_roughness: 0.95,
        ..default()
    });
    let warm_wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.28, 0.14),
        perceptual_roughness: 0.95,
        ..default()
    });
    let thatch = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.72, 0.44),
        cull_mode: None,
        perceptual_roughness: 1.0,
        ..default()
    });
    // A modest timber farm building, inside the old house collision footprint.
    let cube = meshes.add(crate::art::softened_box(Vec3::ONE, 0.08));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(timber.clone()),
        Transform::from_xyz(2.0, 1.25, 0.0).with_scale(Vec3::new(3.5, 2.1, 5.0)),
    ));
    // Rounded horizontal wall timbers have alternating warm tones and irregular ends.
    let log = meshes.add(Cylinder::new(0.13, 5.05).mesh().resolution(12));
    let end_log = meshes.add(Cylinder::new(0.13, 3.55).mesh().resolution(12));
    for row in 0..8 {
        let y = 0.38 + row as f32 * 0.25;
        let material = if row % 3 == 0 {
            timber.clone()
        } else {
            warm_wood.clone()
        };
        for x in [0.29, 3.71] {
            commands.spawn((
                Mesh3d(log.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(x, y, 0.)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ));
        }
        // The front wall leaves a real visual recess around the doorway.
        commands.spawn((
            Mesh3d(end_log.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(2., y, -2.49)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ));
        for x in [0.84, 3.16] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.13, 1.18).mesh().resolution(12))),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(x, y, 2.49)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ));
        }
    }
    let dark = materials.add(Color::srgb(0.08, 0.07, 0.045));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(dark),
        Transform::from_xyz(2.0, 0.99, 2.51).with_scale(Vec3::new(0.85, 1.58, 0.02)),
    ));
    // Continuous bowed thatch, an uneven eave fringe and a bound rounded ridge.
    commands.spawn((
        Mesh3d(meshes.add(crate::art::thatch_roof())),
        MeshMaterial3d(thatch.clone()),
        Transform::from_xyz(2., 0., 0.),
    ));
    let ridge: Vec<_> = (0..33)
        .map(|i| {
            let z = -2.88 + i as f32 * 0.18;
            Vec3::new(2., 3.52 - 0.07 * (z / 2.85).powi(2), z)
        })
        .collect();
    commands.spawn((
        Mesh3d(meshes.add(crate::art::tube(&ridge, 0.09))),
        MeshMaterial3d(thatch.clone()),
        Transform::default(),
    ));
    let fringe = meshes.add(
        Capsule3d::new(0.036, 0.19)
            .mesh()
            .longitudes(8)
            .latitudes(4),
    );
    let fringe_count = if quality == VisualQuality::High {
        70
    } else {
        35
    };
    for side in [-1., 1.] {
        for i in 0..fringe_count {
            let z = -2.78 + i as f32 * 5.56 / (fringe_count - 1) as f32;
            commands.spawn((
                Mesh3d(fringe.clone()),
                MeshMaterial3d(thatch.clone()),
                Transform::from_xyz(2. + side * 2.12, 2.23 - 0.04 * (z / 2.85).powi(2), z)
                    .with_rotation(Quat::from_rotation_z(side * 0.16))
                    .with_scale(Vec3::new(1., 0.7 + noise(i + 650) * 0.6, 1.)),
            ));
        }
    }
    let post = meshes.add(Cylinder::new(0.085, 1.7).mesh().resolution(12));
    for x in [1.52, 2.48] {
        commands.spawn((
            Mesh3d(post.clone()),
            MeshMaterial3d(warm_wood.clone()),
            Transform::from_xyz(x, 1.03, 2.57),
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(crate::art::softened_box(Vec3::new(1.15, 0.16, 0.20), 0.04))),
        MeshMaterial3d(warm_wood.clone()),
        Transform::from_xyz(2., 1.86, 2.57),
    ));
    let gable = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-1.75, 0.0, 0.0], [1.75, 0.0, 0.0], [0.0, 1.12, 0.0]],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    let gable = meshes.add(gable);
    for (z, angle) in [(2.5, 0.0), (-2.5, std::f32::consts::PI)] {
        commands.spawn((
            Mesh3d(gable.clone()),
            MeshMaterial3d(warm_wood.clone()),
            Transform::from_xyz(2.0, 2.3, z).with_rotation(Quat::from_rotation_y(angle)),
        ));
    }
}

fn spawn_pines(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    quality: VisualQuality,
) {
    let timber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.27, 0.17, 0.10),
        perceptual_roughness: 0.95,
        ..default()
    });
    // Shared pine positions also define trunk clearance for first-person walking.
    let foliage = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.29, 0.16),
        perceptual_roughness: 1.0,
        ..default()
    });
    let pine = meshes.add(crate::art::foliage());
    let trunk = meshes.add(Cylinder::new(0.13, 2.4));
    for i in 0..9 {
        let Vec2 { x, y: z } = pine_position(i);
        commands.spawn((
            Mesh3d(trunk.clone()),
            MeshMaterial3d(timber.clone()),
            Transform::from_xyz(x, 1.2, z),
        ));
        for layer in 0..5 {
            let h = 1.20 + layer as f32 * 0.48;
            let width = (0.95 - layer as f32 * 0.145) * (0.90 + noise(i + 300) * 0.2);
            let branches = if quality == VisualQuality::High { 5 } else { 3 };
            for bough in 0..branches {
                let angle = bough as f32 * std::f32::consts::TAU / branches as f32
                    + layer as f32 * 0.71
                    + i as f32;
                let offset = Vec2::new(angle.cos(), angle.sin()) * width * 0.40;
                commands.spawn((
                    Mesh3d(pine.clone()),
                    MeshMaterial3d(foliage.clone()),
                    Transform::from_xyz(
                        x + offset.x,
                        h + noise(i * 31 + layer * 5 + bough) * 0.16,
                        z + offset.y,
                    )
                    .with_rotation(Quat::from_rotation_y(angle))
                    .with_scale(Vec3::new(width * 0.72, 0.34, width * 0.52)),
                ));
            }
        }
    }
}

fn spawn_grass(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    quality: VisualQuality,
) {
    let grass = meshes.add(grass_mesh());
    let grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.47, 0.53, 0.23),
        cull_mode: None,
        perceptual_roughness: 1.0,
        ..default()
    });
    for i in 0..quality.grass_patches() {
        let x = (noise(i * 3) - 0.5) * 16.8;
        let z = (noise(i * 3 + 1) - 0.5) * 11.8;
        // Keep house, work sites and the south stockpile approach visually clear.
        let field = crate::farming::FIELD_CENTER;
        if !walkable(Vec2::new(x, z))
            || (x < -1.0 && z > 0.0)
            || ((x - field.x).abs() < 2.1 && (z - field.y).abs() < 1.5)
        {
            continue;
        }
        commands.spawn((
            Mesh3d(grass.clone()),
            MeshMaterial3d(grass_mat.clone()),
            Transform::from_xyz(x, 0.21, z),
            GrassPatch(noise(i * 3 + 2) * 6.0),
            SeasonalSurface::Grass,
            bevy::light::NotShadowCaster,
        ));
    }
}

fn animate_meadow(time: Res<Time>, mut patches: Query<(&GrassPatch, &mut Transform)>) {
    for (patch, mut transform) in &mut patches {
        transform.rotation =
            Quat::from_rotation_z((time.elapsed_secs() * 1.7 + patch.0).sin() * 0.07);
    }
}
/// Returns a sun direction and brightness with a readable moonlit night floor.
fn sun_at(hour: f32) -> (Quat, f32) {
    let elevation = ((hour - 6.0) / 24.0 * std::f32::consts::TAU).sin();
    (
        Quat::from_euler(
            EulerRot::XYZ,
            -elevation.max(0.08) * 1.1,
            -0.65 + (hour - 16.0) * 0.08,
            -0.4,
        ),
        700.0 + elevation.max(0.0) * 13_000.0,
    )
}
fn daylight(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut day: ResMut<Daylight>,
    mut suns: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sky: ResMut<ClearColor>,
    mut fog: Query<&mut DistanceFog>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        day.running = !day.running;
    }
    if keys.just_pressed(KeyCode::KeyY) {
        day.hour = (day.hour + 3.0).rem_euclid(24.0);
    }
    if day.running {
        day.hour = (day.hour + time.delta_secs() / 30.0).rem_euclid(24.0);
    }
    let elevation = ((day.hour - 6.0) / 24.0 * std::f32::consts::TAU).sin();
    let daylight = ((elevation + 0.15) / 0.4).clamp(0.0, 1.0);
    let sky_rgb = Vec3::new(0.055, 0.085, 0.14).lerp(Vec3::new(0.64, 0.75, 0.76), daylight);
    sky.0 = Color::srgb(sky_rgb.x, sky_rgb.y, sky_rgb.z);
    for mut fog in &mut fog {
        fog.color = sky.0;
    }
    ambient.brightness = 60.0 + daylight * 200.0;
    let (rotation, illuminance) = sun_at(day.hour);
    for (mut transform, mut light) in &mut suns {
        transform.rotation = rotation;
        light.illuminance = illuminance;
        light.color = if (6.0..9.0).contains(&day.hour) || (16.0..20.0).contains(&day.hour) {
            Color::srgb(1.0, 0.80, 0.59)
        } else {
            Color::srgb(0.91, 0.94, 1.0)
        };
    }
}

// The movement shore follows the actual mesh, with a small dry-land setback.
fn inside_shore(p: Vec2) -> bool {
    let mut inside = false;
    for i in 0..64 {
        let a = coast_point(i, 0.94);
        let b = coast_point((i + 1) % 64, 0.94);
        if (a.z > p.y) != (b.z > p.y) && p.x < (b.x - a.x) * (p.y - a.z) / (b.z - a.z) + a.x {
            inside = !inside;
        }
    }
    inside
}
fn pine_position(i: u32) -> Vec2 {
    Vec2::new(
        -7.7 + i as f32 * 1.7 + (noise(i + 200) - 0.5) * 0.8,
        -6.65 - noise(i + 201) * 0.65,
    )
}
pub fn walkable(p: Vec2) -> bool {
    inside_shore(p)
        && !crate::livestock::fence_blocks(p)
        && !(p.x > -0.10 && p.x < 4.10 && p.y.abs() < 2.85)
        && !(p.x > -8.85 && p.x < -5.15 && p.y > 0.05 && p.y < 1.95)
        && !(0..9).any(|i| p.distance_squared(pine_position(i)) < 0.35_f32.powi(2))
}
pub fn slide(position: Vec2, displacement: Vec2) -> Vec2 {
    let mut p = position;
    // Substeps prevent crossing a footprint or shore after a long frame.
    let steps = (displacement.length() / 0.10).ceil().max(1.0) as u32;
    let step = displacement / steps as f32;
    for _ in 0..steps {
        let next = p + Vec2::new(step.x, 0.0);
        if walkable(next) {
            p = next;
        }
        let next = p + Vec2::new(0.0, step.y);
        if walkable(next) {
            p = next;
        }
    }
    p
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coast_and_pine_collision_match_rendered_scenery() {
        assert!(inside_shore(Vec2::ZERO));
        for i in 0..64 {
            let outer = coast_point(i, 1.08);
            assert!(!walkable(Vec2::new(outer.x, outer.z)));
        }
        for i in 0..9 {
            assert!(!walkable(pine_position(i)));
        }
        for p in crate::simulation::TREE_POSITIONS {
            assert!(walkable(p));
        }
        assert!(walkable(crate::simulation::STOCKPILE));
    }
    #[test]
    fn sunlight_is_finite_readable_and_brighter_at_noon() {
        for hour in 0..24 {
            let (rotation, brightness) = sun_at(hour as f32);
            assert!(rotation.is_finite() && brightness >= 700.0);
        }
        assert!(sun_at(12.0).1 > sun_at(0.0).1 * 10.0);
    }
    #[test]
    fn visitor_cannot_cross_house_rock_or_shore_even_on_long_frames() {
        for (start, movement) in [
            (Vec2::new(-2.0, 0.0), Vec2::new(10.0, 0.0)),
            (Vec2::new(-7.0, -2.0), Vec2::new(0.0, 8.0)),
            (Vec2::new(-3.0, 4.0), Vec2::new(-20.0, 0.0)),
        ] {
            let end = slide(start, movement);
            assert!(walkable(end));
            assert!(end.distance(start) < movement.length());
        }
    }
}

#[derive(Component)]
enum SeasonalSurface {
    Meadow,
    Grass,
}
fn seasonal_palette(
    settlement: Option<Res<Settlement>>,
    surfaces: Query<(&SeasonalSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last: Local<Option<Season>>,
) {
    let Some(settlement) = settlement else {
        return;
    };
    let season = settlement.calendar.season();
    if *last == Some(season) {
        return;
    }
    let (meadow, grass) = match season {
        Season::Spring => (Color::srgb(0.39, 0.49, 0.20), Color::srgb(0.47, 0.53, 0.23)),
        Season::Summer => (Color::srgb(0.44, 0.48, 0.20), Color::srgb(0.56, 0.54, 0.25)),
        Season::Autumn => (Color::srgb(0.47, 0.38, 0.21), Color::srgb(0.63, 0.46, 0.25)),
        Season::Winter => (Color::srgb(0.42, 0.44, 0.36), Color::srgb(0.52, 0.51, 0.40)),
    };
    for (surface, handle) in &surfaces {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.base_color = match surface {
                SeasonalSurface::Meadow => meadow,
                SeasonalSurface::Grass => grass,
            };
        }
    }
    *last = Some(season);
}

#[cfg(test)]
mod seasonal_tests {
    use super::*;
    #[test]
    fn seasonal_colors_reuse_existing_meshes_and_materials() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Settlement>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Update, seasonal_palette);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(Color::WHITE);
        let entity = app
            .world_mut()
            .spawn((
                SeasonalSurface::Meadow,
                MeshMaterial3d(handle.clone()),
                Transform::from_xyz(1.0, 2.0, 3.0),
            ))
            .id();
        app.update();
        let spring = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(&handle)
            .unwrap()
            .base_color;
        app.world_mut()
            .resource_mut::<Settlement>()
            .calendar
            .advance(360.0);
        app.update();
        let winter = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(&handle)
            .unwrap()
            .base_color;
        assert_ne!(spring, winter);
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .unwrap()
                .0,
            handle
        );
        assert_eq!(
            app.world().get::<Transform>(entity).unwrap().translation,
            Vec3::new(1.0, 2.0, 3.0)
        );
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
        app.world_mut()
            .resource_mut::<Settlement>()
            .calendar
            .advance(120.0);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&handle)
                .unwrap()
                .base_color,
            spring
        );
    }
}
