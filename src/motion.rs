//! Deterministic presentation poses from the real task, cargo and saved world clock.
use crate::{
    fishing::CoastalTask as C,
    simulation::{Settlement, Task, Worker},
};
use bevy::prelude::*;
pub fn walking(task: Task) -> bool {
    matches!(
        task,
        Task::ToCrewRest
            | Task::ToTree
            | Task::ToLog
            | Task::ToStockpile
            | Task::ToSite
            | Task::ToFarmGateway
            | Task::ToSeedStore
            | Task::ToField
            | Task::DeliveringGrain
            | Task::FromField
            | Task::ToPastureGateway
            | Task::ToPastureEdge
            | Task::ToPasture
            | Task::ToSheep
            | Task::ToWool
            | Task::FromPasture
            | Task::FromPastureEdge
            | Task::DeliveringWool
            | Task::ToFoodStore
            | Task::ToHome
            | Task::FromHome
            | Task::ToFuelStore
            | Task::ToFuelHome
            | Task::ToTradeStore
            | Task::ToTrader
            | Task::FromTrader
            | Task::Coastal(
                C::ToExportPile
                    | C::ToBoat
                    | C::ReturnFromBoat
                    | C::ToFishingSpot
                    | C::ReturningFresh
                    | C::ToBrineStore
                    | C::ToTableWithSalt
                    | C::BackForRaw
                    | C::ToTableWithRaw
                    | C::BackWithPreserved
                    | C::ToPickPreserved
                    | C::ToMealStore
                    | C::ToFishHome
            )
    )
}
fn cargo(w: &Worker, s: &Settlement) -> Option<Vec3> {
    if w.carrying.is_some() {
        Some(Vec3::new(0., 0.25, 0.32))
    } else if w.carrying_grain.is_some()
        || w.carrying_wool.is_some()
        || w.carrying_salt.is_some()
        || w.carrying_fish.is_some()
        || (w.id == 1 && s.farm.seeds == crate::farming::SeedLocation::Carried)
    {
        Some(Vec3::new(0.3, 0.10, 0.))
    } else {
        None
    }
}
pub fn pose(w: &Worker, s: &Settlement, seconds: f64, side: f32, arm: bool) -> (Quat, Vec3) {
    let cycle = (seconds as f32 * 7.5 + w.id as f32 * 0.7).sin() * side;
    if !arm {
        return (
            Quat::from_rotation_x(if walking(w.task) { cycle * 0.60 } else { 0. }),
            Vec3::ONE,
        );
    }
    if let Some(world_target) = cargo(w, s) {
        let local = Quat::from_rotation_y(-w.facing) * world_target;
        let shoulder = Vec3::new(side * 0.285, 0.34, 0.02);
        let delta = local + Vec3::new(side * 0.08, 0., 0.) - shoulder;
        let length = (delta.length() / 0.41).clamp(0.60, 1.15);
        return (
            Quat::from_rotation_arc(Vec3::NEG_Y, delta.normalize_or_zero()),
            Vec3::new(1., length, 1.),
        );
    }
    let action = match w.task {
        Task::Felling => {
            if side > 0. {
                -0.8 + (seconds as f32 * 9.).sin() * 0.55
            } else {
                -0.2
            }
        }
        Task::Reaping
        | Task::Sowing
        | Task::Shearing
        | Task::PreparingSeeds
        | Task::Coastal(C::Salting) => -0.65 + (seconds as f32 * 5.).sin() * 0.16,
        Task::Coastal(C::Fishing) => -0.48 + (seconds as f32 * 2.).sin() * 0.10,
        _ if walking(w.task) => -cycle * 0.30,
        _ => 0.,
    };
    (
        Quat::from_rotation_z(side * 0.13) * Quat::from_rotation_x(action),
        Vec3::ONE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walking_swings_opposite_legs_and_loaded_arms_reach_towards_real_cargo() {
        let mut w = Worker::default();
        let s = Settlement::default();
        w.task = Task::ToTree;
        let left = pose(&w, &s, 0.17, -1., false).0;
        let right = pose(&w, &s, 0.17, 1., false).0;
        assert!((left * right).abs_diff_eq(Quat::IDENTITY, 0.00001));
        assert_ne!(left, Quat::IDENTITY);
        w.task = Task::Felling;
        assert_eq!(pose(&w, &s, 0.17, 1., false).0, Quat::IDENTITY);
        assert_ne!(
            pose(&w, &s, 0.17, 1., true).0,
            pose(&w, &s, 0.31, 1., true).0
        );
        for kind in 0..5 {
            let mut w = Worker::default();
            w.task = Task::ToStockpile;
            w.facing = 1.1;
            match kind {
                0 => w.carrying = Some(0),
                1 => w.carrying_grain = Some(0),
                2 => w.carrying_wool = Some(0),
                3 => w.carrying_salt = Some(0),
                _ => w.carrying_fish = Some(0),
            }
            let saved = serde_json::to_vec(&w).unwrap();
            let restored: Worker = serde_json::from_slice(&saved).unwrap();
            for side in [-1., 1.] {
                let (rotation, scale) = pose(&w, &s, 123.25, side, true);
                assert!(rotation.is_finite() && scale.is_finite());
                assert!((0.60..=1.15).contains(&scale.y));
                let delta = Quat::from_rotation_y(-w.facing) * cargo(&w, &s).unwrap()
                    + Vec3::new(side * 0.08, 0., 0.)
                    - Vec3::new(side * 0.285, 0.34, 0.02);
                assert!((rotation * Vec3::NEG_Y).dot(delta.normalize()) > 0.9999);
                assert_eq!(
                    pose(&w, &s, 123.25, side, true),
                    pose(&restored, &s, 123.25, side, true)
                );
                assert_eq!(
                    pose(&w, &s, 123.25, side, true),
                    pose(&w, &s, 234.0, side, true)
                );
            }
        }
    }
    #[test]
    fn older_resident_saves_default_facing_without_changing_job_or_cargo() {
        let mut w = Worker::default();
        w.facing = 2.0;
        w.carrying_wool = Some(2);
        w.task = Task::DeliveringWool;
        let mut old = serde_json::to_value(&w).unwrap();
        old.as_object_mut().unwrap().remove("facing");
        let restored: Worker = serde_json::from_value(old).unwrap();
        assert_eq!(restored.facing, 0.0);
        assert_eq!(restored.carrying_wool, Some(2));
        assert_eq!(restored.task, Task::DeliveringWool);
    }
}
