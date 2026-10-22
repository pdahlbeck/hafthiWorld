//! Two-layer terrain: meadow albedo blends into the original soil at paths and the beach.
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Meadow {
    #[texture(100)]
    #[sampler(101)]
    pub color: Handle<Image>,
    #[uniform(102)]
    pub tint: Vec4,
}
impl MaterialExtension for Meadow {
    fn fragment_shader() -> ShaderRef {
        "embedded://hafthi_world/shaders/coastal_ground.wesl".into()
    }
}
pub type GroundMaterial = ExtendedMaterial<StandardMaterial, Meadow>;
pub struct GroundPlugin;
impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GroundMaterial>::default());
    }
}
