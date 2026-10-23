//! Textured licensed anatomy, project costume, native clip blending and procedural contact IK.
use crate::{
    character_motion::{Clip, MotionMemory, two_bone},
    fishing::CoastalTask,
    motion,
    presentation::{AxeVisual, WorkerVisual},
    simulation::{Settlement, SimulationClock, Task, Worker},
};
use bevy::{
    app::AnimationSystems, camera::visibility::NoFrustumCulling, ecs::system::SystemParam,
    prelude::*, world_serialization::WorldInstanceReady,
};
use std::collections::HashMap;
const MODEL: &str = "embedded://hafthi_world/models/textured-resident.glb";
pub struct ResidentPlugin;
impl Plugin for ResidentPlugin {
    fn build(&self, app: &mut App) {
        crate::art_assets::register(app);
        app.init_resource::<MotionMemory>()
            .add_systems(Startup, load_library)
            .add_systems(
                Update,
                (prepare_graph, spawn_models, activate_models).chain(),
            )
            .add_systems(PostUpdate, drive_graph.before(AnimationSystems))
            .add_systems(
                PostUpdate,
                contact_ik
                    .after(AnimationSystems)
                    .before(TransformSystems::Propagate),
            );
    }
}
#[derive(Component)]
struct Requested;
#[derive(Component)]
struct ModelReady;
#[derive(Component)]
struct Activated;
#[derive(Component)]
pub(crate) struct StaticHuman(pub u32);
#[derive(Component, Clone)]
struct Character {
    actor: Entity,
    owner: Option<u32>,
    seed: u32,
    legs: [Option<[Entity; 3]>; 2],
    arms: [Option<[Entity; 3]>; 2],
}
struct Graph {
    handle: Handle<AnimationGraph>,
    nodes: [AnimationNodeIndex; 4],
    durations: [f32; 4],
}
#[derive(Resource)]
struct Library {
    asset: Handle<Gltf>,
    graph: Option<Graph>,
    walk_rate: f32,
    foot_height: [f32; 2],
}
#[derive(serde::Deserialize)]
struct Manifest {
    walk_source_speed: f32,
    foot_height: HashMap<String, f32>,
}
fn load_library(mut commands: Commands, server: Res<AssetServer>) {
    let m: Manifest = serde_json::from_str(include_str!("../assets/models/resident-motion.json"))
        .expect("checked-in motion manifest");
    commands.insert_resource(Library {
        asset: server.load(MODEL),
        graph: None,
        walk_rate: crate::simulation::SPEED / m.walk_source_speed,
        foot_height: [m.foot_height["L"], m.foot_height["R"]],
    });
}
fn prepare_graph(
    mut library: ResMut<Library>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    if library.graph.is_some() {
        return;
    }
    let Some(gltf) = gltfs.get(&library.asset) else {
        return;
    };
    let Some(handles) = Clip::ALL
        .iter()
        .map(|clip| gltf.named_animations.get(clip.name()).cloned())
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let Some(durations) = handles
        .iter()
        .map(|h| clips.get(h).map(AnimationClip::duration))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let (graph, nodes) = AnimationGraph::from_clips(handles);
    library.graph = Some(Graph {
        handle: graphs.add(graph),
        nodes: nodes.try_into().expect("four named clips"),
        durations: durations.try_into().expect("four clip durations"),
    });
}
fn spawn_models(
    mut commands: Commands,
    server: Res<AssetServer>,
    workers: Query<(Entity, &WorkerVisual), Without<Requested>>,
    visitors: Query<Entity, (With<StaticHuman>, Without<Requested>)>,
) {
    for entity in workers
        .iter()
        .filter(|(_, w)| matches!(w.0, 1 | 2))
        .map(|(e, _)| e)
        .chain(visitors.iter())
    {
        commands
            .entity(entity)
            .insert(Requested)
            .with_children(|p| {
                p.spawn((
                    WorldAssetRoot(server.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
                    Transform::from_xyz(0., -0.7, 0.),
                    Visibility::Hidden,
                ))
                .observe(model_ready)
                .observe(materials_ready);
            });
    }
}
fn model_ready(event: On<WorldInstanceReady>, mut commands: Commands) {
    commands.entity(event.entity).insert(ModelReady);
}
fn materials_ready(
    event: On<WorldInstanceReady>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    owners: Query<&WorkerVisual>,
    mut parts: Query<(
        &mut MeshMaterial3d<StandardMaterial>,
        &bevy::gltf::GltfMaterialName,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let id = parents
        .get(event.entity)
        .ok()
        .and_then(|p| owners.get(p.parent()).ok())
        .map_or(0, |v| v.0);
    let tint = match id {
        1 => Color::srgb(0.45, 0.58, 0.70),
        2 => Color::srgb(0.58, 0.64, 0.48),
        _ => Color::srgb(0.68, 0.54, 0.40),
    };
    let mut copies = HashMap::new();
    for entity in children.iter_descendants(event.entity) {
        let Ok((mut part, name)) = parts.get_mut(entity) else {
            continue;
        };
        if name.0 == "Undyed sage wool" {
            if let Some(handle) = copies.get(&part.0.id()) {
                part.0 = Handle::clone(handle);
                continue;
            }
            let Some(source) = materials.get(&part.0) else {
                continue;
            };
            let mut garment = source.clone();
            garment.base_color = tint;
            let handle = materials.add(garment);
            copies.insert(part.0.id(), handle.clone());
            part.0 = handle;
        } else if let Some(mut material) = materials.get_mut(&part.0) {
            if name.0.starts_with("Resident skin") {
                material.perceptual_roughness = 0.78;
                material.reflectance = 0.30;
                // A small thin-surface backlight contribution, not a full skin SSS solver.
                material.diffuse_transmission = 0.03;
            } else if name.0 == "Resident hair" {
                material.alpha_mode = AlphaMode::Mask(0.35);
                material.cull_mode = None;
                material.double_sided = true;
                material.perceptual_roughness = 0.95;
                material.reflectance = 0.15;
            }
        }
    }
}
#[derive(SystemParam)]
struct Scene<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    names: Query<'w, 's, &'static Name>,
    owners: Query<'w, 's, (Option<&'static WorkerVisual>, Option<&'static StaticHuman>)>,
    players: Query<'w, 's, &'static mut AnimationPlayer>,
    skins: Query<'w, 's, Entity, With<bevy::mesh::skinning::SkinnedMesh>>,
    axes: Query<'w, 's, Entity, With<AxeVisual>>,
}
type ReadyRoots<'w, 's> =
    Query<'w, 's, (Entity, &'static ChildOf), (With<ModelReady>, Without<Activated>)>;
fn activate_models(
    mut commands: Commands,
    library: Res<Library>,
    roots: ReadyRoots,
    mut scene: Scene,
) {
    let Some(graph) = &library.graph else {
        return;
    };
    for (root, parent) in &roots {
        let actor = parent.parent();
        let Ok((worker, visitor)) = scene.owners.get(actor) else {
            continue;
        };
        let owner = worker.map(|w| w.0);
        let seed = owner.unwrap_or_else(|| visitor.map_or(3, |v| v.0));
        let descendants: Vec<_> = scene.children.iter_descendants(root).collect();
        let Some(player_entity) = descendants
            .iter()
            .copied()
            .find(|e| scene.players.contains(*e))
        else {
            continue;
        };
        let mut player = scene
            .players
            .get_mut(player_entity)
            .expect("player checked");
        for (i, node) in graph.nodes.iter().enumerate() {
            player
                .play(*node)
                .repeat()
                .pause()
                .set_weight(if i == 0 { 1. } else { 0. });
        }
        let names: HashMap<_, _> = descendants
            .iter()
            .filter_map(|e| scene.names.get(*e).ok().map(|name| (name.as_str(), *e)))
            .collect();
        let chain = |side: &str, parts: [&str; 3]| {
            parts
                .map(|p| names.get(format!("Bip01 {side} {p}").as_str()).copied())
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .and_then(|v| v.try_into().ok())
        };
        commands.entity(player_entity).insert((
            AnimationGraphHandle(graph.handle.clone()),
            Character {
                actor,
                owner,
                seed,
                legs: [
                    chain("L", ["Thigh", "Calf", "Foot"]),
                    chain("R", ["Thigh", "Calf", "Foot"]),
                ],
                arms: [
                    chain("L", ["UpperArm", "Forearm", "Hand"]),
                    chain("R", ["UpperArm", "Forearm", "Hand"]),
                ],
            },
        ));
        if owner == Some(1)
            && let Some(grip) = names.get("AxeGrip")
        {
            for axe in &scene.axes {
                commands
                    .entity(axe)
                    .insert((ChildOf(*grip), Transform::default()));
            }
        }
        commands.entity(actor).remove::<Mesh3d>();
        if let Ok(old) = scene.children.get(actor) {
            for child in old.iter().filter(|e| *e != root) {
                commands.entity(child).despawn();
            }
        }
        for skin in descendants.iter().filter(|e| scene.skins.contains(**e)) {
            commands.entity(*skin).insert(NoFrustumCulling);
        }
        commands
            .entity(root)
            .insert((Activated, Visibility::Inherited));
    }
}
fn clip_for(w: &Worker) -> Clip {
    if motion::walking(w.task) {
        Clip::Walk
    } else {
        match w.task {
            Task::Felling
            | Task::Reaping
            | Task::Sowing
            | Task::Shearing
            | Task::PreparingSeeds => Clip::Work,
            Task::Coastal(CoastalTask::Salting | CoastalTask::Fishing) => Clip::Table,
            _ => Clip::Idle,
        }
    }
}
fn drive_graph(
    library: Res<Library>,
    clock: Res<SimulationClock>,
    workers: Query<&Worker>,
    mut memory: ResMut<MotionMemory>,
    mut players: Query<(&Character, &mut AnimationPlayer)>,
    mut roots: Query<&mut Transform, With<WorkerVisual>>,
) {
    let Some(graph) = &library.graph else {
        return;
    };
    let ids: Vec<_> = workers.iter().map(|w| w.id).collect();
    memory.retain(&ids);
    for (character, mut player) in &mut players {
        let weights = if let Some(owner) = character.owner {
            let Some(worker) = workers.iter().find(|w| w.id == owner) else {
                continue;
            };
            let sample = memory.update(owner, clip_for(worker), worker.facing, clock.0);
            if let Ok(mut root) = roots.get_mut(character.actor) {
                root.rotation = Quat::from_rotation_y(sample.heading);
            }
            sample.weights
        } else {
            [1., 0., 0., 0.]
        };
        for (i, node) in graph.nodes.iter().enumerate() {
            if let Some(animation) = player.animation_mut(*node) {
                let rate = if i == Clip::Walk.index() {
                    library.walk_rate
                } else {
                    1.
                };
                let time = ((clock.0 * rate as f64 + character.seed as f64 * 0.37)
                    % graph.durations[i] as f64) as f32;
                animation.pause().set_weight(weights[i]).set_seek_time(time);
            }
        }
    }
}
// Compute this frame's hierarchy before Bevy propagates GlobalTransform; never use stale globals for IK.
fn current_world(
    entity: Entity,
    transforms: &Query<&mut Transform>,
    parents: &Query<&ChildOf>,
) -> Option<Mat4> {
    let mut chain = Vec::new();
    let mut cursor = entity;
    for _ in 0..64 {
        chain.push(*transforms.get(cursor).ok()?);
        if let Ok(parent) = parents.get(cursor) {
            cursor = parent.parent();
        } else {
            return Some(
                chain
                    .into_iter()
                    .rev()
                    .fold(Mat4::IDENTITY, |m, t| m * t.to_matrix()),
            );
        }
    }
    None
}
fn set_world_rotation(
    entity: Entity,
    rotation: Quat,
    transforms: &mut Query<&mut Transform>,
    parents: &Query<&ChildOf>,
) {
    let parent = parents
        .get(entity)
        .ok()
        .and_then(|p| current_world(p.parent(), transforms, parents))
        .map(|m| m.to_scale_rotation_translation().1)
        .unwrap_or(Quat::IDENTITY);
    if let Ok(mut t) = transforms.get_mut(entity) {
        t.rotation = parent.inverse() * rotation;
    }
}
fn solve_chain(
    chain: [Entity; 3],
    target: Vec3,
    pole: Vec3,
    weight: f32,
    transforms: &mut Query<&mut Transform>,
    parents: &Query<&ChildOf>,
) {
    if weight <= 0.0001 {
        return;
    }
    let Some(a) = current_world(chain[0], transforms, parents) else {
        return;
    };
    let Some(b) = current_world(chain[1], transforms, parents) else {
        return;
    };
    let Some(c) = current_world(chain[2], transforms, parents) else {
        return;
    };
    let (ar, ap) = (a.to_scale_rotation_translation().1, a.w_axis.truncate());
    let bp = b.w_axis.truncate();
    let cp = c.w_axis.truncate();
    let target = cp.lerp(target, weight.clamp(0., 1.));
    let Some((joint, end)) = two_bone(ap, bp, cp, target, pole) else {
        return;
    };
    let first = Quat::from_rotation_arc((bp - ap).normalize(), (joint - ap).normalize()) * ar;
    set_world_rotation(chain[0], first, transforms, parents);
    let Some(b) = current_world(chain[1], transforms, parents) else {
        return;
    };
    let Some(after) = current_world(chain[2], transforms, parents) else {
        return;
    };
    let bp = b.w_axis.truncate();
    let cp = after.w_axis.truncate();
    if let (Some(from), Some(to)) = ((cp - bp).try_normalize(), (end - bp).try_normalize()) {
        set_world_rotation(
            chain[1],
            Quat::from_rotation_arc(from, to) * b.to_scale_rotation_translation().1,
            transforms,
            parents,
        );
    }
    set_world_rotation(
        chain[2],
        c.to_scale_rotation_translation().1,
        transforms,
        parents,
    );
}
fn contact_ik(
    library: Res<Library>,
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    characters: Query<&Character>,
    parents: Query<&ChildOf>,
    mut transforms: Query<&mut Transform>,
) {
    for character in &characters {
        let Some(worker) = character
            .owner
            .and_then(|id| workers.iter().find(|w| w.id == id))
        else {
            continue;
        };
        let Some(actor) = current_world(character.actor, &transforms, &parents) else {
            continue;
        };
        let rotation = actor.to_scale_rotation_translation().1;
        for (side, chain) in character.legs.iter().enumerate() {
            let Some(chain) = chain else {
                continue;
            };
            let Some(foot) = current_world(chain[2], &transforms, &parents) else {
                continue;
            };
            let position = foot.w_axis.truncate();
            let p = Vec2::new(position.x, position.z);
            let y = crate::landscape::height(p) + library.foot_height[side];
            let lift = position.y - y;
            let weight = (1. - ((lift - 0.02) / 0.08).clamp(0., 1.)).clamp(0., 1.);
            solve_chain(
                *chain,
                Vec3::new(position.x, y, position.z),
                position + rotation * Vec3::Z * 0.4,
                weight,
                &mut transforms,
                &parents,
            );
        }
        let large = worker.carrying.is_some();
        let small = worker.carrying_grain.is_some()
            || worker.carrying_wool.is_some()
            || worker.carrying_salt.is_some()
            || worker.carrying_fish.is_some()
            || (worker.id == 1 && settlement.farm.seeds == crate::farming::SeedLocation::Carried);
        if !large && !small {
            continue;
        }
        let centre = if large {
            Vec3::new(worker.position.x, 1.22, worker.position.y + 0.32)
        } else {
            Vec3::new(worker.position.x + 0.3, 1.13, worker.position.y)
        };
        let shoulders = character.arms.map(|chain| {
            chain
                .and_then(|c| current_world(c[0], &transforms, &parents))
                .map(|m| m.w_axis.truncate())
        });
        let distances = shoulders.map(|p| p.map_or(10., |p| p.distance(centre)));
        // Logs retain their existing world-aligned cargo orientation. Assign
        // the nearer end to each hand, including when the resident faces back.
        let first_positive = match shoulders {
            [Some(a), Some(b)] => a.x >= b.x,
            _ => true,
        };
        for (side, chain) in character.arms.iter().enumerate() {
            let Some(chain) = chain else {
                continue;
            };
            let sign = if side == 0 { 1. } else { -1. };
            let weight = if large {
                1.
            } else {
                (0.5 + (distances[1 - side] - distances[side]) / 0.12).clamp(0., 1.)
            };
            let grip_sign = if (side == 0) == first_positive {
                1.
            } else {
                -1.
            };
            let target = centre + Vec3::X * if large { grip_sign * 0.23 } else { 0. };
            let pole = actor.w_axis.truncate() + rotation * Vec3::new(sign * 0.5, 0.15, -0.2);
            solve_chain(*chain, target, pole, weight, &mut transforms, &parents);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contact_solver_uses_current_local_hierarchy_and_preserves_end_orientation() {
        let mut app = App::new();
        let root = app
            .world_mut()
            .spawn(Transform::from_xyz(2., 0.2, -1.).with_rotation(Quat::from_rotation_y(0.7)))
            .id();
        let a = app
            .world_mut()
            .spawn((ChildOf(root), Transform::from_xyz(0., 1., 0.)))
            .id();
        let b = app
            .world_mut()
            .spawn((
                ChildOf(a),
                Transform::from_xyz(0., -0.5, 0.).with_rotation(Quat::from_rotation_x(0.3)),
            ))
            .id();
        let c = app
            .world_mut()
            .spawn((
                ChildOf(b),
                Transform::from_xyz(0., -0.45, 0.).with_rotation(Quat::from_rotation_z(0.2)),
            ))
            .id();
        let target = Vec3::new(2.2, 0.4, -0.8);
        app.add_systems(
            Update,
            move |mut transforms: Query<&mut Transform>, parents: Query<&ChildOf>| {
                let before = current_world(c, &transforms, &parents)
                    .unwrap()
                    .to_scale_rotation_translation()
                    .1;
                solve_chain(
                    [a, b, c],
                    target,
                    Vec3::new(2., 1., 0.),
                    1.,
                    &mut transforms,
                    &parents,
                );
                let world = [a, b, c].map(|e| current_world(e, &transforms, &parents).unwrap());
                let points = world.map(|m| m.w_axis.truncate());
                assert!(points[2].distance(target) < 0.0001);
                assert!((points[0].distance(points[1]) - 0.5).abs() < 0.0001);
                assert!((points[1].distance(points[2]) - 0.45).abs() < 0.0001);
                assert!(world[2].to_scale_rotation_translation().1.dot(before).abs() > 0.99999);
            },
        );
        // No GlobalTransform propagation is available in this app: the solve
        // must use this frame's local transforms rather than stale globals.
        app.update();
    }

    #[test]
    fn native_player_seeks_from_saved_clock_and_restores_paused_blend() {
        let mut app = App::new();
        app.init_resource::<MotionMemory>()
            .insert_resource(SimulationClock(12.))
            .add_systems(Update, drive_graph);
        let mut graphs = Assets::<AnimationGraph>::default();
        let (graph, indices) =
            AnimationGraph::from_clips((0..4).map(|_| Handle::<AnimationClip>::default()));
        let nodes: [AnimationNodeIndex; 4] = indices.try_into().unwrap();
        app.insert_resource(Library {
            asset: Handle::default(),
            graph: Some(Graph {
                handle: graphs.add(graph),
                nodes,
                durations: [3., 1.2, 11., 10.],
            }),
            walk_rate: 1.3,
            foot_height: [0.1; 2],
        });
        let worker = app.world_mut().spawn(Worker::default()).id();
        let actor = app
            .world_mut()
            .spawn((WorkerVisual(1), Transform::default()))
            .id();
        let mut player = AnimationPlayer::default();
        for node in nodes {
            player.play(node).repeat();
        }
        let player = app
            .world_mut()
            .spawn((
                Character {
                    actor,
                    owner: Some(1),
                    seed: 1,
                    legs: [None; 2],
                    arms: [None; 2],
                },
                player,
            ))
            .id();
        app.update();
        app.world_mut().get_mut::<Worker>(worker).unwrap().task = Task::ToTree;
        app.update();
        app.world_mut().resource_mut::<SimulationClock>().0 = 12.14;
        app.update();
        let state = |app: &App| {
            let p = app.world().get::<AnimationPlayer>(player).unwrap();
            nodes.map(|node| {
                let a = p.animation(node).unwrap();
                assert!(a.is_paused());
                (a.weight(), a.seek_time())
            })
        };
        let middle = state(&app);
        assert!((middle[0].0 - 0.5).abs() < 0.001 && (middle[1].0 - 0.5).abs() < 0.001);
        let saved = app.world().resource::<MotionMemory>().clone();
        // Render updates do not advance native animation while simulation time is paused.
        for _ in 0..3 {
            app.update();
            assert_eq!(state(&app), middle);
        }
        app.world_mut().resource_mut::<SimulationClock>().0 = 15.;
        app.update();
        assert_ne!(state(&app), middle);
        app.world_mut().insert_resource(saved);
        app.world_mut().resource_mut::<SimulationClock>().0 = 12.14;
        app.update();
        assert_eq!(state(&app), middle);
    }
}
