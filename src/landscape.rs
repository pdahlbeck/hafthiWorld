//! Shared height field, render chunks and original scenic vegetation for the coastal hinterland.
//! Workers keep their existing level clearing; the observer can walk into the surrounding hills.
use crate::{coast, performance::VisualQuality, world::noise};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub const WATER_LEVEL: f32 = -0.38;
pub const MIN: Vec2 = Vec2::new(-60., -80.);
pub const MAX: Vec2 = Vec2::new(60., 28.);
/// Dry beach crest, also used by the analytical shore/water material.
pub fn shoreline(x: f32) -> f32 {
    8.7 + 1.4 * (x * 0.075).sin() + 0.5 * (x * 0.29).sin()
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
pub fn height(p: Vec2) -> f32 {
    let d = p.y - shoreline(p.x);
    if d > -0.8 {
        return (0.2 - (d + 0.8) * 0.29).max(-3.);
    }
    let hinterland = smooth(9., 25., (-p.y).max(p.x.abs() - 3.));
    let hills = 2.8
        + 1.5 * (p.x * 0.073 + p.y * 0.046).sin()
        + 1.1 * (p.y * 0.13 - p.x * 0.029).cos()
        + 0.35 * (p.x * 0.37 + p.y * 0.22).sin();
    0.2 + hinterland * hills.max(0.) * smooth(1.8, 9., -d)
}
pub fn normal(p: Vec2) -> Vec3 {
    let e = 0.03;
    Vec3::new(
        height(p - Vec2::X * e) - height(p + Vec2::X * e),
        2. * e,
        height(p - Vec2::Y * e) - height(p + Vec2::Y * e),
    )
    .normalize()
}
pub fn dry(p: Vec2) -> bool {
    p.is_finite() && p.x > MIN.x + 1. && p.x < MAX.x - 1. && p.y > MIN.y + 1. && height(p) > 0.03
}
fn yard_wear(p: Vec2) -> f32 {
    [
        (Vec2::new(2., 3.), Vec2::new(-2.5, 3.5)),
        (Vec2::new(-2.5, 3.5), Vec2::new(-6., 5.)),
        (Vec2::new(-6., 5.), Vec2::new(-7., 6.6)),
        (Vec2::new(2., 3.), Vec2::new(6., 4.5)),
        (Vec2::new(-2.5, 3.5), Vec2::new(-4.5, -3.)),
    ]
    .into_iter()
    .map(|(a, b)| (1. - distance_to_segment(p, a, b) / 0.7).clamp(0., 1.))
    .fold(0., f32::max)
}
pub fn chunk(origin: Vec2, steps: u32) -> Mesh {
    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    for z in 0..=steps {
        for x in 0..=steps {
            let p = origin + Vec2::new(x as f32, z as f32) * (12. / steps as f32);
            let h = height(p);
            let n = normal(p);
            points.push([p.x, h, p.y]);
            normals.push(n.to_array());
            uv.push([p.x * 0.58, p.y * 0.58]);
            let patch = (0.5
                + 0.3 * (p.x * 0.19 + p.y * 0.21).sin()
                + 0.2 * (p.x * 0.43 - p.y * 0.11).cos())
            .clamp(0., 1.);
            let green = Vec3::new(0.83, 0.92, 0.77).lerp(Vec3::new(1.08, 1.05, 0.94), patch);
            let green = green.lerp(Vec3::new(1.1, 1.02, 0.87), yard_wear(p) * 0.85);
            let beach = smooth(-3.5, 0.5, p.y - shoreline(p.x));
            let exposed = (1. - n.y).max(0.) * 2.;
            let c = green.lerp(
                Vec3::new(1.05, 1.02, 0.91),
                beach.max(exposed).clamp(0., 1.),
            );
            let grass = (1. - beach.max(exposed).clamp(0., 1.)) * (1. - yard_wear(p) * 0.9);
            colors.push([c.x, c.y, c.z, grass]);
        }
    }
    for z in 0..steps {
        for x in 0..steps {
            let a = z * (steps + 1) + x;
            let b = a + steps + 1;
            indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices));
    m.generate_tangents().expect("height field UVs");
    m
}
/// Deterministic scenic trees, distinct from the three finite harvestable trees.
pub fn tree(i: u32) -> (Vec2, f32) {
    let p = Vec2::new(
        -53. + noise(i * 5 + 2000) * 106.,
        -12. - noise(i * 5 + 2001) * 60.,
    );
    (p, 3.2 + noise(i * 5 + 2002) * 4.5)
}
pub fn tree_blocks(p: Vec2) -> bool {
    (0..180).any(|i| {
        let (q, h) = tree(i);
        p.distance_squared(q) < (h * 0.0195 + 0.12).powi(2)
    })
}
const WALLS: [(Vec2, Vec2); 4] = [
    (Vec2::new(11., -6.), Vec2::new(11., -17.)),
    (Vec2::new(11., -17.), Vec2::new(26., -21.)),
    (Vec2::new(-26., -16.), Vec2::new(-12., -12.)),
    (Vec2::new(-26., -16.), Vec2::new(-30., -28.)),
];
fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let d = b - a;
    p.distance(a + d * ((p - a).dot(d) / d.length_squared()).clamp(0., 1.))
}
pub fn wall_blocks(p: Vec2) -> bool {
    WALLS
        .iter()
        .any(|(a, b)| distance_to_segment(p, *a, *b) < 0.35)
}
fn stone_walls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    rock: &Mesh,
) {
    for (segment, (a, b)) in WALLS.into_iter().enumerate() {
        let count = (a.distance(b) / 0.42).ceil() as u32;
        let mut merged = None;
        for row in 0..3 {
            for stone in 0..count {
                let id = segment as u32 * 1000 + row * 100 + stone;
                let t = (stone as f32 + row as f32 * 0.30) / (count as f32);
                let p = a.lerp(b, t);
                let h = height(p);
                let transform = Transform::from_xyz(p.x, h + 0.13 + row as f32 * 0.19, p.y)
                    .with_scale(Vec3::new(0.27, 0.48, 0.33))
                    .with_rotation(Quat::from_rotation_y(
                        noise(id + 109) * std::f32::consts::TAU,
                    ));
                merge_mesh(&mut merged, rock.clone().transformed_by(transform));
            }
        }
        commands.spawn((
            Mesh3d(meshes.add(merged.expect("wall courses"))),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Name::new("Dry stone boundary"),
        ));
    }
}
fn merge_mesh(target: &mut Option<Mesh>, mesh: Mesh) {
    if let Some(target) = target {
        target.merge(&mesh).expect("same authored attributes");
    } else {
        *target = Some(mesh);
    }
}
fn leaf_spray(seed: u32, count: u32) -> Mesh {
    // Individual folded leaves create a porous silhouette and directional leaf normals.
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uv = Vec::new();
    for i in 0..count {
        let id = seed + i * 11;
        let centre = Vec3::new(
            (noise(id + 1) - 0.5) * 1.7,
            (noise(id + 2) - 0.5) * 1.4,
            (noise(id + 3) - 0.5) * 1.7,
        );
        if centre.length() > 0.85 {
            continue;
        }
        let a = noise(id + 4) * std::f32::consts::TAU;
        let long = Vec3::new(a.cos(), 0.1 + (noise(id + 5) - 0.5), a.sin()).normalize()
            * (0.085 + noise(id + 6) * 0.055);
        let side = long.cross(Vec3::Y).normalize() * 0.045;
        let ridge = centre + Vec3::Y * 0.014;
        let tone = 0.65 + noise(id + 7) * 0.35;
        for triangle in [
            [centre - long, centre + side, ridge],
            [centre + side, centre + long, ridge],
            [centre + long, centre - side, ridge],
            [centre - side, centre - long, ridge],
        ] {
            let n = (triangle[1] - triangle[0])
                .cross(triangle[2] - triangle[0])
                .normalize();
            for (j, p) in triangle.into_iter().enumerate() {
                positions.push(p.to_array());
                normals.push(n.to_array());
                colors.push([tone * 0.83, tone, tone * 0.72, 1.]);
                uv.push([j as f32 * 0.5, 0.]);
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
}
pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    server: &AssetServer,
    quality: VisualQuality,
    ground_materials: &mut Assets<crate::terrain_material::GroundMaterial>,
) -> Vec<Entity> {
    let mut ground_entities = Vec::new();
    let meadow = server
        .load_builder()
        .with_settings(|s: &mut bevy::image::ImageLoaderSettings| {
            s.sampler =
                bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    ..bevy::image::ImageSamplerDescriptor::linear()
                });
        })
        .load("embedded://hafthi_world/materials/meadow-color.png");
    let soil = ground_materials.add(crate::terrain_material::GroundMaterial {
        base: coast::material(server, "ground"),
        extension: crate::terrain_material::Meadow {
            color: meadow,
            tint: Vec4::ONE,
        },
    });
    for iz in 0..9 {
        for ix in 0..10 {
            let origin = MIN + Vec2::new(ix as f32, iz as f32) * 12.;
            let steps = if quality == VisualQuality::High {
                24
            } else {
                12
            };
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(chunk(origin, steps))),
                    MeshMaterial3d(soil.clone()),
                    Transform::default(),
                    Name::new("Height field chunk"),
                ))
                .id();
            ground_entities.push(entity);
        }
    }
    let stone = materials.add(coast::material(server, "limestone"));
    let rock = coast::rock(901);
    stone_walls(commands, meshes, stone.clone(), &rock);
    // Strand gravel and weathered inland stones are merged per strip for culling/batching.
    for strip in 0..10 {
        let mut merged = None;
        for i in 0..90 {
            let id = strip * 100 + i;
            let x = -60. + strip as f32 * 12. + noise(id + 44) * 12.;
            let z = shoreline(x) - 1.5 + noise(id + 45) * 4.;
            let p = Vec2::new(x, z);
            let s = 0.045 + noise(id + 46).powi(3) * 0.65;
            merge_mesh(
                &mut merged,
                rock.clone().transformed_by(
                    Transform::from_xyz(x, height(p) + s * 0.08, z)
                        .with_scale(Vec3::splat(s))
                        .with_rotation(Quat::from_rotation_y(
                            noise(id + 47) * std::f32::consts::TAU,
                        )),
                ),
            );
        }
        if let Some(mesh) = merged {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(stone.clone()),
                Transform::default(),
            ));
        }
    }
    // Keep the existing large limestone obstacle and its original collision footprint.
    commands.spawn((
        Mesh3d(meshes.add(rock.clone())),
        MeshMaterial3d(stone.clone()),
        Transform::from_xyz(-7., 0.32, 1.).with_scale(Vec3::new(1.5, 1.2, 1.)),
    ));
    for (i, x) in [19., 27., 36., 44., 53.].into_iter().enumerate() {
        let z = shoreline(x) + 0.8 + noise(i as u32 + 702) * 0.7;
        let vertical = 2.8 + noise(i as u32 + 700) * 1.5;
        commands.spawn((
            Mesh3d(meshes.add(coast::limestone_stack(i as u32 * 71))),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(x, height(Vec2::new(x, z)), z).with_scale(Vec3::new(
                0.7 + noise(i as u32 + 701) * 0.5,
                vertical,
                0.9,
            )),
            Name::new("Eroded limestone stack"),
        ));
    }
    let bark = materials.add(coast::material(server, "wood"));
    let leaves = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.44, 0.28),
        cull_mode: None,
        double_sided: true,
        diffuse_transmission: 0.35,
        perceptual_roughness: 1.,
        ..default()
    });
    let crown = leaf_spray(
        1901,
        if quality == VisualQuality::High {
            320
        } else {
            180
        },
    );
    let trunk = meshes.add(Cone::new(0.13, 1.).mesh().resolution(10));
    // Per-tree crowns share source geometry and merge lobes into one draw, rather than one entity per leaf/bough.
    let mut crown_variants = Vec::new();
    for variant in 0..5 {
        let mut merged = None;
        for lobe in 0..9 {
            let id = variant * 17 + lobe;
            let pos = Vec3::new(
                (noise(id + 66) - 0.5) * 0.66,
                0.62 + noise(id + 67) * 0.32,
                (noise(id + 68) - 0.5) * 0.66,
            );
            merge_mesh(
                &mut merged,
                crown.clone().transformed_by(
                    Transform::from_translation(pos).with_scale(Vec3::new(0.36, 0.36, 0.36)),
                ),
            );
        }
        crown_variants.push(meshes.add(merged.expect("crown lobes")));
    }
    for i in 0..180 {
        let (p, h) = tree(i);
        let base = height(p);
        commands.spawn((
            Mesh3d(trunk.clone()),
            MeshMaterial3d(bark.clone()),
            Transform::from_xyz(p.x, base + h * 0.40, p.y).with_scale(Vec3::new(
                h * 0.15,
                h * 0.8,
                h * 0.15,
            )),
        ));
        commands.spawn((
            Mesh3d(crown_variants[i as usize % 5].clone()),
            MeshMaterial3d(leaves.clone()),
            Transform::from_xyz(p.x, base, p.y).with_scale(Vec3::splat(h)),
        ));
    }
    // Low shrubs bridge the formerly bare foreground and distant woodland.
    let bush = meshes.add(crown);
    for i in 0..140 {
        let p = Vec2::new(
            -38. + noise(i * 3 + 9000) * 76.,
            -8. - noise(i * 3 + 9001) * 40.,
        );
        let s = 0.35 + noise(i * 3 + 9002) * 0.65;
        commands.spawn((
            Mesh3d(bush.clone()),
            MeshMaterial3d(leaves.clone()),
            Transform::from_xyz(p.x, height(p) + s * 0.55, p.y).with_scale(Vec3::new(
                s,
                s * 0.75,
                s,
            )),
        ));
    }
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.72, 0.53),
        cull_mode: None,
        double_sided: true,
        diffuse_transmission: 0.35,
        perceptual_roughness: 1.,
        ..default()
    });
    let grass_source = crate::world::grass_mesh();
    for strip in 0..8 {
        let mut merged = None;
        let count = if quality == VisualQuality::High {
            32
        } else {
            12
        };
        for patch in 0..count {
            let id = strip * 100 + patch;
            let x = -44. + strip as f32 * 11. + noise(id + 12000) * 11.;
            let z = -9. - noise(id + 12001) * 46.;
            let mut grass = grass_source.clone();
            if let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
                grass.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                for p in points {
                    p[0] += x;
                    p[2] += z;
                    p[1] += height(Vec2::new(p[0], p[2])) + 0.01;
                }
            }
            merge_mesh(&mut merged, grass);
        }
        commands.spawn((
            Mesh3d(meshes.add(merged.expect("grass patches"))),
            MeshMaterial3d(grass_material.clone()),
            Transform::default(),
            crate::world::SeasonalSurface::Grass,
            bevy::light::NotShadowCaster,
        ));
    }
    ground_entities
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn existing_work_and_boat_coordinates_keep_their_surface_contract() {
        for p in crate::simulation::TREE_POSITIONS.into_iter().chain([
            crate::simulation::STOCKPILE,
            crate::simulation::BUILD_SITE,
            Vec2::new(-7., 6.6),
        ]) {
            assert!((height(p) - 0.2).abs() < 0.00001, "{p:?}");
        }
        assert!(height(Vec2::new(-8., 9.1)) < WATER_LEVEL);
        assert!(height(Vec2::new(-18., 13.)) < WATER_LEVEL);
        assert!(height(Vec2::new(0., -40.)) > 1.);
    }
    #[test]
    fn hills_fade_to_beach_without_a_height_discontinuity() {
        for x in -55..55 {
            let z = shoreline(x as f32) - 0.8;
            let a = height(Vec2::new(x as f32, z - 0.001));
            let b = height(Vec2::new(x as f32, z + 0.001));
            assert!((a - b).abs() < 0.001, "beach discontinuity at {x}: {a}/{b}");
        }
    }
    #[test]
    fn scenic_colliders_match_authored_positions_without_blocking_work() {
        for i in 0..180 {
            assert!(tree_blocks(tree(i).0));
        }
        for (a, b) in WALLS {
            assert!(wall_blocks(a.lerp(b, 0.5)));
        }
        for p in crate::simulation::TREE_POSITIONS
            .into_iter()
            .chain([crate::simulation::STOCKPILE])
        {
            assert!(!wall_blocks(p) && !tree_blocks(p));
        }
    }
    #[test]
    fn normals_are_finite_and_chunk_edges_match() {
        for x in -50..50 {
            for z in -70..20 {
                let p = Vec2::new(x as f32, z as f32);
                assert!(height(p).is_finite());
                assert!((normal(p).length() - 1.).abs() < 0.0001);
            }
        }
        use bevy::mesh::VertexAttributeValues;
        let a = chunk(Vec2::new(-12., -12.), 12);
        let b = chunk(Vec2::new(0., -12.), 12);
        let Some(VertexAttributeValues::Float32x3(a)) = a.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        let Some(VertexAttributeValues::Float32x3(b)) = b.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        for row in 0..=12 {
            assert_eq!(a[row * 13 + 12], b[row * 13]);
        }
    }
}
