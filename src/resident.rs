//! One authored skinned resident study. The economy still owns the resident and cargo.
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
struct RigJoint {
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
) {
    for (entity, worker) in &workers {
        if worker.0 != 1 {
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
            .observe(ready);
        });
    }
}
fn ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    transforms: Query<&Transform>,
    nodes: Query<(Option<&Name>, Option<&bevy::mesh::skinning::SkinnedMesh>)>,
    axes: Query<Entity, With<AxeVisual>>,
) {
    let root = event.entity;
    let Ok(parent) = parents.get(root) else {
        return;
    };
    let owner = parent.parent();
    // Swap only once the complete model is ready; the same simulation visual root survives.
    commands.entity(owner).remove::<Mesh3d>();
    let descendants: Vec<_> = children.iter_descendants(root).collect();
    let right_hand = descendants.iter().copied().find(|id| {
        nodes
            .get(*id)
            .is_ok_and(|(n, _)| n.is_some_and(|n| n.as_str() == "hand.R"))
    });
    if let Some(hand) = right_hand {
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
        let Ok((name, skin)) = nodes.get(id) else {
            continue;
        };
        if skin.is_some() {
            commands.entity(id).insert(NoFrustumCulling);
        }
        let Some(name) = name else { continue };
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
    let Some(worker) = workers.iter().find(|w| w.id == 1) else {
        return;
    };
    for (joint, mut t) in &mut joints {
        let d = delta(joint.kind, joint.side, worker, &settlement, clock.0);
        *t = joint.rest;
        t.rotation = joint.parent_basis.inverse() * d * joint.parent_basis * joint.rest.rotation;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
