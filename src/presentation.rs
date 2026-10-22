//! Persistent scene entities read shared state; rendering never advances logistics.
use crate::{
    camera::{CameraMode, CameraRig},
    farming::{GrainLocation, SEED_UNITS, SeedLocation},
    fishing::FishLocation as Fish,
    livestock::{SHEEP_COUNT, WoolLocation},
    orders::{ORDER_PANEL_BOTTOM, ORDER_PANEL_HEIGHT, OrderButton, OrderInput, OrderState},
    simulation::{
        BUILD_COST, BUILD_SITE, LogLocation, STOCKPILE, Settlement, SimulationClock, TOTAL_LOGS,
        TREE_POSITIONS, Task, Worker, WorkerRole, ground_log_position,
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
        app.init_resource::<crate::inspection::Inspection>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    spawn_helper_visual,
                    (sync_worker, animate_residents).chain(),
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
pub(crate) struct WorkerVisual(pub(crate) u32);
#[derive(Component)]
pub(crate) struct AxeVisual;
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
    let wool = materials.add(StandardMaterial {
        base_color: Color::srgb(0.23, 0.31, 0.21),
        perceptual_roughness: 0.95,
        ..default()
    });
    let skin = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.43, 0.30),
        perceptual_roughness: 0.95,
        ..default()
    });
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
            Mesh3d(meshes.add(crate::art::tunic())),
            MeshMaterial3d(wool.clone()),
            Transform::default(),
            WorkerVisual(1),
        ))
        .with_children(|p| {
            let hand = crate::art::human_parts(p, &mut meshes, &mut materials, wool, skin);
            p.commands().entity(hand).with_children(|hand| {
                hand.spawn((
                    Transform::from_xyz(0.0, -0.25, 0.03),
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
        });
    let leaf = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.36, 0.25),
        cull_mode: None,
        perceptual_roughness: 1.0,
        ..default()
    });
    let trunk = meshes.add(Cone::new(0.14, 3.3));
    let crown = meshes.add(crate::art::foliage());
    let stump = meshes.add(Cylinder::new(0.14, 0.30));
    for (i, p) in TREE_POSITIONS.iter().enumerate() {
        commands
            .spawn((
                Mesh3d(trunk.clone()),
                MeshMaterial3d(wood.clone()),
                Transform::from_xyz(p.x, 1.85, p.y),
                TreeVisual(i),
            ))
            .with_children(|parent| {
                for layer in 0..6 {
                    let h = -0.35 + layer as f32 * 0.37;
                    let width = 1.0 - layer as f32 * 0.13;
                    for bough in 0..3 {
                        let angle = bough as f32 * 2.0944 + layer as f32 * 0.65;
                        parent.spawn((
                            Mesh3d(crown.clone()),
                            MeshMaterial3d(leaf.clone()),
                            Transform::from_xyz(
                                angle.cos() * width * 0.2,
                                h,
                                angle.sin() * width * 0.2,
                            )
                            .with_rotation(Quat::from_rotation_y(angle))
                            .with_scale(Vec3::new(width, 0.9, width)),
                        ));
                    }
                }
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
            Name::new(format!("Log {}", i)),
        ));
    }
    let gold = materials.add(Color::srgb(0.82, 0.65, 0.24));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.55, 0.025))),
        MeshMaterial3d(gold),
        Transform::default(),
        SelectionVisual,
    ));
    let pad = meshes.add(crate::art::softened_box(Vec3::new(1.7, 0.06, 1.0), 0.02));
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
            Mesh3d(meshes.add(crate::art::softened_box(size, 0.025))),
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
            right: px(360),
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
                ("J: Wool/salt", OrderButton::Barter),
                ("K: Firewood", OrderButton::SupplyFuel),
                ("P: Fish", OrderButton::Fish),
                ("L: Salt fish", OrderButton::SaltFish),
                ("N: Helper", OrderButton::InviteHelper),
                ("U: Boat", OrderButton::CollectWool),
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
fn spawn_helper_visual(
    mut commands: Commands,
    workers: Query<&Worker>,
    visuals: Query<(Entity, &WorkerVisual)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(helper) = workers
        .iter()
        .find(|w| w.id == 2 && w.role == WorkerRole::Coastal)
    else {
        for (entity, visual) in &visuals {
            if visual.0 == 2 {
                commands.entity(entity).despawn();
            }
        }
        return;
    };
    if visuals.iter().any(|(_, v)| v.0 == 2) {
        return;
    }
    let tunic = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.37, 0.48),
        perceptual_roughness: 0.95,
        ..default()
    });
    let skin = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.43, 0.30),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands
        .spawn((
            WorkerVisual(2),
            Name::new("Resident 2 coastal worker"),
            Mesh3d(meshes.add(crate::art::tunic())),
            MeshMaterial3d(tunic.clone()),
            Transform::from_xyz(helper.position.x, 0.9, helper.position.y),
        ))
        .with_children(|p| {
            crate::art::human_parts(p, &mut meshes, &mut materials, tunic, skin);
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
    mut visuals: Query<(&WorkerVisual, &mut Transform)>,
    mut axes: AxeTransforms,
) {
    for (visual, mut t) in &mut visuals {
        let Some(w) = workers.iter().find(|w| w.id == visual.0) else {
            continue;
        };
        t.rotation = Quat::from_rotation_y(w.facing);
        t.translation = Vec3::new(w.position.x, 0.9, w.position.y);
    }
    let Some(w) = workers.iter().find(|w| w.id == 1) else {
        return;
    };
    for (mut axe, mut visible) in &mut axes {
        *visible = if w.carrying.is_some()
            || w.carrying_grain.is_some()
            || w.carrying_wool.is_some()
            || w.carrying_salt.is_some()
            || w.carrying_fish.is_some()
            || matches!(
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
            )
            || w.task.tending_sheep()
            || w.task.trading()
            || matches!(w.task, Task::Coastal(_))
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
            (clock.0 as f32 * 9.0).sin() * 0.15
        } else {
            0.0
        });
    }
}
fn animate_residents(
    clock: Res<SimulationClock>,
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    parents: Query<&WorkerVisual>,
    mut limbs: Query<(&crate::art::Limb, &ChildOf, &mut Transform)>,
) {
    for (limb, parent, mut t) in &mut limbs {
        let Ok(visual) = parents.get(parent.parent()) else {
            continue;
        };
        let Some(worker) = workers.iter().find(|w| w.id == visual.0) else {
            continue;
        };
        let pose = crate::motion::pose(worker, &settlement, clock.0, limb.side, limb.arm);
        t.rotation = pose.0;
        t.scale = pose.1;
    }
}

