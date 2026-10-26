//! Saved presentation state for clip and heading blends. Never makes gameplay decisions.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const BLEND_SECONDS: f64 = 0.28;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clip {
    #[default]
    Idle,
    Walk,
    Work,
    Table,
}
impl Clip {
    pub const ALL: [Self; 4] = [Self::Idle, Self::Walk, Self::Work, Self::Table];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Walk => "Walk",
            Self::Work => "Work",
            Self::Table => "Table",
        }
    }
    fn weights(self) -> [f32; 4] {
        let mut result = [0.; 4];
        result[self.index()] = 1.;
        result
    }
}
fn blend(now: f64, start: f64) -> f32 {
    let t = ((now - start) / BLEND_SECONDS).clamp(0., 1.) as f32;
    t * t * (3. - 2. * t)
}
fn angle(a: f32) -> f32 {
    a.sin().atan2(a.cos())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlendState {
    from: [f32; 4],
    target: Clip,
    changed_at: f64,
    heading_from: f32,
    heading_to: f32,
    heading_at: f64,
    last_time: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub weights: [f32; 4],
    pub heading: f32,
}
impl BlendState {
    fn new(target: Clip, heading: f32, time: f64) -> Self {
        Self {
            from: target.weights(),
            target,
            changed_at: time,
            heading_from: heading,
            heading_to: heading,
            heading_at: time,
            last_time: time,
        }
    }
    fn sample(&self, time: f64) -> Sample {
        let to = self.target.weights();
        let t = blend(time, self.changed_at);
        Sample {
            weights: std::array::from_fn(|i| self.from[i] + (to[i] - self.from[i]) * t),
            heading: self.heading_from
                + angle(self.heading_to - self.heading_from) * blend(time, self.heading_at),
        }
    }
    fn update(&mut self, target: Clip, heading: f32, time: f64) -> Sample {
        if time < self.last_time {
            *self = Self::new(target, heading, time);
        }
        if target != self.target {
            self.from = self.sample(time).weights;
            self.target = target;
            self.changed_at = time;
        }
        if angle(heading - self.heading_to).abs() > 0.0001 {
            self.heading_from = self.sample(time).heading;
            self.heading_to = heading;
            self.heading_at = time;
        }
        self.last_time = time;
        self.sample(time)
    }
    fn valid(&self, time: f64) -> bool {
        self.from
            .iter()
            .all(|w| w.is_finite() && (0.0..=1.0).contains(w))
            && (self.from.iter().sum::<f32>() - 1.).abs() < 0.001
            && [self.heading_from, self.heading_to]
                .iter()
                .all(|v| v.is_finite())
            && [self.changed_at, self.heading_at, self.last_time]
                .iter()
                .all(|t| t.is_finite() && *t >= 0. && *t <= time + 0.0001)
    }
}
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionMemory {
    actors: BTreeMap<u32, BlendState>,
}
impl MotionMemory {
    pub fn update(&mut self, id: u32, clip: Clip, heading: f32, time: f64) -> Sample {
        self.actors
            .entry(id)
            .or_insert_with(|| BlendState::new(clip, heading, time))
            .update(clip, heading, time)
    }
    pub fn retain(&mut self, ids: &[u32]) {
        self.actors.retain(|id, _| ids.contains(id));
    }
    pub fn valid(&self, time: f64) -> bool {
        self.actors.len() <= 3
            && self
                .actors
                .iter()
                .all(|(id, state)| matches!(*id, 1..=3) && state.valid(time))
    }
}

/// Analytic two-bone solve with a preferred bend plane and unreachable-target clamp.
pub fn two_bone(a: Vec3, b: Vec3, c: Vec3, target: Vec3, pole: Vec3) -> Option<(Vec3, Vec3)> {
    let upper = a.distance(b);
    let lower = b.distance(c);
    if ![a, b, c, target, pole].iter().all(|v| v.is_finite()) || upper < 0.0001 || lower < 0.0001 {
        return None;
    }
    let direction = (target - a).try_normalize()?;
    let distance = a
        .distance(target)
        .clamp((upper - lower).abs() + 0.00001, upper + lower - 0.00001);
    let offset = pole - a;
    let bend = (offset - direction * offset.dot(direction))
        .try_normalize()
        .unwrap_or_else(|| direction.any_orthonormal_vector());
    let along = (upper * upper + distance * distance - lower * lower) / (2. * distance);
    let across = (upper * upper - along * along).max(0.).sqrt();
    Some((
        a + direction * along + bend * across,
        a + direction * distance,
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blends_pause_save_restore_and_take_shortest_heading_path() {
        let mut memory = MotionMemory::default();
        memory.update(1, Clip::Idle, 3.10, 12.0);
        let beginning = memory.update(1, Clip::Walk, -3.10, 12.0);
        assert_eq!(beginning.weights, Clip::Idle.weights());
        let middle = memory.update(1, Clip::Walk, -3.10, 12.14);
        assert!(
            (middle.weights[0] - 0.5).abs() < 0.0001 && (middle.weights[1] - 0.5).abs() < 0.0001
        );
        assert!((middle.heading - std::f32::consts::PI).abs() < 0.01);
        let encoded = serde_json::to_vec(&memory).unwrap();
        let mut restored: MotionMemory = serde_json::from_slice(&encoded).unwrap();
        for _ in 0..5 {
            assert_eq!(middle, restored.update(1, Clip::Walk, -3.10, 12.14));
        }
        assert!(restored.valid(12.14));
        assert_eq!(
            restored.update(1, Clip::Walk, -3.10, 12.4).weights,
            Clip::Walk.weights()
        );
        let before = restored.update(2, Clip::Table, 0., 12.4);
        restored.update(1, Clip::Idle, 0., 13.);
        assert_eq!(before, restored.update(2, Clip::Table, 0., 12.4));
    }
    #[test]
    fn ik_preserves_lengths_and_clamps_unreachable_targets() {
        let a = Vec3::new(0., 1., 0.);
        let b = Vec3::new(0., 0.5, 0.1);
        let c = Vec3::ZERO;
        for target in [
            Vec3::new(0.3, 0.1, 0.2),
            Vec3::new(0., -5., 0.),
            Vec3::new(0.001, 0.99, 0.),
        ] {
            let (joint, end) = two_bone(a, b, c, target, Vec3::Z).unwrap();
            assert!((a.distance(joint) - a.distance(b)).abs() < 0.0001);
            assert!((joint.distance(end) - b.distance(c)).abs() < 0.0001);
            assert!(end.is_finite());
            if a.distance(target) < a.distance(b) + b.distance(c) {
                assert!(end.distance(target) < 0.0001);
            }
        }
        assert!(two_bone(a, a, c, c, Vec3::Z).is_none());
        assert!(two_bone(a, b, c, Vec3::splat(f32::NAN), Vec3::Z).is_none());
    }
}
