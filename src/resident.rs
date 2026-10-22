//! Shared authored skin with resident-specific work poses and static visiting crew.
use crate::{
    motion,
    presentation::{AxeVisual, WorkerVisual},
    simulation::{Settlement, SimulationClock, Worker},
};
use bevy::{
    camera::visibility::NoFrustumCulling, prelude::*, world_serialization::WorldInstanceReady,
};

pub struct ResidentPlugin;
impl Plugin for ResidentPlugin {
    fn build(&self, app: &mut App) {
        crate::art_assets::register(app);
        app.add_systems(Update, spawn_study)
            .add_systems(PostUpdate, pose_rig.before(TransformSystems::Propagate));
    }
}
#[derive(Component)]
struct Study;
#[derive(Component)]
pub(crate) struct StaticHuman;
#[derive(Component)]
struct RigJoint {
    owner: u32,
    side: f32,
    kind: JointKind,
    rest: Transform,
    parent_basis: Quat,
}
#[derive(Clone, Copy)]
enum JointKind {
    Arm,
    Forearm,
    Thigh,
    Shin,
    Foot,
}
fn spawn_study(
    mut commands: Commands,
    server: Res<AssetServer>,
    workers: Query<(Entity, &WorkerVisual), Without<Study>>,
    visitors: Query<Entity, (With<StaticHuman>, Without<Study>)>,
) {
    for (entity, worker) in &workers {
        if worker.0 != 1 && worker.0 != 2 {
            continue;
        }
        commands.entity(entity).insert(Study).with_children(|p| {
            p.spawn((
                WorldAssetRoot(
                    server.load(
                        GltfAssetLabel::Scene(0)
                            .from_asset("embedded://hafthi_world/models/coastal-resident.glb"),
                    ),
                ),
                Transform::from_xyz(0., -0.7, 0.),
            ))
            .observe(ready)
            .observe(tint_clothing);
        });
    }
    for entity in &visitors {
        commands.entity(entity).insert(Study).with_children(|p| {
            p.spawn((
                WorldAssetRoot(
                    server.load(
                        GltfAssetLabel::Scene(0)
                            .from_asset("embedded://hafthi_world/models/coastal-resident.glb"),
                    ),
                ),
                Transform::from_xyz(0., -0.7, 0.),
            ))
            .observe(ready)
            .observe(tint_clothing);
        });
    }
}
fn tint_clothing(
    event: On<WorldInstanceReady>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    owners: Query<&WorkerVisual>,
    mut parts: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let owner = parents
        .get(event.entity)
        .ok()
        .and_then(|p| owners.get(p.parent()).ok())
        .map_or(0, |v| v.0);
    let tint = match owner {
        1 => Color::srgb(0.65, 0.78, 1.05),
        2 => Color::srgb(0.9, 1., 0.84),
        _ => Color::srgb(1.05, 0.85, 0.7),
    };
    let mut copies: std::collections::HashMap<
        bevy::asset::AssetId<StandardMaterial>,
        Handle<StandardMaterial>,
    > = std::collections::HashMap::new();
    for entity in children.iter_descendants(event.entity) {
        let Ok(mut part) = parts.get_mut(entity) else {
            continue;
        };
        if let Some(handle) = copies.get(&part.0.id()) {
            part.0 = handle.clone();
            continue;
        }
        let Some(source) = materials.get(&part.0) else {
            continue;
        };
        // Only the wool garment has a base-color map; keep skin, linen, leather and hair intact.
        if source.base_color_texture.is_none() {
            continue;
        }
        let mut garment = source.clone();
        garment.base_color = tint;
        let handle = materials.add(garment);
        copies.insert(part.0.id(), handle.clone());
        part.0 = handle;
    }
}
fn ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    transforms: Query<&Transform>,
    nodes: Query<(
        Option<&Name>,
        Option<&bevy::mesh::skinning::SkinnedMesh>,
        Option<&WorkerVisual>,
    )>,
    axes: Query<Entity, With<AxeVisual>>,
) {
    let root = event.entity;
    let Ok(parent) = parents.get(root) else {
        return;
    };
    let owner = parent.parent();
    let resident_id = nodes
        .get(owner)
        .ok()
        .and_then(|(_, _, visual)| visual.map(|v| v.0))
        .unwrap_or(0);
    // Swap only once the complete model is ready; the same simulation visual root survives.
    commands.entity(owner).remove::<Mesh3d>();
    let descendants: Vec<_> = children.iter_descendants(root).collect();
    let right_hand = descendants.iter().copied().find(|id| {
        nodes
            .get(*id)
            .is_ok_and(|(n, _, _)| n.is_some_and(|n| n.as_str() == "hand.R"))
    });
    if let Some(hand) = right_hand.filter(|_| resident_id == 1) {
        for axe in &axes {
            commands
                .entity(axe)
                .insert((ChildOf(hand), Transform::from_xyz(0., 0.06, 0.02)));
        }
    }
    if let Ok(old) = children.get(owner) {
        for child in old.iter().filter(|c| *c != root) {
            commands.entity(child).despawn();
        }
    }
    for id in descendants {
        let Ok((name, skin, _)) = nodes.get(id) else {
            continue;
        };
        if skin.is_some() {
            commands.entity(id).insert(NoFrustumCulling);
        }
        let Some(name) = name else { continue };
        if resident_id == 0 {
            continue;
        }
        let Some((kind, side)) = joint(name.as_str()) else {
            continue;
        };
        let Ok(rest) = transforms.get(id) else {
            continue;
        };
        let mut basis = Quat::IDENTITY;
        let mut cursor = id;
        while let Ok(parent) = parents.get(cursor) {
            cursor = parent.parent();
            if cursor == root {
                break;
            }
            if let Ok(t) = transforms.get(cursor) {
                basis = t.rotation * basis;
            }
        }
        commands.entity(id).insert(RigJoint {
            owner: resident_id,
            side,
            kind,
            rest: *rest,
            parent_basis: basis,
        });
    }
}
fn joint(name: &str) -> Option<(JointKind, f32)> {
    let (name, side) = name.rsplit_once('.')?;
    let side = match side {
        "L" => -1.,
        "R" => 1.,
        _ => return None,
    };
    Some((
        match name {
            "upper_arm" => JointKind::Arm,
            "forearm" => JointKind::Forearm,
            "thigh" => JointKind::Thigh,
            "shin" => JointKind::Shin,
            "foot" => JointKind::Foot,
            _ => return None,
        },
        side,
    ))
}
fn delta(kind: JointKind, side: f32, w: &Worker, settlement: &Settlement, seconds: f64) -> Quat {
    let phase = seconds as f32 * 7.5 + w.id as f32 * 0.7;
    let swing = phase.sin() * side;
    let walking = motion::walking(w.task);
    let loaded = w.carrying.is_some()
        || w.carrying_grain.is_some()
        || w.carrying_wool.is_some()
        || w.carrying_salt.is_some()
        || w.carrying_fish.is_some()
        || (w.id == 1 && settlement.farm.seeds == crate::farming::SeedLocation::Carried);
    match kind {
        JointKind::Arm if loaded => {
            let target = if w.carrying.is_some() {
                Vec3::new(0., 0.25, 0.32)
            } else {
                Vec3::new(0.3, 0.1, 0.)
            };
            let shoulder = Vec3::new(side * 0.205, 0.57, 0.);
            let d = Quat::from_rotation_y(-w.facing) * (target + Vec3::Y * 0.03) - shoulder;
            let rest = Vec3::new(side * 0.051, -0.23, 0.).normalize();
            Quat::from_rotation_arc(rest, d.normalize())
        }
        JointKind::Arm => motion::pose(w, settlement, seconds, side, true).0,
        JointKind::Forearm => Quat::from_rotation_x(if loaded {
            -0.10
        } else if walking {
            -0.18 - swing.max(0.) * 0.28
        } else {
            -0.12
        }),
        JointKind::Thigh => Quat::from_rotation_x(if walking { swing * 0.36 } else { 0. }),
        JointKind::Shin => Quat::from_rotation_x(if walking {
            -(-swing).max(0.) * 0.65
        } else {
            0.
        }),
        JointKind::Foot => {
            Quat::from_rotation_x(if walking { (-swing).max(0.) * 0.20 } else { 0. })
        }
    }
}
fn pose_rig(
    clock: Res<SimulationClock>,
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    mut joints: Query<(&RigJoint, &mut Transform)>,
) {
    for (joint, mut t) in &mut joints {
        let Some(worker) = workers.iter().find(|w| w.id == joint.owner) else {
            continue;
        };
        let d = delta(joint.kind, joint.side, worker, &settlement, clock.0);
        *t = joint.rest;
        t.rotation = joint.parent_basis.inverse() * d * joint.parent_basis * joint.rest.rotation;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rigs_follow_their_own_resident_and_saved_clock() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Settlement>()
            .insert_resource(SimulationClock(12.5))
            .add_systems(Update, pose_rig);
        let mut founder = Worker::default();
        founder.task = crate::simulation::Task::ToTree;
        let mut helper = Worker::default();
        helper.id = 2;
        app.world_mut().spawn(founder.clone());
        let helper_entity = app.world_mut().spawn(helper).id();
        let make = |owner| {
            (
                RigJoint {
                    owner,
                    side: 1.,
                    kind: JointKind::Thigh,
                    rest: Transform::default(),
                    parent_basis: Quat::IDENTITY,
                },
                Transform::default(),
            )
        };
        let a = app.world_mut().spawn(make(1)).id();
        let b = app.world_mut().spawn(make(2)).id();
        app.update();
        let saved = *app.world().get::<Transform>(a).unwrap();
        assert_ne!(saved.rotation, Quat::IDENTITY);
        assert_eq!(
            app.world().get::<Transform>(b).unwrap().rotation,
            Quat::IDENTITY
        );
        for _ in 0..3 {
            app.update();
            assert_eq!(*app.world().get::<Transform>(a).unwrap(), saved);
        }
        app.world_mut()
            .get_mut::<Worker>(helper_entity)
            .unwrap()
            .task = crate::simulation::Task::ToTree;
        app.update();
        assert_eq!(*app.world().get::<Transform>(a).unwrap(), saved);
        assert_ne!(
            app.world().get::<Transform>(b).unwrap().rotation,
            Quat::IDENTITY
        );
        app.world_mut().resource_mut::<SimulationClock>().0 = 12.7;
        app.update();
        assert_ne!(*app.world().get::<Transform>(a).unwrap(), saved);
        app.world_mut().resource_mut::<SimulationClock>().0 = 12.5;
        app.update();
        assert_eq!(*app.world().get::<Transform>(a).unwrap(), saved);
    }
    #[test]
    fn rig_poses_are_finite_deterministic_and_cargo_respects_heading() {
        let mut w = Worker::default();
        let s = Settlement::default();
        w.task = crate::simulation::Task::ToTree;
        for kind in [
            JointKind::Arm,
            JointKind::Forearm,
            JointKind::Thigh,
            JointKind::Shin,
            JointKind::Foot,
        ] {
            for side in [-1., 1.] {
                let q = delta(kind, side, &w, &s, 12.5);
                assert!(q.is_finite());
                assert_eq!(q, delta(kind, side, &w, &s, 12.5));
            }
        }
        w.carrying_wool = Some(0);
        let a = delta(JointKind::Arm, 1., &w, &s, 12.5);
        w.facing = 2.;
        assert_ne!(a, delta(JointKind::Arm, 1., &w, &s, 12.5));
        assert!(joint("hand.R").is_none());
        assert!(joint("shin.L").is_some());
    }
}
