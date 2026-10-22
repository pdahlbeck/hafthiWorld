//! Original procedural sculpted meshes. These shapes do not define simulation or collision.
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

fn mesh(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    uv: Vec<[f32; 2]>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
}
/// Rounded edges retain useful flat timber faces instead of inflating everything into spheres.
pub fn softened_box(size: Vec3, radius: f32) -> Mesh {
    let half = size * 0.5;
    let radius = radius.min(half.min_element() * 0.8).max(0.001);
    let inner = half - Vec3::splat(radius);
    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uv = Vec::new();
    for (normal, u, v) in [
        (Vec3::X, Vec3::Y, Vec3::Z),
        (-Vec3::X, Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::Z, Vec3::X),
        (-Vec3::Y, Vec3::X, Vec3::Z),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (-Vec3::Z, Vec3::Y, Vec3::X),
    ] {
        let hu = half.dot(u);
        let hv = half.dot(v);
        let axis = |h: f32| {
            [
                -h,
                -h + radius * 0.3,
                -h + radius,
                h - radius,
                h - radius * 0.3,
                h,
            ]
        };
        let us = axis(hu);
        let vs = axis(hv);
        for i in 0..5 {
            for j in 0..5 {
                for (a, b) in [(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 1)] {
                    let p = normal * half.dot(normal.abs()) + u * us[i + a] + v * vs[j + b];
                    let core = p.clamp(-inner, inner);
                    let n = (p - core).normalize();
                    points.push((core + n * radius).to_array());
                    normals.push(n.to_array());
                    colors.push([1.; 4]);
                    uv.push([(us[i + a] / hu + 1.) * 0.5, (vs[j + b] / hv + 1.) * 0.5]);
                }
            }
        }
    }
    mesh(points, normals, colors, uv)
}
fn surface(
    nu: usize,
    nv: usize,
    f: impl Fn(f32, f32) -> Vec3,
    color: impl Fn(f32, f32) -> [f32; 4],
    reverse: bool,
) -> Mesh {
    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uv = Vec::new();
    for i in 0..nu {
        for j in 0..nv {
            for (a, b) in if reverse {
                [(0., 0.), (1., 1.), (1., 0.), (0., 0.), (0., 1.), (1., 1.)]
            } else {
                [(0., 0.), (1., 0.), (1., 1.), (0., 0.), (1., 1.), (0., 1.)]
            } {
                let u = (i as f32 + a) / nu as f32;
                let v = (j as f32 + b) / nv as f32;
                let du = f((u + 0.001).min(1.), v) - f((u - 0.001).max(0.), v);
                let dv = f(u, (v + 0.001).min(1.)) - f(u, (v - 0.001).max(0.));
                let n = du.cross(dv).normalize_or_zero() * if reverse { -1. } else { 1. };
                points.push(f(u, v).to_array());
                normals.push(n.to_array());
                colors.push(color(u, v));
                uv.push([u, v]);
            }
        }
    }
    mesh(points, normals, colors, uv)
}
pub fn tunic() -> Mesh {
    let rings = [
        (-0.30, 0.27, 0.18),
        (-0.22, 0.29, 0.19),
        (-0.02, 0.21, 0.15),
        (0.14, 0.23, 0.16),
        (0.32, 0.27, 0.17),
        (0.41, 0.20, 0.14),
        (0.46, 0.10, 0.10),
    ];
    surface(
        24,
        rings.len() - 1,
        |u, v| {
            let ring = (v * (rings.len() - 1) as f32).min((rings.len() - 1) as f32 - 0.00001);
            let i = ring.floor() as usize;
            let t = ring - i as f32;
            let (y, x, z) = rings[i];
            let (ny, nx, nz) = rings[i + 1];
            let a = u * std::f32::consts::TAU;
            let fold = 1. + 0.025 * (a * 6.).cos();
            Vec3::new(
                (x + (nx - x) * t) * a.cos() * fold,
                y + (ny - y) * t,
                (z + (nz - z) * t) * a.sin() * fold,
            )
        },
        |u, v| {
            let shade = 0.86 + 0.10 * (u * std::f32::consts::TAU * 6.).cos() + v * 0.04;
            [shade, shade, shade, 1.]
        },
        true,
    )
}
pub fn thatch_roof() -> Mesh {
    surface(
        48,
        84,
        |u, v| {
            let x = (u - 0.5) * 4.28;
            let z = (v - 0.5) * 5.7;
            let grooves = (v * 42. * std::f32::consts::TAU).cos() * 0.018;
            Vec3::new(
                x,
                3.51 - 1.19 * (x.abs() / 2.14).powf(1.08) - 0.07 * (z / 2.85).powi(2) + grooves,
                z,
            )
        },
        |u, v| {
            let grain = (v * 42. * std::f32::consts::TAU).cos() * 0.06;
            let patch = (u * 31. + v * 17.).sin() * (v * 43.).sin() * 0.025;
            let light = 0.82 + grain + patch;
            [light, light * 0.95, light * 0.86, 1.]
        },
        true,
    )
}
pub fn boat_hull(inner: bool) -> Mesh {
    surface(
        56,
        24,
        |u, v| {
            let z = (u - 0.5) * 4.;
            let a = (v - 0.5) * std::f32::consts::PI;
            let taper = (1. - (z / 2.04).powi(2)).max(0.025).powf(0.65);
            let beam = 0.75 * taper;
            if inner {
                Vec3::new(
                    beam * a.sin() * 0.90,
                    -0.17 + (0.63 + 0.25 * (z / 2.).powi(4)) * (1. - a.cos()) + 0.05,
                    z,
                )
            } else {
                Vec3::new(
                    beam * a.sin(),
                    -0.20 + (0.63 + 0.25 * (z / 2.).powi(4)) * (1. - a.cos()),
                    z,
                )
            }
        },
        |u, v| {
            let plank = (v * 12.).floor() as u32;
            let shade = 0.82 + (plank % 3) as f32 * 0.065 + 0.025 * (u * 180. + v * 23.).sin();
            [shade, shade * 0.91, shade * 0.78, 1.]
        },
        !inner,
    )
}
pub fn boat_deck() -> Mesh {
    surface(
        48,
        8,
        |u, v| {
            let z = (u - 0.5) * 3.7;
            let width = 0.56 * (1. - (z / 2.05).powi(2)).max(0.02).sqrt();
            Vec3::new((v - 0.5) * 2. * width, 0.21, z)
        },
        |u, v| {
            let grain = 0.88 + 0.06 * (v * 8.).floor().sin() + 0.025 * (u * 170.).sin();
            [grain, grain * 0.9, grain * 0.72, 1.]
        },
        false,
    )
}
/// Continuous rounded timber rails/ridge, with a small polygon count and shared material.
pub fn tube(path: &[Vec3], radius: f32) -> Mesh {
    surface(
        path.len() - 1,
        10,
        |u, v| {
            let segment = (u * (path.len() - 1) as f32).min((path.len() - 1) as f32 - 0.00001);
            let i = segment.floor() as usize;
            let tangent = (path[i + 1] - path[i]).normalize();
            let side = tangent.cross(Vec3::Y).normalize_or_zero();
            let up = side.cross(tangent);
            let a = v * std::f32::consts::TAU;
            path[i].lerp(path[i + 1], segment - i as f32) + radius * (side * a.cos() + up * a.sin())
        },
        |_, _| [1.; 4],
        false,
    )
}
pub fn foliage() -> Mesh {
    // Open, layered needle sprays leave sky holes instead of a solid foliage balloon.
    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uv = Vec::new();
    for branch in 0..13 {
        let angle = branch as f32 * 2.39996;
        let direction = Vec3::new(angle.cos(), 0., angle.sin());
        let side = direction.cross(Vec3::Y);
        let length = 0.55 + crate::world::noise(branch + 60) * 0.45;
        for needle in 0..18 {
            let t = needle as f32 / 18.;
            let centre = direction * t * length + Vec3::Y * (0.22 * (1. - t) - 0.12 * t * t);
            let width = 0.16 * (1. - t) + 0.025;
            for sign in [-1., 1.] {
                let a = centre - direction * 0.08;
                let b = centre + side * sign * width - direction * 0.13 + Vec3::Y * 0.025;
                let c = centre + direction * 0.10 + Vec3::Y * 0.04;
                let n = (b - a).cross(c - a).normalize();
                for (p, tex) in [(a, [0., 0.]), (b, [1., 0.]), (c, [0.5, 1.])] {
                    points.push(p.to_array());
                    normals.push(n.to_array());
                    uv.push(tex);
                    let light = 0.7 + crate::world::noise(branch * 20 + needle) * 0.3;
                    colors.push([light * 0.84, light, light * 0.83, 1.]);
                }
            }
        }
    }
    mesh(points, normals, colors, uv)
}
#[derive(Component, Clone, Copy)]
pub struct Limb {
    pub side: f32,
    pub arm: bool,
}
pub fn human_parts(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cloth: Handle<StandardMaterial>,
    skin: Handle<StandardMaterial>,
) -> Entity {
    let leather = materials.add(StandardMaterial {
        base_color: Color::srgb(0.19, 0.12, 0.075),
        perceptual_roughness: 0.95,
        ..default()
    });
    let hair = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.13, 0.075),
        perceptual_roughness: 0.95,
        ..default()
    });
    let eyes = materials.add(StandardMaterial {
        base_color: Color::srgb(0.055, 0.045, 0.035),
        perceptual_roughness: 0.95,
        ..default()
    });
    let sphere = meshes.add(Sphere::new(1.).mesh().uv(16, 12));
    let limb = meshes.add(
        Capsule3d::new(0.075, 0.30)
            .mesh()
            .longitudes(12)
            .latitudes(8),
    );
    parent.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(skin.clone()),
        Transform::from_xyz(0., 0.69, 0.).with_scale(Vec3::new(0.17, 0.205, 0.16)),
    ));
    parent.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(hair),
        Transform::from_xyz(0., 0.79, -0.018).with_scale(Vec3::new(0.172, 0.12, 0.16)),
    ));
    parent.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(skin.clone()),
        Transform::from_xyz(0., 0.68, 0.16).with_scale(Vec3::new(0.035, 0.045, 0.045)),
    ));
    for x in [-0.065, 0.065] {
        parent.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(eyes.clone()),
            Transform::from_xyz(x, 0.73, 0.149).with_scale(Vec3::splat(0.016)),
        ));
    }
    let belt: Vec<_> = (0..49)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 48.;
            Vec3::new(a.cos() * 0.22, -0.01, a.sin() * 0.16)
        })
        .collect();
    parent.spawn((
        Mesh3d(meshes.add(tube(&belt, 0.024))),
        MeshMaterial3d(leather.clone()),
        Transform::default(),
    ));
    parent.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.075, 0.15))),
        MeshMaterial3d(skin.clone()),
        Transform::from_xyz(0., 0.48, 0.),
    ));
    let shoe = meshes.add(softened_box(Vec3::new(0.15, 0.12, 0.27), 0.045));
    let mut right_arm = None;
    for sign in [-1., 1.] {
        let arm = parent
            .spawn((
                Limb {
                    side: sign,
                    arm: true,
                },
                Transform::from_xyz(sign * 0.285, 0.34, 0.02)
                    .with_rotation(Quat::from_rotation_z(sign * 0.13)),
                Visibility::Inherited,
            ))
            .with_children(|arm| {
                arm.spawn((
                    Mesh3d(limb.clone()),
                    MeshMaterial3d(cloth.clone()),
                    Transform::from_xyz(0., -0.18, 0.),
                ));
                arm.spawn((
                    Mesh3d(sphere.clone()),
                    MeshMaterial3d(skin.clone()),
                    Transform::from_xyz(0., -0.41, 0.01).with_scale(Vec3::new(0.060, 0.080, 0.065)),
                ));
            })
            .id();
        if sign > 0. {
            right_arm = Some(arm);
        }
        parent
            .spawn((
                Limb {
                    side: sign,
                    arm: false,
                },
                Transform::from_xyz(sign * 0.12, -0.23, 0.),
                Visibility::Inherited,
            ))
            .with_children(|leg| {
                leg.spawn((
                    Mesh3d(limb.clone()),
                    MeshMaterial3d(leather.clone()),
                    Transform::from_xyz(0., -0.20, 0.),
                ));
                leg.spawn((
                    Mesh3d(shoe.clone()),
                    MeshMaterial3d(leather.clone()),
                    Transform::from_xyz(0., -0.41, 0.055),
                ));
            });
    }
    right_arm.expect("both arms are authored")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn sculpted_surfaces_have_finite_unit_normals_and_outward_winding() {
        let shapes = [
            softened_box(Vec3::ONE, 0.1),
            tunic(),
            thatch_roof(),
            boat_hull(false),
            boat_hull(true),
            boat_deck(),
            tube(&[Vec3::ZERO, Vec3::Z], 0.1),
        ];
        for (shape, mesh) in shapes.iter().enumerate() {
            let Some(VertexAttributeValues::Float32x3(points)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions")
            };
            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("normals")
            };
            assert_eq!(points.len(), normals.len());
            for (point, normal) in points.iter().zip(normals) {
                assert!(Vec3::from_array(*point).is_finite());
                assert!(
                    (Vec3::from_array(*normal).length() - 1.).abs() < 0.01,
                    "shape {shape} has bad normal {normal:?}"
                );
            }
            for (triangle, normals) in points.chunks_exact(3).zip(normals.chunks_exact(3)) {
                let a = Vec3::from_array(triangle[0]);
                let b = Vec3::from_array(triangle[1]);
                let c = Vec3::from_array(triangle[2]);
                let normal = Vec3::from_array(normals[0])
                    + Vec3::from_array(normals[1])
                    + Vec3::from_array(normals[2]);
                assert!(
                    (b - a).cross(c - a).dot(normal) > -0.0001,
                    "inverted triangle in shape {shape}"
                );
            }
            if shape == 2 || shape == 5 {
                assert!(
                    normals.iter().all(|n| n[1] > 0.),
                    "roof/deck must face upward"
                );
            }
        }
    }
}
