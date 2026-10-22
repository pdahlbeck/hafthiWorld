//! Persistent scene entities read shared state; rendering never advances logistics.
use crate::{
    camera::{CameraMode, CameraRig},
    farming::{GrainLocation, SEED_UNITS, SeedLocation},
    livestock::{SHEEP_COUNT, WoolLocation},
    orders::{ORDER_PANEL_BOTTOM, ORDER_PANEL_HEIGHT, OrderButton, OrderInput, OrderState},
    simulation::{
        BUILD_COST, BUILD_SITE, LogLocation, STOCKPILE, Settlement, SimulationClock, TOTAL_LOGS,
        TREE_POSITIONS, Task, Worker, ground_log_position,
    },
    trade::SaltLocation,
};
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{ActivateOnPress, Button},
};

pub struct PresentationPlugin;
impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                sync_worker,
                sync_logs,
                sync_trees,
                sync_construction,
                sync_selection,
                sync_buttons,
                hud,
            )
                .after(OrderInput),
        );
    }
}
#[derive(Component)]
struct WorkerVisual;
#[derive(Component)]
struct AxeVisual;
#[derive(Component)]
struct LogVisual(usize);
#[derive(Component)]
struct TreeVisual(usize);
#[derive(Component)]
struct ConstructionVisual(usize);
#[derive(Component)]
struct SelectionVisual;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct OrderPanel;
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wool = materials.add(Color::srgb(0.23, 0.31, 0.21));
    let skin = materials.add(Color::srgb(0.65, 0.43, 0.30));
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.27, 0.13),
        perceptual_roughness: 0.9,
        ..default()
    });
    let steel = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.34, 0.33),
        metallic: 0.7,
        perceptual_roughness: 0.6,
        ..default()
    });
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
            p.spawn((
                Transform::from_xyz(0.34, 0.3, 0.1),
                Visibility::default(),
                AxeVisual,
            ))
            .with_children(|axe| {
                axe.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.035, 0.45, 0.035))),
                    MeshMaterial3d(wood.clone()),
                    Transform::default(),
                ));
                axe.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.19, 0.12, 0.06))),
                    MeshMaterial3d(steel),
                    Transform::from_xyz(0.07, 0.20, 0.0),
                ));
            });
        });
    let leaf = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.37, 0.16),
        perceptual_roughness: 1.0,
        ..default()
    });
    let trunk = meshes.add(Cylinder::new(0.12, 1.8));
    let crown = meshes.add(Sphere::new(0.85).mesh().ico(2).expect("valid crown mesh"));
    let stump = meshes.add(Cylinder::new(0.14, 0.30));
    for (i, p) in TREE_POSITIONS.iter().enumerate() {
        commands
            .spawn((
                Mesh3d(trunk.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(p.x, 1.1, p.y),
                TreeVisual(i),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(crown.clone()),
                    MeshMaterial3d(leaf.clone()),
                    Transform::from_xyz(0.0, 1.15, 0.0),
                ));
            });
        commands.spawn((
            Mesh3d(stump.clone()),
            MeshMaterial3d(wood.clone()),
            Transform::from_xyz(p.x, 0.35, p.y),
        ));
    }
    let log = meshes.add(Cylinder::new(0.12, 0.85));
    for i in 0..TOTAL_LOGS {
        commands.spawn((
            Mesh3d(log.clone()),
            MeshMaterial3d(wood.clone()),
            Transform::default(),
            Visibility::Hidden,
            LogVisual(i),
        ));
    }
    let gold = materials.add(Color::srgb(0.82, 0.65, 0.24));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.55, 0.025))),
        MeshMaterial3d(gold),
        Transform::default(),
        SelectionVisual,
    ));
    let pad = meshes.add(Cuboid::new(1.7, 0.06, 1.0));
    let stone = materials.add(Color::srgb(0.64, 0.62, 0.53));
    for p in [STOCKPILE, BUILD_SITE] {
        commands.spawn((
            Mesh3d(pad.clone()),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(p.x, 0.23, p.y),
        ));
    }
    let parts = [
        (Vec3::new(-0.58, 0.85, 0.0), Vec3::new(0.15, 1.2, 0.15)),
        (Vec3::new(0.58, 0.85, 0.0), Vec3::new(0.15, 1.2, 0.15)),
        (Vec3::new(0.0, 1.48, 0.0), Vec3::new(1.45, 0.15, 0.22)),
    ];
    for (i, (offset, size)) in parts.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(wood.clone()),
            Transform::from_translation(Vec3::new(BUILD_SITE.x, 0.0, BUILD_SITE.y) + offset),
            Visibility::Hidden,
            ConstructionVisual(i),
        ));
    }
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: px(16.0).into(),
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.96, 0.85)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            right: px(20),
            top: px(20),
            ..default()
        },
        StatusText,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(20),
                right: px(20),
                bottom: px(ORDER_PANEL_BOTTOM),
                height: px(ORDER_PANEL_HEIGHT),
                flex_wrap: FlexWrap::Wrap,
                row_gap: px(8),
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            },
            OrderPanel,
        ))
        .with_children(|panel| {
            for (label, order) in [
                ("1: Tree 1", OrderButton::Select(0)),
                ("2: Tree 2", OrderButton::Select(1)),
                ("3: Tree 3", OrderButton::Select(2)),
                ("F: Harvest", OrderButton::Harvest),
                ("B: Build rack (2 logs)", OrderButton::Build),
                ("O: Sow field", OrderButton::Sow),
                ("R: Reap crop", OrderButton::Reap),
                ("C: Shear sheep", OrderButton::Shear),
                ("H: Supply home", OrderButton::SupplyHome),
                ("V: Save seed", OrderButton::ReserveSeed),
                ("J: Barter wool/salt", OrderButton::Barter),
                ("K: Supply firewood", OrderButton::SupplyFuel),
            ] {
                panel
                    .spawn((
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        order,
                        Node {
                            height: px(44),
                            padding: UiRect::horizontal(px(14)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.22, 0.19, 0.14)),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: px(16.0).into(),
                                ..default()
                            },
                            TextColor(Color::srgb(0.98, 0.96, 0.85)),
                        ));
                    });
            }
        });
}
type AxeTransforms<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<AxeVisual>, Without<WorkerVisual>),
>;
fn sync_worker(
    workers: Query<&Worker>,
    clock: Res<SimulationClock>,
    mut visuals: Query<&mut Transform, With<WorkerVisual>>,
    mut axes: AxeTransforms,
) {
    let Ok(w) = workers.single() else {
        return;
    };
    for mut t in &mut visuals {
        let previous = Vec2::new(t.translation.x, t.translation.z);
        let direction = w.position - previous;
        if direction.length_squared() > 0.00001 {
            t.rotation = Quat::from_rotation_y(direction.x.atan2(direction.y));
        }
        t.translation = Vec3::new(w.position.x, 0.9, w.position.y);
    }
    for (mut axe, mut visible) in &mut axes {
        *visible = if matches!(
            w.task,
            Task::ToFuelStore
                | Task::LoadingFuel
                | Task::ToFuelHome
                | Task::UnloadingFuel
                | Task::ToFoodStore
                | Task::LoadingFood
                | Task::ToHome
                | Task::UnloadingFood
                | Task::FromHome
        ) || w.task.tending_sheep()
            || w.task.trading()
            || matches!(
                w.task,
                Task::ToSeedStore
                    | Task::PreparingSeeds
                    | Task::ToFarmGateway
                    | Task::LoadingSeeds
                    | Task::ToField
                    | Task::Sowing
                    | Task::Reaping
                    | Task::LoadingGrain
                    | Task::DeliveringGrain
                    | Task::StoringGrain
                    | Task::FromField
            ) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        axe.rotation = Quat::from_rotation_z(if w.task == Task::Felling {
            (clock.0 as f32 * 9.0).sin() * 0.8
        } else {
            0.0
        });
    }
}
fn sync_logs(
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    mut logs: Query<(&LogVisual, &mut Transform, &mut Visibility)>,
) {
    let Ok(worker) = workers.single() else {
        return;
    };
    for (log, mut t, mut visibility) in &mut logs {
        let state = settlement.logs[log.0];
        *visibility = if matches!(
            state,
            LogLocation::InTree | LogLocation::Installed | LogLocation::Burned(_)
        ) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        t.translation = match state {
            LogLocation::Home(_) => crate::hearth::log_position(log.0),
            LogLocation::Ground => {
                let p = ground_log_position(log.0);
                Vec3::new(p.x, 0.38, p.y)
            }
            LogLocation::Carried => Vec3::new(worker.position.x, 1.15, worker.position.y + 0.32),
            LogLocation::Stored => {
                let slot = log.0;
                Vec3::new(
                    STOCKPILE.x + (slot % 3) as f32 * 0.4 - 0.4,
                    0.43 + (slot / 3) as f32 * 0.23,
                    STOCKPILE.y,
                )
            }
            _ => Vec3::ZERO,
        };
        t.rotation = if state == LogLocation::Carried {
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
        } else {
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
        };
    }
}
fn sync_trees(settlement: Res<Settlement>, mut trees: Query<(&TreeVisual, &mut Visibility)>) {
    for (tree, mut v) in &mut trees {
        *v = if settlement.trees[tree.0].felled {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
fn sync_construction(
    settlement: Res<Settlement>,
    mut parts: Query<(&ConstructionVisual, &mut Visibility)>,
) {
    for (part, mut v) in &mut parts {
        *v = if (part.0 < BUILD_COST && settlement.installed() > part.0)
            || settlement.construction.complete
        {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
fn sync_selection(
    orders: Res<OrderState>,
    mut visual: Query<&mut Transform, With<SelectionVisual>>,
) {
    let p = TREE_POSITIONS[orders.selected];
    for mut t in &mut visual {
        t.translation = Vec3::new(p.x, 0.23, p.y);
    }
}
fn sync_buttons(
    orders: Res<OrderState>,
    rig: Res<CameraRig>,
    mut panels: Query<&mut Node, With<OrderPanel>>,
    mut buttons: Query<(&OrderButton, &Hovered, Has<Pressed>, &mut BackgroundColor)>,
) {
    for mut node in &mut panels {
        node.display = if rig.mode == CameraMode::Strategy {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (action, hovered, pressed, mut color) in &mut buttons {
        let selected = matches!(action,OrderButton::Select(i) if *i==orders.selected);
        *color = BackgroundColor(if pressed {
            Color::srgb(0.5, 0.37, 0.16)
        } else if hovered.get() || selected {
            Color::srgb(0.36, 0.30, 0.17)
        } else {
            Color::srgb(0.22, 0.19, 0.14)
        });
    }
}
fn hud(
    rig: Res<CameraRig>,
    s: Res<Settlement>,
    orders: Res<OrderState>,
    clock: Res<SimulationClock>,
    workers: Query<(Entity, &Worker)>,
    mut texts: Query<&mut Text, With<StatusText>>,
) {
    let Ok((id, w)) = workers.single() else {
        return;
    };
    let mode = if rig.mode == CameraMode::Strategy {
        "STRATEGY"
    } else {
        "FIRST PERSON"
    };
    let tree = &s.trees[orders.selected];
    let tree_status = if tree.felled {
        "felled"
    } else if tree.ordered {
        "ordered"
    } else {
        "standing"
    };
    let build = if s.construction.complete {
        "complete"
    } else if s.installed() == BUILD_COST {
        "assembling"
    } else if s.construction.ordered {
        "waiting for delivered timber"
    } else {
        "not ordered"
    };
    let controls = if rig.mode == CameraMode::Strategy {
        "WASD orbit | Q/E zoom | Arrows pan | Click tree / 1-3 select | F harvest | B build | O sow | R reap | C shear | H home | V seeds | J barter | K fuel | Tab walk | Esc quit"
    } else if rig.captured {
        "WASD walk | Mouse look | 1-3 select | F harvest | B build | O sow | R reap | C shear | H home | V seeds | J barter | K fuel | Tab strategy | Esc release mouse"
    } else {
        "Click to capture mouse | Tab / Esc strategy (simulation keeps running)"
    };
    for mut text in &mut texts {
        *text = Text::new(format!(
            "hafthiWorld | {mode} | Simulation {:.1}s | Year {} {} ({:.0}s left) | {}\nWorker {id:?} (home {}): {:?} | Carrying log: {} | Tree {}: {tree_status}\nLogs: in trees {} | ground {} | carried {} | stored {} | installed {}/{}\nTimber rack: {build}\nCrop: {} {:.0}% | seeds stored {} / carried {} / used {}\nGrain: field {} | carried {} | stored {} | harvests {} | seed batches saved {} ({})\nSheep shorn {}/{} | Wool: fleece {} | ground {} | carried {} | stored {} | trader {}\nTrade 2 wool : 1 salt ({}): offered {} | paid {} | carried {} | stored {}\nHome {}: {} | pantry {} | food used {} | missed {} | next use {:.0}s\nWinter fuel: {} | at home {} / {} | burned {} | missed {} | next use {:.0} winter s\n{}\n{controls}",
            clock.0,
            s.calendar.year,
            s.calendar.season().label(),
            s.calendar.remaining(),
            if s.calendar.can_sow() {
                "sowing open"
            } else {
                "new sowing waits for spring"
            },
            w.home_id,
            w.task,
            w.carrying.is_some(),
            orders.selected + 1,
            s.count(LogLocation::InTree),
            s.count(LogLocation::Ground),
            s.count(LogLocation::Carried),
            s.stored(),
            s.installed(),
            BUILD_COST,
            s.farm.phase.label(),
            s.farm.progress() * 100.0,
            if s.farm.seeds == SeedLocation::Stored {
                SEED_UNITS
            } else {
                0
            },
            if s.farm.seeds == SeedLocation::Carried {
                SEED_UNITS
            } else {
                0
            },
            if s.farm.seeds == SeedLocation::Sown {
                SEED_UNITS
            } else {
                0
            },
            s.farm.count(GrainLocation::Ground),
            s.farm.count(GrainLocation::Carried),
            s.farm.count(GrainLocation::Stored),
            s.farm.harvests,
            s.farm.seed_batches_saved,
            if s.farm.reserve_ordered {
                "saving on"
            } else {
                "V to save"
            },
            s.flock.sheep.iter().filter(|s| s.shorn).count(),
            SHEEP_COUNT,
            s.flock.count(WoolLocation::Fleece),
            s.flock.count(WoolLocation::Ground),
            s.flock.count(WoolLocation::Carried),
            s.flock.count(WoolLocation::Stored),
            s.flock.count(WoolLocation::Trader),
            if s.barter.ordered {
                "ordered"
            } else {
                "J to order"
            },
            s.barter.count(SaltLocation::Offered),
            s.barter.count(SaltLocation::Paid),
            s.barter.count(SaltLocation::Carried),
            s.barter.count(SaltLocation::Stored),
            s.household.id,
            s.household
                .status(s.farm.count(GrainLocation::Home(s.household.id))),
            s.farm.count(GrainLocation::Home(s.household.id)),
            s.household.food_used,
            s.household.missed_rations,
            s.household.next_use(),
            s.household.hearth.status(
                s.calendar.season(),
                s.count(LogLocation::Home(s.household.id))
            ),
            s.count(LogLocation::Home(s.household.id)),
            crate::hearth::FUEL_TARGET,
            s.household.hearth.burned,
            s.household.hearth.missed,
            s.household.hearth.next_use(),
            orders.feedback
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn firewood_entities_persist_through_delivery_camera_changes_and_two_winters() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .init_resource::<OrderState>()
            .add_plugins((
                crate::simulation::SimulationPlugin,
                PresentationPlugin,
                crate::hearth::HearthPlugin,
            ));
        app.update();
        let logs: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<LogVisual>>()
            .iter(app.world())
            .collect();
        assert_eq!(logs.len(), TOTAL_LOGS);
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        let entity_count = app.world().entities().len();
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.logs.fill(LogLocation::Stored); // Controlled initial stock for the rendering integration test.
            s.calendar.advance(360.0);
            s.household.hearth.request();
        }
        let mut delivered = false;
        for frame in 0..6500 {
            if frame % 11 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            let s = app.world().resource::<Settlement>();
            for &id in &logs {
                let slot = app.world().get::<LogVisual>(id).unwrap().0;
                if s.logs[slot] == LogLocation::Home(s.household.id) {
                    delivered = true;
                    assert_eq!(
                        app.world().get::<Transform>(id).unwrap().translation,
                        crate::hearth::log_position(slot)
                    );
                    assert_ne!(
                        *app.world().get::<Visibility>(id).unwrap(),
                        Visibility::Hidden
                    );
                }
            }
        }
        assert!(delivered && app.world().get::<Worker>(worker).is_some());
        assert_eq!(app.world().entities().len(), entity_count);
        assert_eq!(
            app.world().resource::<Settlement>().household.hearth.burned as usize,
            TOTAL_LOGS
        );
        for id in logs {
            assert_eq!(
                *app.world().get::<Visibility>(id).unwrap(),
                Visibility::Hidden
            );
        }
    }
    #[test]
    fn scene_keeps_the_same_worker_tree_and_log_entities_through_harvest_and_build() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .init_resource::<OrderState>()
            .add_plugins((crate::simulation::SimulationPlugin, PresentationPlugin));
        app.update();
        let log_ids: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<LogVisual>>()
            .iter(app.world())
            .collect();
        let tree_ids: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<TreeVisual>>()
            .iter(app.world())
            .collect();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<WorkerVisual>>()
            .single(app.world())
            .unwrap();
        assert_eq!(log_ids.len(), TOTAL_LOGS);
        assert!(
            log_ids
                .iter()
                .all(|&id| *app.world().get::<Visibility>(id).unwrap() == Visibility::Hidden)
        );
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.request_harvest(0);
            s.request_build();
        }
        for frame in 0..900 {
            if frame % 9 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
        }
        assert!(app.world().resource::<Settlement>().construction.complete);
        assert!(app.world().get::<WorkerVisual>(worker).is_some());
        for id in log_ids {
            assert!(app.world().get::<LogVisual>(id).is_some());
        }
        for id in tree_ids {
            assert!(app.world().get::<TreeVisual>(id).is_some());
        }
        let completed_parts = app
            .world_mut()
            .query::<(&ConstructionVisual, &Visibility)>()
            .iter(app.world())
            .filter(|(_, v)| **v != Visibility::Hidden)
            .count();
        assert_eq!(completed_parts, 3);
    }
}
