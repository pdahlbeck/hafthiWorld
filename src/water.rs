//! Portable forward-PBR water extension: world-space waves, shore absorption and sky Fresnel.
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct CoastalWater {
    #[uniform(100)]
    values: Vec4,
}
impl MaterialExtension for CoastalWater {
    fn fragment_shader() -> ShaderRef {
        "embedded://hafthi_world/shaders/coastal_water.wesl".into()
    }
}
type WaterMaterial = ExtendedMaterial<StandardMaterial, CoastalWater>;
pub struct WaterPlugin;
impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, animate);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1200., 1200.))),
        MeshMaterial3d(materials.add(WaterMaterial {
            base: StandardMaterial {
                base_color: Color::srgba(0.1, 0.2, 0.24, 0.9),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.2,
                reflectance: 0.5,
                ..default()
            },
            extension: CoastalWater {
                values: Vec4::new(0., 1., 0., 0.),
            },
        })),
        Transform::from_xyz(0., crate::landscape::WATER_LEVEL, 0.),
        bevy::light::NotShadowCaster,
        Name::new("Baltic surface"),
    ));
}
fn animate(
    time: Res<Time>,
    day: Res<crate::world::Daylight>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    let daylight = (((day.hour - 6.) / 24. * std::f32::consts::TAU).sin() + 0.15).clamp(0., 1.);
    for (_, m) in materials.iter_mut() {
        m.extension.values = Vec4::new(time.elapsed_secs(), daylight, 0., 0.);
    }
}
