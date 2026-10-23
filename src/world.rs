//! Seeded coastal scenery. Presentation never advances the settlement simulation.
use crate::{calendar::Season, performance::VisualQuality, simulation::Settlement};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DaylightUpdate;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Daylight>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    animate_meadow,
                    daylight.in_set(DaylightUpdate),
                    seasonal_palette,
                ),
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
// Small deterministic hash: identical scenery on every platform, no RNG dependency.
pub(crate) fn noise(index: u32) -> f32 {
    let mut n = index.wrapping_add(0x9e3779b9);
    n = (n ^ (n >> 16)).wrapping_mul(0x85ebca6b);
    n = (n ^ (n >> 13)).wrapping_mul(0xc2b2ae35);
    (n ^ (n >> 16)) as f32 / u32::MAX as f32
}

pub(crate) fn grass_mesh() -> Mesh {
    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uv = Vec::new();
    for i in 0..56 {
        let x = (noise(i * 7) - 0.5) * 1.3;
        let z = (noise(i * 7 + 1) - 0.5) * 1.3;
        let h = 0.12 + noise(i * 7 + 2) * 0.25;
        let a = noise(i * 7 + 3) * std::f32::consts::TAU;
        let side = Vec3::new(a.cos(), 0., a.sin());
        let lean = Vec3::new(a.sin(), 0., -a.cos()) * (0.05 + noise(i + 600) * 0.12);
        for k in 0..3 {
            for (step, sign) in [
                (0., -1.),
                (1., -1.),
                (1., 1.),
                (0., -1.),
                (1., 1.),
                (0., 1.),
            ] {
                let t = (k as f32 + step) / 3.;
                let p = Vec3::new(x, h * t, z) + lean * t * t + side * (sign * 0.014 * (1. - t));
                points.push(p.to_array());
                normals.push(
                    (Vec3::Y * 0.35 + side.cross(Vec3::Y))
                        .normalize()
                        .to_array(),
                );
                let tint = 0.75 + noise(i + 900) * 0.22;
                colors.push([tint * (0.68 + t * 0.25), tint, tint * 0.64, 1.]);
                uv.push([sign * 0.5 + 0.5, t]);
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    quality: Res<VisualQuality>,
    server: Res<AssetServer>,
    mut ground_materials: ResMut<Assets<crate::terrain_material::GroundMaterial>>,
) {
    let ground = crate::landscape::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        &server,
        *quality,
        &mut ground_materials,
    );
    for entity in ground {
        commands.entity(entity).insert(SeasonalSurface::Meadow);
    }
    spawn_farmstead(
        &mut commands,
        &mut meshes,
        &mut materials,
        *quality,
        &server,
    );
    spawn_pines(
        &mut commands,
        &mut meshes,
        &mut materials,
        *quality,
        &server,
    );
    spawn_grass(&mut commands, &mut meshes, &mut materials, *quality);
    let sun = commands
        .spawn((
            DirectionalLight {
                illuminance: 11_000.0,
                shadow_maps_enabled: quality.shadows(),
                contact_shadows_enabled: *quality == VisualQuality::High,
                ..default()
            },
            Transform::default(),
            Sun,
        ))
        .id();
    if *quality == VisualQuality::High {
        commands.entity(sun).insert((
            bevy::light::SunDisk::EARTH,
            bevy::light::CascadeShadowConfigBuilder {
                maximum_distance: 140.0,
                first_cascade_far_bound: 12.0,
                ..default()
            }
            .build(),
        ));
    }
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.73, 0.82, 0.94),
        brightness: 260.0,
        affects_lightmapped_meshes: true,
    });
}
fn spawn_farmstead(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    _quality: VisualQuality,
    server: &AssetServer,
) {
    let timber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.72, 0.72),
        perceptual_roughness: 0.95,
        ..crate::coast::material(server, "wood")
    });
    let warm_wood = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        perceptual_roughness: 0.95,
        ..crate::coast::material(server, "wood")
    });
    let thatch = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.92, 0.90),
        base_color_texture: Some(crate::coast::texture(server, "thatch-color", true)),
        normal_map_texture: Some(crate::coast::texture(server, "thatch-normal", false)),
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        ..default()
    });
    // Earth-toned infill and weathered posts retain the owned home's exact footprint.
    let cube = meshes.add(crate::art::softened_box(Vec3::ONE, 0.035));
    let infill = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.65, 0.55),
        perceptual_roughness: 1.,
        ..crate::coast::material(server, "limestone")
    });
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(infill),
        Transform::from_xyz(2., 1.25, 0.).with_scale(Vec3::new(3.5, 2.1, 5.)),
    ));
    let foundation = materials.add(crate::coast::material(server, "limestone"));
    for x in [0.27, 3.73] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(foundation.clone()),
            Transform::from_xyz(x, 0.36, 0.).with_scale(Vec3::new(0.27, 0.32, 5.15)),
        ));
        for z in [-2.5, -0.83, 0.83, 2.5] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(x, 1.34, z).with_scale(Vec3::new(0.13, 1.95, 0.14)),
            ));
        }
        for y in [0.51, 2.20] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(x, y, 0.).with_scale(Vec3::new(0.15, 0.12, 5.1)),
            ));
        }
        for z in [-1.65, 1.65] {
            commands.spawn((
                Mesh3d(meshes.add(crate::art::tube(
                    &[Vec3::new(x, 0.55, z - 0.70), Vec3::new(x, 2.15, z + 0.70)],
                    0.052,
                ))),
                MeshMaterial3d(timber.clone()),
                Transform::default(),
            ));
        }
    }
    for z in [-2.52, 2.52] {
        for x in [0.28, 1.50, 2.50, 3.72] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(x, 1.32, z).with_scale(Vec3::new(0.13, 1.95, 0.14)),
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
        Mesh3d(meshes.add(crate::art::tube(&ridge, 0.045))),
        MeshMaterial3d(thatch.clone()),
        Transform::default(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(crate::art::thatch_ends())),
        MeshMaterial3d(thatch.clone()),
        Transform::from_xyz(2.0, 0.0, 0.0),
    ));
    // Bent ridge spars and roof battens cast real shadows rather than painted bands.
    for i in 0..7 {
        let z = -2.65 + i as f32 * 0.88;
        let path: Vec<_> = (0..25)
            .map(|j| {
                let x = -2.10 + j as f32 * 4.20 / 24.0;
                Vec3::new(x + 2.0, crate::art::roof_height(x, z) + 0.08, z)
            })
            .collect();
        commands.spawn((
            Mesh3d(meshes.add(crate::art::tube(&path, 0.032))),
            MeshMaterial3d(timber.clone()),
            Transform::default(),
        ));
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
    for z in [-2.52, 2.52] {
        for i in 0..15 {
            let x = 0.32 + i as f32 * 0.24;
            let h = (1.05 - (x - 2.).abs() * 0.60).max(0.04);
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(timber.clone()),
                Transform::from_xyz(x, 2.3 + h * 0.5, z).with_scale(Vec3::new(0.025, h, 0.025)),
            ));
        }
    }
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
    server: &AssetServer,
) {
    let timber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.72, 0.72),
        perceptual_roughness: 0.95,
        ..crate::coast::material(server, "wood")
    });
    // Shared pine positions also define trunk clearance for first-person walking.
    let foliage = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.94, 0.85),
        base_color_texture: Some(crate::coast::texture(server, "pine-bough", true)),
        alpha_mode: AlphaMode::Mask(0.35),
        cull_mode: None,
        double_sided: true,
        diffuse_transmission: 0.35,
        perceptual_roughness: 1.0,
        ..default()
    });
    let pine = meshes.add(crate::art::foliage());
    let trunk = meshes.add(Cone::new(0.16, 5.4));
    let branch = meshes.add(crate::art::tube(
        &[
            Vec3::ZERO,
            Vec3::new(0.4, -0.03, 0.),
            Vec3::new(0.8, -0.10, 0.),
        ],
        0.018,
    ));
    for i in 0..9 {
        let Vec2 { x, y: z } = pine_position(i);
        commands.spawn((
            Mesh3d(trunk.clone()),
            MeshMaterial3d(timber.clone()),
            Transform::from_xyz(x, 2.90, z),
        ));
        for layer in 0..5 {
            let h = 2.75 + layer as f32 * 0.52;
            let width = (1.35 - layer as f32 * 0.20) * (0.90 + noise(i + 300) * 0.2);
            let branches = if quality == VisualQuality::High { 5 } else { 3 };
            for bough in 0..branches {
                let angle = bough as f32 * std::f32::consts::TAU / branches as f32
                    + layer as f32 * 0.71
                    + i as f32;
                commands.spawn((
                    Mesh3d(branch.clone()),
                    MeshMaterial3d(timber.clone()),
                    Transform::from_xyz(x, h, z)
                        .with_rotation(Quat::from_rotation_y(-angle))
                        .with_scale(Vec3::splat(width)),
                ));
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
                    .with_scale(Vec3::new(width * 0.86, 0.8, width * 0.72)),
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
        base_color: Color::srgb(0.61, 0.66, 0.43),
        cull_mode: None,
        double_sided: true,
        diffuse_transmission: 0.35,
        perceptual_roughness: 1.0,
        ..default()
    });
    for i in 0..quality.grass_patches() {
        let x = (noise(i * 3) - 0.5) * 16.8;
        let z = (noise(i * 3 + 1) - 0.5) * 11.8;
        // Keep house, work sites and the south stockpile approach visually clear.
        let field = crate::farming::FIELD_CENTER;
        if !walkable(Vec2::new(x, z))
            || (x < -5.8 && z > 4.5)
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
#[derive(bevy::ecs::system::SystemParam)]
struct DaylightEnvironment<'w, 's> {
    ambient: ResMut<'w, GlobalAmbientLight>,
    sky: ResMut<'w, ClearColor>,
    quality: Res<'w, VisualQuality>,
    fog: Query<'w, 's, &'static mut DistanceFog>,
}
fn daylight(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut day: ResMut<Daylight>,
    mut suns: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    environment: DaylightEnvironment,
) {
    let DaylightEnvironment {
        mut ambient,
        mut sky,
        quality,
        mut fog,
    } = environment;
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
    let physical = *quality == VisualQuality::High;
    ambient.brightness = if physical {
        0.0
    } else {
        60.0 + daylight * 200.0
    };
    let (rotation, illuminance) = sun_at(day.hour);
    for (mut transform, mut light) in &mut suns {
        transform.rotation = if physical {
            crate::rendering::solar_frame(day.hour).rotation
        } else {
            rotation
        };
        light.illuminance = if physical {
            bevy::light::light_consts::lux::RAW_SUNLIGHT
        } else {
            illuminance
        };
        light.color = if physical {
            Color::WHITE
        } else if (6.0..9.0).contains(&day.hour) || (16.0..20.0).contains(&day.hour) {
            Color::srgb(1.0, 0.90, 0.77)
        } else {
            Color::srgb(0.91, 0.94, 1.0)
        };
    }
}

// The movement shore follows the actual mesh, with a small dry-land setback.
fn inside_shore(p: Vec2) -> bool {
    crate::landscape::dry(p)
}
fn pine_position(i: u32) -> Vec2 {
    Vec2::new(
        -7.7 + i as f32 * 1.7 + (noise(i + 200) - 0.5) * 0.8,
        -6.65 - noise(i + 201) * 0.65,
    )
}
pub fn walkable(p: Vec2) -> bool {
    inside_shore(p)
        && !crate::landscape::tree_blocks(p)
        && !crate::landscape::wall_blocks(p)
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
        for p in [Vec2::new(-18., 13.), Vec2::new(0., 15.), Vec2::new(70., 0.)] {
            assert!(!walkable(p));
        }
        assert!(
            walkable(Vec2::new(0., -20.)) || crate::landscape::tree_blocks(Vec2::new(0., -20.))
        );
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
            (Vec2::new(-3.0, 4.0), Vec2::new(-80.0, 0.0)),
        ] {
            let end = slide(start, movement);
            assert!(walkable(end));
            assert!(end.distance(start) < movement.length());
        }
    }
}

#[derive(Component)]
pub(crate) enum SeasonalSurface {
    Meadow,
    Grass,
}
fn seasonal_palette(
    settlement: Option<Res<Settlement>>,
    surfaces: Query<(&SeasonalSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last: Local<Option<Season>>,
    mut ground: Option<ResMut<Assets<crate::terrain_material::GroundMaterial>>>,
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
                SeasonalSurface::Meadow => {
                    let c = meadow.to_srgba();
                    Color::srgb(
                        0.80 + c.red * 0.4,
                        0.80 + c.green * 0.4,
                        0.80 + c.blue * 0.4,
                    )
                }
                SeasonalSurface::Grass => grass,
            };
        }
    }
    if let Some(ground) = ground.as_mut() {
        let c = meadow.to_srgba();
        let tint = Vec4::new(
            0.80 + c.red * 0.4,
            0.80 + c.green * 0.4,
            0.80 + c.blue * 0.4,
            1.,
        );
        for (_, material) in ground.iter_mut() {
            material.base.base_color = Color::srgb(tint.x, tint.y, tint.z);
            material.extension.tint = tint;
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
