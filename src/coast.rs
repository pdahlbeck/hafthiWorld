//! Continuous coastal surface and original tiled PBR materials. No simulation state.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub fn material(server: &AssetServer, kind: &str) -> StandardMaterial {
    let load = |suffix: &str, srgb| {
        server
            .load_builder()
            .with_settings(move |s: &mut ImageLoaderSettings| {
                s.is_srgb = srgb;
                s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..ImageSamplerDescriptor::linear()
                });
            })
            .load(format!(
                "embedded://hafthi_world/materials/{kind}-{suffix}.png"
            ))
    };
    StandardMaterial {
        base_color_texture: Some(load("color", true)),
        normal_map_texture: Some(load("normal", false)),
        metallic_roughness_texture: Some(load("orm", false)),
        perceptual_roughness: 1.,
        ..default()
    }
}
fn hash(i: u32) -> f32 {
    crate::world::noise(i)
}
/// Broken limestone beds, with angular ledges rather than inflated ellipsoids.
pub fn rock(seed: u32) -> Mesh {
    let mut v = Vec::new();
    let mut uv = Vec::new();
    let mut colors = Vec::new();
    let mut idx = Vec::new();
    let levels = [
        (-0.22, 0.68),
        (-0.16, 1.),
        (-0.08, 0.94),
        (-0.055, 1.02),
        (0.05, 0.81),
        (0.075, 0.86),
        (0.18, 0.67),
        (0.21, 0.45),
    ];
    for (k, (height, radius)) in levels.into_iter().enumerate() {
        for j in 0..=16 {
            let a = j as f32 / 16. * std::f32::consts::TAU;
            let radius = radius * (0.85 + hash(seed + (j % 16) as u32) * 0.25);
            let p = Vec3::new(
                a.cos() * radius,
                height + hash(seed + 100 + (j % 16) as u32) * 0.035,
                a.sin() * radius * 0.64,
            );
            v.push(p.to_array());
            uv.push([j as f32 / 8., k as f32 * 0.12]);
            let shade = 0.76 + k as f32 * 0.025 + hash(seed + j as u32) * 0.06;
            colors.push([shade, shade, shade, 1.]);
        }
    }
    for k in 0..7 {
        for j in 0..16 {
            let a = (k * 17 + j) as u32;
            idx.extend([a, a + 17, a + 18, a, a + 18, a + 1]);
        }
    }
    for j in 1..15 {
        idx.extend([119, 119 + j + 1, 119 + j]);
    }
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, v)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(idx));
    m.compute_smooth_normals();
    m.generate_tangents()
        .expect("authored surface has UVs and normals");
    m
}
/// A weathered stack with an undercut waist and wider cap, rather than stretching a boulder.
pub fn limestone_stack(seed: u32) -> Mesh {
    use bevy::mesh::VertexAttributeValues;
    let mut mesh = rock(seed);
    let original = [0.68, 1., 0.94, 1.02, 0.81, 0.86, 0.67, 0.45];
    let widths = [0.80, 0.94, 0.70, 0.53, 0.57, 0.88, 0.96, 0.78];
    let heights = [0., 0.10, 0.28, 0.43, 0.58, 0.70, 0.86, 1.];
    if let Some(VertexAttributeValues::Float32x3(points)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for (i, p) in points.iter_mut().enumerate() {
            let row = i / 17;
            p[0] *= widths[row] / original[row];
            p[2] *= widths[row] / original[row];
            p[1] = heights[row] + crate::world::noise(seed + (i % 17) as u32 + 100) * 0.018;
        }
    }
    mesh.compute_smooth_normals();
    mesh.generate_tangents().expect("stack UVs");
    mesh
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn terrain_and_strata_have_finite_vertices_normals_and_nonconstant_uvs() {
        for mesh in [
            crate::landscape::chunk(Vec2::new(-12., -12.), 12),
            rock(901),
            limestone_stack(901),
        ] {
            for attribute in [Mesh::ATTRIBUTE_POSITION, Mesh::ATTRIBUTE_NORMAL] {
                let Some(VertexAttributeValues::Float32x3(v)) = mesh.attribute(attribute) else {
                    panic!("missing attribute")
                };
                assert!(v.iter().flatten().all(|x| x.is_finite()));
            }
            let Some(VertexAttributeValues::Float32x2(uv)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
            else {
                panic!("uv")
            };
            assert!(uv.windows(2).any(|v| v[0] != v[1]));
        }
    }
}