fn sync_logs(
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    mut logs: Query<(&LogVisual, &mut Transform, &mut Visibility)>,
) {
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
            LogLocation::Carried => {
                let Some(worker) = workers.iter().find(|w| w.carrying == Some(log.0)) else {
                    *visibility = Visibility::Hidden;
                    continue;
                };
                Vec3::new(worker.position.x, 1.15, worker.position.y + 0.32)
            }
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
    inspection: Res<crate::inspection::Inspection>,
    s: Res<Settlement>,
    orders: Res<OrderState>,
    clock: Res<SimulationClock>,
    workers: Query<(Entity, &Worker)>,
    mut texts: Query<(&mut Text, &mut TextFont), With<StatusText>>,
) {
    let Some((id, w)) = workers.iter().find(|(_, w)| w.id == 1) else {
        return;
    };
    if !inspection.details {
        for (mut text, mut font) in &mut texts {
            font.font_size = px(16).into();
            text.0 = format!(
                "hafthiWorld | {} | Year {} {} | {:.1}s\nResidents {} | Food at home {} | Timber stored {} | Tree {}\nTab: views | I: inspect | F2: details | F3: timing\n{}",
                if rig.mode == CameraMode::Strategy {
                    "STRATEGY"
                } else {
                    "FIRST PERSON"
                },
                s.calendar.year,
                s.calendar.season().label(),
                clock.0,
                workers.iter().count(),
                s.farm.count(GrainLocation::Home(s.household.id))
                    + s.fishery.count(Fish::Home(s.household.id)),
                s.stored(),
                orders.selected + 1,
                orders.feedback
            );
        }
        return;
    }
    let crew =
        if let Some((_, helper)) = workers.iter().find(|(_, w)| w.role == WorkerRole::Coastal) {
            format!(
                "Crew 2 | Resident 2 coastal: {:?} | coastal job held by {}",
                helper.task,
                s.coastal_owner
                    .map_or("none".to_owned(), |id| format!("resident {id}"))
            )
        } else {
            "Crew 1 | Resident 1 handles all work | N: invite coastal helper".to_owned()
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
        "WASD orbit | Q/E zoom | Arrows pan | Click tree / 1-3 select | F harvest | B build | O sow | R reap | C shear | H home | V seeds | J barter | K fuel | P fish | L salt | N helper | U boat | Tab walk | Esc quit"
    } else if rig.captured {
        "WASD walk | Mouse look | 1-3 select | F harvest | B build | O sow | R reap | C shear | H home | V seeds | J barter | K fuel | P fish | L salt | N helper | U boat | Tab strategy | Esc release mouse"
    } else {
        "Click to capture mouse | Tab / Esc strategy (simulation keeps running)"
    };
    for (mut text, mut font) in &mut texts {
        font.font_size = px(14).into();
        *text = Text::new(format!(
            "hafthiWorld | {mode} | Simulation {:.1}s | Year {} {} ({:.0}s left) | {}\nWorker {id:?} (home {}): {:?} | Carrying log: {} | Tree {}: {tree_status}\n{crew}\nLogs: in trees {} | ground {} | carried {} | stored {} | installed {}/{}\nTimber rack: {build}\nCrop: {} {:.0}% | seeds stored {} / carried {} / used {}\nGrain: field {} | carried {} | stored {} | harvests {} | seed batches saved {} ({})\nSheep shorn {}/{} | Wool: fleece {} | ground {} | carried {} | stored {} | trader {}\nTrade 2 wool : 1 salt ({}): offered {} | paid {} | carried {} | stored {} | table {} | used {}\nFish: uncaught {} | raw {} | processing {} | preserved {} | carried {} | home {} | eaten {}\nHome {}: {} | pantry grain {} / fish {} | food used {} (fish {}) | missed {} | next use {:.0}s\nWinter fuel: {} | at home {} / {} | burned {} | missed {} | next use {:.0} winter s\n{}\n{controls}",
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
            s.barter.count(SaltLocation::AtSalting),
            s.barter.count(SaltLocation::Consumed),
            s.fishery.count(Fish::Uncaught),
            s.fishery.count(Fish::FreshAtShore) + s.fishery.count(Fish::RawStored),
            s.fishery.count(Fish::RawAtTable),
            s.fishery.count(Fish::PreservedAtTable) + s.fishery.count(Fish::PreservedStored),
            s.fishery.count(Fish::CarriedRaw) + s.fishery.count(Fish::CarriedPreserved),
            s.fishery.count(Fish::Home(s.household.id)),
            s.fishery.count(Fish::Eaten(s.household.id)),
            s.household.id,
            s.household.status(
                s.farm.count(GrainLocation::Home(s.household.id))
                    + s.fishery.count(Fish::Home(s.household.id))
            ),
            s.farm.count(GrainLocation::Home(s.household.id)),
            s.fishery.count(Fish::Home(s.household.id)),
            s.household.food_used,
            s.household.fish_used,
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
    fn crew_app() -> App {
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
                crate::farming::FarmingPlugin,
                crate::fishing::FishingPlugin,
                crate::livestock::LivestockPlugin,
                crate::trade::TradePlugin,
                crate::household::HouseholdPlugin,
                crate::hearth::HearthPlugin,
            ));
        app.update();
        app
    }
    #[test]
    fn actual_hauler_limbs_pause_and_restore_with_saved_heading_and_clock() {
        let mut app = crew_app();
        let worker = app
            .world_mut()
            .query::<(Entity, &Worker)>()
            .iter(app.world())
            .find(|(_, w)| w.id == 1)
            .unwrap()
            .0;
        app.world_mut()
            .resource_mut::<Settlement>()
            .request_harvest(0);
        let mut saw_walking = false;
        for tick in 0..600 {
            if tick % 7 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            let w = app.world().get::<Worker>(worker).unwrap();
            saw_walking |= crate::motion::walking(w.task);
            if w.carrying.is_some() {
                break;
            }
        }
        let saved_worker = app.world().get::<Worker>(worker).unwrap().clone();
        assert!(saw_walking && saved_worker.carrying.is_some());
        let saved_world = app.world().resource::<Settlement>().clone();
        let saved_clock = app.world().resource::<SimulationClock>().0;
        app.world_mut()
            .resource_mut::<crate::session::Session>()
            .paused = true;
        app.update();
        let pose: Vec<_> = app
            .world_mut()
            .query::<(Entity, &crate::art::Limb, &Transform, &ChildOf)>()
            .iter(app.world())
            .filter(|(_, _, _, parent)| {
                app.world()
                    .get::<WorkerVisual>(parent.parent())
                    .is_some_and(|visual| visual.0 == 1)
            })
            .map(|(id, _, t, _)| (id, *t))
            .collect();
        assert_eq!(pose.len(), 4);
        for _ in 0..20 {
            app.world_mut().resource_mut::<CameraRig>().toggle();
            app.update();
        }
        for (id, t) in &pose {
            assert_eq!(*app.world().get::<Transform>(*id).unwrap(), *t);
        }
        app.world_mut()
            .resource_mut::<crate::session::Session>()
            .paused = false;
        for _ in 0..120 {
            app.update();
        }
        app.world_mut()
            .resource_mut::<crate::session::Session>()
            .paused = true;
        let encoded = serde_json::to_vec(&saved_worker).unwrap();
        app.world_mut()
            .entity_mut(worker)
            .insert(serde_json::from_slice::<Worker>(&encoded).unwrap());
        app.world_mut().insert_resource(saved_world);
        app.world_mut().resource_mut::<SimulationClock>().0 = saved_clock;
        app.update();
        for (id, t) in &pose {
            assert_eq!(*app.world().get::<Transform>(*id).unwrap(), *t);
        }
        let body = app
            .world_mut()
            .query::<(&WorkerVisual, &Transform)>()
            .iter(app.world())
            .find(|(v, _)| v.0 == 1)
            .unwrap()
            .1;
        assert_eq!(body.rotation, Quat::from_rotation_y(saved_worker.facing));
    }
    #[test]
    fn two_residents_work_in_parallel_with_real_sources_correct_cargo_and_persistent_views() {
        use crate::{
            fishing::{FISH_UNITS, FishLocation},
            simulation::COASTAL_REST,
        };
        let mut app = crew_app();
        let original_scene: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<Mesh3d>>()
            .iter(app.world())
            .collect();
        let mut recruited_entities = Vec::new();
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            assert!(s.request_helper());
            assert!(!s.request_helper());
            s.household.hearth.request();
            s.household.request_supply();
            s.farm.request_reserve();
            s.flock.request_shear();
            s.barter.request();
            s.fishery.request_fishing();
            s.fishery.request_salting();
            s.request_build();
            for i in 0..3 {
                s.request_harvest(i);
            }
        }
        let mut worked_together = false;
        let mut cargo_seen = [false; 5];
        for frame in 0..10000 {
            {
                let mut s = app.world_mut().resource_mut::<Settlement>();
                if s.farm.harvests < 2 {
                    if matches!(
                        s.farm.phase,
                        crate::farming::CropPhase::Fallow | crate::farming::CropPhase::Harvested
                    ) {
                        s.farm.request_sow();
                    }
                    s.farm.request_reap();
                }
            }
            if frame % 13 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            if frame == 2 {
                recruited_entities = app
                    .world_mut()
                    .query::<Entity>()
                    .iter(app.world())
                    .collect();
            }
            let workers: Vec<_> = app
                .world_mut()
                .query::<&Worker>()
                .iter(app.world())
                .map(|w| {
                    (
                        w.id,
                        w.position,
                        w.task,
                        w.carrying,
                        w.carrying_grain,
                        w.carrying_wool,
                        w.carrying_salt,
                        w.carrying_fish,
                    )
                })
                .collect();
            assert_eq!(workers.len(), 2);
            let a = workers.iter().find(|w| w.0 == 1).unwrap();
            let b = workers.iter().find(|w| w.0 == 2).unwrap();
            worked_together |= a.2 == Task::Felling && matches!(b.2, Task::Coastal(_));
            assert!(!matches!(a.2, Task::Coastal(_)));
            let names: Vec<_> = app
                .world_mut()
                .query::<(&Name, &Transform)>()
                .iter(app.world())
                .map(|(n, t)| (n.as_str().to_owned(), t.translation))
                .collect();
            for w in &workers {
                assert!(crate::world::walkable(w.1));
                let cargo = [w.3, w.4, w.5, w.6, w.7];
                assert!(cargo.iter().flatten().count() <= 1);
                for (kind, slot) in cargo.iter().enumerate() {
                    if let Some(slot) = slot {
                        cargo_seen[kind] = true;
                        let name = match kind {
                            0 => format!("Log {slot}"),
                            1 => format!("Grain {slot}"),
                            2 => format!("Wool {slot}"),
                            3 => format!("Salt pouch {}", slot + 1),
                            _ => format!("Fish {slot}"),
                        };
                        let visual = names
                            .iter()
                            .find(|(n, _)| n == &name)
                            .expect("persistent cargo visual");
                        let expected = if kind == 0 {
                            Vec3::new(w.1.x, 1.15, w.1.y + 0.32)
                        } else {
                            Vec3::new(w.1.x + 0.3, 1.0, w.1.y)
                        };
                        assert!(
                            visual.1.distance(expected) < 0.001,
                            "{name} detached from resident {}",
                            w.0
                        );
                    }
                }
            }
            for (visual, transform) in app
                .world_mut()
                .query::<(&WorkerVisual, &Transform)>()
                .iter(app.world())
            {
                let w = workers.iter().find(|w| w.0 == visual.0).unwrap();
                assert_eq!(transform.translation, Vec3::new(w.1.x, 0.9, w.1.y));
            }
            let s = app.world().resource::<Settlement>();
            assert_eq!(
                s.fishery.count(FishLocation::CarriedRaw)
                    + s.fishery.count(FishLocation::CarriedPreserved),
                workers.iter().filter(|w| w.7.is_some()).count()
            );
            assert_eq!(
                s.barter.count(SaltLocation::Carried),
                workers.iter().filter(|w| w.6.is_some()).count()
            );
            assert_eq!(
                s.count(LogLocation::Carried),
                workers.iter().filter(|w| w.3.is_some()).count()
            );
            let grain = s
                .farm
                .grain
                .iter()
                .filter(|g| {
                    matches!(
                        g,
                        GrainLocation::Ground
                            | GrainLocation::Carried
                            | GrainLocation::Stored
                            | GrainLocation::Home(_)
                    )
                })
                .count();
            assert_eq!(
                4 * s.farm.harvests,
                grain as u64 + s.household.food_used - s.household.fish_used
                    + s.farm.seed_batches_saved
            );
            assert_eq!(
                s.fishery.batches as usize,
                s.barter.count(SaltLocation::Consumed)
            );
            assert_eq!(
                s.fishery.count(FishLocation::Eaten(1)) as u64,
                s.household.fish_used
            );
            assert!(s.fishery.count(FishLocation::Eaten(1)) <= FISH_UNITS);
        }
        assert!(worked_together);
        assert!(cargo_seen.into_iter().all(|seen| seen));
        let s = app.world().resource::<Settlement>();
        assert_eq!(s.household.food_used, 10);
        assert_eq!(s.household.fish_used, 4);
        assert_eq!(s.installed(), BUILD_COST);
        assert_eq!(s.household.hearth.burned, 4);
        assert_eq!(s.farm.harvests, 2);
        assert_eq!(s.fishery.batches, 2);
        assert_eq!(s.flock.count(WoolLocation::Trader), 4);
        assert_eq!(s.barter.count(SaltLocation::Consumed), 2);
        assert!(
            original_scene
                .iter()
                .all(|&e| app.world().get_entity(e).is_ok())
        );
        assert!(
            recruited_entities
                .iter()
                .all(|&e| app.world().get_entity(e).is_ok())
        );
        assert_eq!(
            app.world_mut().query::<Entity>().iter(app.world()).count(),
            recruited_entities.len()
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<WorkerVisual>>()
                .iter(app.world())
                .count(),
            2
        );
        let helper = app
            .world_mut()
            .query::<&Worker>()
            .iter(app.world())
            .find(|w| w.id == 2)
            .unwrap();
        assert_eq!(helper.position, COASTAL_REST);
        assert_eq!(helper.task, Task::Idle);
    }
    #[test]
    fn runtime_invitation_during_salting_preserves_original_actor_and_job_owner() {
        let mut app = crew_app();
        let original = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.fishery.request_fishing();
            s.fishery.request_salting();
            s.household.request_supply();
            s.barter.salt.fill(SaltLocation::Stored);
        }
        for _ in 0..2000 {
            app.update();
            if app.world().get::<Worker>(original).unwrap().task
                == Task::Coastal(crate::fishing::CoastalTask::Salting)
            {
                break;
            }
        }
        assert_eq!(app.world().resource::<Settlement>().coastal_owner, Some(1));
        assert!(
            app.world_mut()
                .resource_mut::<Settlement>()
                .request_helper()
        );
        let mut helper_worked = false;
        for frame in 0..2000 {
            if frame % 7 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            let owner = app.world().resource::<Settlement>().coastal_owner;
            let primary_task = app.world().get::<Worker>(original).unwrap().task;
            let helper = app
                .world_mut()
                .query::<&Worker>()
                .iter(app.world())
                .find(|w| w.id == 2)
                .unwrap();
            helper_worked |= matches!(helper.task, Task::Coastal(_));
            if owner == Some(1) {
                assert!(!matches!(helper.task, Task::Coastal(_)));
            }
            if owner == Some(2) {
                assert!(!matches!(primary_task, Task::Coastal(_)));
            }
        }
        assert!(helper_worked);
        assert!(app.world().get::<Worker>(original).is_some());
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<WorkerVisual>>()
                .iter(app.world())
                .count(),
            2
        );
        assert_eq!(app.world().resource::<Settlement>().fishery.batches, 2);
        assert_eq!(
            app.world()
                .resource::<Settlement>()
                .barter
                .count(SaltLocation::Consumed),
            2
        );
    }
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
