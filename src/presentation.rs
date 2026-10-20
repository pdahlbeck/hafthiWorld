//! Visible proof of shared entities. Presentation reads, but never advances, logistics.
use crate::{
    camera::{CameraInput, CameraMode, CameraRig},
    simulation::{INITIAL_LOGS, STOCKPILE, SUPPLY, SimulationClock, Stocks, Worker},
};
use bevy::prelude::*;

pub struct PresentationPlugin;
impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (sync_worker, sync_logs, hud).after(CameraInput));
    }
}
#[derive(Component)]
struct WorkerVisual;
#[derive(Component)]
struct LogVisual(u32);
#[derive(Component)]
struct StatusText;
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wool = materials.add(Color::srgb(0.23, 0.31, 0.21));
    let skin = materials.add(Color::srgb(0.65, 0.43, 0.30));
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.48, 1.2, 0.35))),
            MeshMaterial3d(wool),
            Transform::default(),
            WorkerVisual,
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(meshes.add(Sphere::new(0.19))),
                MeshMaterial3d(skin),
                Transform::from_xyz(0.0, 0.78, 0.0),
            ));
        });
    let log = meshes.add(Cylinder::new(0.12, 0.85));
    let wood = materials.add(Color::srgb(0.45, 0.27, 0.13));
    for id in 0..INITIAL_LOGS {
        commands.spawn((
            Mesh3d(log.clone()),
            MeshMaterial3d(wood.clone()),
            Transform::default(),
            LogVisual(id),
        ));
    }
    // Low marker pads identify endpoints without expanding the art scope.
    let pad = meshes.add(Cuboid::new(1.7, 0.06, 1.0));
    let stone = materials.add(Color::srgb(0.64, 0.62, 0.53));
    for p in [SUPPLY, STOCKPILE] {
        commands.spawn((
            Mesh3d(pad.clone()),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(p.x, 0.23, p.y),
        ));
    }
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: px(17.0).into(),
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.96, 0.85)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            top: px(20),
            ..default()
        },
        StatusText,
    ));
}
fn sync_worker(workers: Query<&Worker>, mut visuals: Query<&mut Transform, With<WorkerVisual>>) {
    let Ok(worker) = workers.single() else {
        return;
    };
    for mut t in &mut visuals {
        t.translation = Vec3::new(worker.position.x, 0.9, worker.position.y);
    }
}
fn sync_logs(
    stocks: Res<Stocks>,
    workers: Query<&Worker>,
    mut logs: Query<(&LogVisual, &mut Transform)>,
) {
    let Ok(worker) = workers.single() else {
        return;
    };
    for (log, mut t) in &mut logs {
        let carried = worker.carrying && log.0 == stocks.stored;
        t.translation = if log.0 < stocks.stored {
            Vec3::new(
                STOCKPILE.x + (log.0 % 3) as f32 * 0.40 - 0.4,
                0.43 + (log.0 / 3) as f32 * 0.23,
                STOCKPILE.y,
            )
        } else if carried {
            Vec3::new(worker.position.x, 1.15, worker.position.y + 0.32)
        } else {
            let i = log.0 - stocks.stored - u32::from(worker.carrying);
            Vec3::new(
                SUPPLY.x + (i % 3) as f32 * 0.40 - 0.4,
                0.43 + (i / 3) as f32 * 0.23,
                SUPPLY.y,
            )
        };
        t.rotation = if carried {
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
        } else {
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
        };
    }
}
fn hud(
    rig: Res<CameraRig>,
    stocks: Res<Stocks>,
    clock: Res<SimulationClock>,
    worker: Query<(Entity, &Worker)>,
    mut texts: Query<&mut Text, With<StatusText>>,
) {
    let Ok((id, w)) = worker.single() else {
        return;
    };
    let mode = match rig.mode {
        CameraMode::Strategy => "STRATEGY",
        CameraMode::FirstPerson => "FIRST PERSON",
    };
    let controls = if rig.mode == CameraMode::Strategy {
        "WASD orbit | Q/E or wheel zoom | Arrows pan | Tab walk | Esc quit"
    } else if rig.captured {
        "WASD walk | Shift run | Mouse look | Tab strategy | Esc release mouse"
    } else {
        "Click to capture mouse | Tab / Esc strategy (simulation keeps running)"
    };
    for mut text in &mut texts {
        *text = Text::new(format!(
            "hafthiWorld | {mode} | Simulation {:0.1}s\nWorker {id:?}: {:?} | Carrying: {}\nSupply: {} | Stockpile: {} / {INITIAL_LOGS}\n{controls}",
            clock.0, w.task, w.carrying, stocks.supply, stocks.stored
        ));
    }
}
