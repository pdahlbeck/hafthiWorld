//! Existing coastal prototype, plus shared ground and footprint queries.
use bevy::prelude::*;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Flat placeholders establish scene scale before art-directed terrain arrives.
    let meadow = materials.add(Color::srgb(0.40, 0.58, 0.32));
    let sea = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 0.60, 0.70, 0.85),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.27,
        ..default()
    });
    let limestone = materials.add(Color::srgb(0.83, 0.81, 0.70));
    let timber = materials.add(Color::srgb(0.49, 0.29, 0.16));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(18.0, 0.4, 13.0))),
        MeshMaterial3d(meadow),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(60.0, 0.05, 60.0))),
        MeshMaterial3d(sea),
        Transform::from_xyz(0.0, -0.35, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 0.35, 1.2))),
        MeshMaterial3d(limestone),
        Transform::from_xyz(-7.0, 0.3, 1.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.5, 2.4, 5.0))),
        MeshMaterial3d(timber),
        Transform::from_xyz(2.0, 1.4, 0.0),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.65, -0.4)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.78, 0.85, 1.0),
        brightness: 260.0,
        affects_lightmapped_meshes: true,
    });
}

// Match the visible meadow, house and limestone block, with visitor clearance.
pub fn walkable(p: Vec2) -> bool {
    p.x >= -8.65
        && p.x <= 8.65
        && p.y >= -6.15
        && p.y <= 6.15
        && !(p.x > -0.10 && p.x < 4.10 && p.y.abs() < 2.85)
        && !(p.x > -8.85 && p.x < -5.15 && p.y > 0.05 && p.y < 1.95)
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
