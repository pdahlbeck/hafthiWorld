use bevy::prelude::*;

const CAMERA_DISTANCE: f32 = 28.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hafthiWorld — Gotland, c. 900".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.69, 0.84, 0.94)))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
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
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.65, -0.4)),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.78, 0.85, 1.0),
        brightness: 260.0,
        affects_lightmapped_meshes: true,
    });

    let yaw: f32 = 0.5;
    let pitch: f32 = 0.50;
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_position(yaw, pitch, CAMERA_DISTANCE))
            .looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera { yaw, pitch, distance: CAMERA_DISTANCE },
    ));
}

fn camera_position(yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    Vec3::new(
        distance * pitch.cos() * yaw.sin(),
        distance * pitch.sin(),
        distance * pitch.cos() * yaw.cos(),
    )
}

fn orbit_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    let dt = time.delta_secs();
    for (mut transform, mut orbit) in &mut cameras {
        if keyboard.pressed(KeyCode::KeyA) { orbit.yaw -= dt; }
        if keyboard.pressed(KeyCode::KeyD) { orbit.yaw += dt; }
        if keyboard.pressed(KeyCode::KeyW) { orbit.pitch += dt * 0.7; }
        if keyboard.pressed(KeyCode::KeyS) { orbit.pitch -= dt * 0.7; }
        if keyboard.pressed(KeyCode::KeyQ) { orbit.distance -= dt * 12.0; }
        if keyboard.pressed(KeyCode::KeyE) { orbit.distance += dt * 12.0; }
        orbit.pitch = orbit.pitch.clamp(0.15, 1.3);
        orbit.distance = orbit.distance.clamp(6.0, 90.0);
        transform.translation = camera_position(orbit.yaw, orbit.pitch, orbit.distance);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
