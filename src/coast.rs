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
fn path_distance(p: Vec2) -> f32 {
    [
        (Vec2::new(2., 3.), Vec2::new(-4., 4.)),
        (Vec2::new(-4., 4.), Vec2::new(-7., 6.5)),
        (Vec2::new(-4., 4.), Vec2::new(-4., -3.)),
        (Vec2::new(2., 3.), Vec2::new(5.5, 1.5)),
    ]
    .into_iter()
    .map(|(a, b)| {
        let d = b - a;
        p.distance(a + d * ((p - a).dot(d) / d.length_squared()).clamp(0., 1.))
    })
    .fold(f32::MAX, f32::min)
}
/// Radial tessellation makes the meadow, gravel shoulder and submerged shelf one surface.
/// The playable centre stays level at y=0.2, matching the existing collision/route contract.
pub fn ground() -> Mesh {
    let mut points = Vec::new();
    let mut uv = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    const N: usize = 128;
    const R: usize = 32;
    for r in 0..=R {
        for i in 0..=N {
            let scale = r as f32 / R as f32 * 1.22;
            let angle = i as f32 / N as f32 * std::f32::consts::TAU;
            let edge = 1. + 0.045 * (angle * 5.).sin() + 0.025 * (angle * 11.).cos();
            let (c, s) = (angle.cos(), angle.sin());
            let x = c.signum() * c.abs().sqrt() * 11.2 * edge * scale;
            let z = s.signum() * s.abs().sqrt() * 8.3 * edge * scale;
            let y = if scale < 0.92 {
                0.2
            } else {
                0.2 - ((scale - 0.92) / 0.30).clamp(0., 1.) * 0.64
            };
            points.push([x, y, z]);
            uv.push([x * 0.58, z * 0.58]);
            let patch = ((x * 0.62 + z * 0.31).sin() * (z * 0.78 - x * 0.19).cos()) * 0.5 + 0.5;
            let meadow = Vec3::new(0.48, 0.67, 0.44).lerp(Vec3::new(0.76, 0.78, 0.62), patch);
            let path = (1. - path_distance(Vec2::new(x, z)) / 0.65).clamp(0., 1.) * 0.85;
            let land = meadow.lerp(Vec3::new(1.05, 0.99, 0.87), path);
            let beach = ((scale - 0.83) / 0.18).clamp(0., 1.);
            let color = land.lerp(Vec3::new(1.12, 1.10, 1.01), beach);
            colors.push([color.x, color.y, color.z, 1.]);
        }
    }
    for r in 0..R {
        for i in 0..N {
            let a = (r * (N + 1) + i) as u32;
            let b = a + (N + 1) as u32;
            indices.extend([a, b + 1, b, a, a + 1, b + 1]);
        }
    }
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices));
    m.compute_smooth_normals();
    m.generate_tangents()
        .expect("authored surface has UVs and normals");
    m
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
pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    server: &AssetServer,
) -> Entity {
    let ground = commands
        .spawn((
            Mesh3d(meshes.add(ground())),
            MeshMaterial3d(materials.add(material(server, "ground"))),
            Transform::default(),
        ))
        .id();
    let stone = materials.add(material(server, "limestone"));
    let rocks: Vec<_> = (0..6).map(|i| meshes.add(rock(i * 193))).collect();
    for i in 0..110 {
        // Broken clusters and gaps avoid a uniform necklace around the island.
        if hash(i + 271) < 0.29 {
            continue;
        }
        let p = crate::world::coast_point((i * 17 % 64) as usize, 0.92 + hash(i + 190) * 0.12);
        let scale = 0.38 + hash(i + 300) * 0.80;
        commands.spawn((
            Mesh3d(rocks[i as usize % rocks.len()].clone()),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(p.x, -0.03, p.z)
                .with_rotation(Quat::from_rotation_y(hash(i + 500) * std::f32::consts::TAU))
                .with_scale(Vec3::new(scale, scale * 0.8, scale)),
        ));
    }
    commands.spawn((
        Mesh3d(rocks[2].clone()),
        MeshMaterial3d(stone),
        Transform::from_xyz(-7., 0.32, 1.).with_scale(Vec3::new(1.5, 1.2, 1.)),
    ));
    let sea = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.24, 0.27),
        perceptual_roughness: 0.27,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1200., 1200.))),
        MeshMaterial3d(sea),
        Transform::from_xyz(0., -0.38, 0.),
    ));
    ground
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn terrain_and_strata_have_finite_vertices_normals_and_nonconstant_uvs() {
        for mesh in [ground(), rock(901)] {
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
