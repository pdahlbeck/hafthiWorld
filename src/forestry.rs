//! Replanting, timed growth and stable physical timber slots. Never recycle spent material.
use crate::simulation::{LOGS_PER_TREE, LogLocation, Settlement, TOTAL_LOGS, TREE_POSITIONS, Tree};
use bevy::prelude::*;
pub const GROW_SECONDS: f64 = 2.0 * crate::calendar::SEASON_SECONDS;
pub const MAX_LOGS: usize = 2048;
pub const LIVE_LOG_LIMIT: usize = 24;
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct Plot {
    pub ordered: bool,
    pub growing: bool,
    pub growth: f64,
    pub generation: u64,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimberPriority {
    #[default]
    Balanced,
    Building,
    Winter,
}
impl TimberPriority {
    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::Building => "Building first",
            Self::Winter => "Winter reserves",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Balanced => Self::Building,
            Self::Building => Self::Winter,
            Self::Winter => Self::Balanced,
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Forestry {
    pub plots: [Plot; 3],
    pub origins: Vec<usize>,
    pub slots: Vec<usize>,
    pub priority: TimberPriority,
}
impl Default for Forestry {
    fn default() -> Self {
        Self {
            plots: [Plot::default(); 3],
            origins: (0..TOTAL_LOGS).map(|i| i / LOGS_PER_TREE).collect(),
            slots: (0..TOTAL_LOGS).collect(),
            priority: default(),
        }
    }
}
impl Forestry {
    pub fn advance(&mut self, seconds: f64, trees: &mut [Tree; 3]) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        for (plot, tree) in self.plots.iter_mut().zip(trees) {
            if plot.growing {
                plot.growth = (plot.growth + seconds).min(GROW_SECONDS);
                if plot.growth >= GROW_SECONDS {
                    plot.growing = false;
                    plot.generation = plot.generation.saturating_add(1);
                    tree.felled = false;
                    tree.ordered = false;
                }
            }
        }
    }
    pub fn valid(&self, logs: &[LogLocation], trees: &[Tree; 3]) -> bool {
        logs.len() >= TOTAL_LOGS
            && logs.len() <= MAX_LOGS
            && logs.len().is_multiple_of(LOGS_PER_TREE)
            && self.origins.len() == logs.len()
            && self.slots.len() == logs.len()
            && self.origins.iter().all(|&i| i < 3)
            && self.slots.iter().all(|&i| i < MAX_LOGS)
            && self.plots.iter().zip(trees).all(|(p, t)| {
                p.growth.is_finite()
                    && (0.0..=GROW_SECONDS).contains(&p.growth)
                    && (!(p.ordered || p.growing) || t.felled)
                    && !(p.ordered && p.growing)
            })
    }
}
impl Settlement {
    pub fn harvest_room(&self) -> bool {
        let pending = self
            .trees
            .iter()
            .enumerate()
            .filter(|(i, t)| t.ordered && !t.felled && self.forestry.plots[*i].generation > 0)
            .count()
            * LOGS_PER_TREE;
        let live = self
            .logs
            .iter()
            .filter(|&&l| {
                matches!(
                    l,
                    LogLocation::InTree
                        | LogLocation::Ground
                        | LogLocation::Carried
                        | LogLocation::Stored
                )
            })
            .count();
        self.logs.len() + pending + LOGS_PER_TREE <= MAX_LOGS
            && live + pending + LOGS_PER_TREE <= LIVE_LOG_LIMIT
    }
    pub fn plot_status(&self, i: usize) -> String {
        let p = &self.forestry.plots[i];
        if p.ordered {
            "planting ordered".into()
        } else if p.growing {
            format!("growing {:.0}%", 100.0 * p.growth / GROW_SECONDS)
        } else if self.trees[i].felled {
            "stump: X to replant".into()
        } else if self.trees[i].ordered {
            "harvest ordered".into()
        } else {
            "mature: F to harvest".into()
        }
    }
    pub fn request_planting(&mut self, tree: usize) -> bool {
        let Some(t) = self.trees.get(tree) else {
            return false;
        };
        let p = &mut self.forestry.plots[tree];
        if !t.felled || p.ordered || p.growing {
            return false;
        }
        p.ordered = true;
        true
    }
    pub fn plant(&mut self, tree: usize) {
        let p = &mut self.forestry.plots[tree];
        if p.ordered && self.trees[tree].felled {
            p.ordered = false;
            p.growing = true;
            p.growth = 0.;
        }
    }
    pub fn fell(&mut self, tree: usize) {
        if self.trees[tree].felled || !self.trees[tree].ordered {
            return;
        }
        if self.forestry.plots[tree].generation == 0
            && self.logs[tree * LOGS_PER_TREE..(tree + 1) * LOGS_PER_TREE]
                .iter()
                .any(|&l| l != LogLocation::InTree)
        {
            self.trees[tree].ordered = false;
            return;
        }

        if self.forestry.plots[tree].generation == 0 {
            for i in tree * LOGS_PER_TREE..(tree + 1) * LOGS_PER_TREE {
                self.logs[i] = LogLocation::Ground;
            }
        } else {
            if self.logs.len() + LOGS_PER_TREE > MAX_LOGS {
                self.trees[tree].ordered = false;
                return;
            }
            for _ in 0..LOGS_PER_TREE {
                self.logs.push(LogLocation::Ground);
                self.forestry.origins.push(tree);
                self.forestry.slots.push(0);
            }
        }
        self.trees[tree].felled = true;
    }
    pub fn log_ground_position(&self, i: usize) -> Vec2 {
        if i < TOTAL_LOGS {
            return crate::simulation::ground_log_position(i);
        }
        TREE_POSITIONS[self.forestry.origins[i]]
            + Vec2::new((i % LOGS_PER_TREE) as f32 * 0.45 - 0.225, 0.5)
    }
    /// A slot changes only when actual transport unloads it. Other log piles do
    /// not slide or reorder just because another item was picked up or burned.
    pub fn place_log(&mut self, i: usize, location: LogLocation) {
        let slot = (0..MAX_LOGS)
            .find(|&slot| {
                !self
                    .logs
                    .iter()
                    .enumerate()
                    .any(|(j, &l)| j != i && l == location && self.forestry.slots[j] == slot)
            })
            .expect("bounded log slots");
        self.forestry.slots[i] = slot;
        self.logs[i] = location;
    }
    pub fn timber_build_need(&self) -> usize {
        let rack = if self.construction.ordered && !self.construction.complete {
            crate::simulation::BUILD_COST.saturating_sub(self.installed())
        } else {
            0
        };
        let home = if self.homestead.ordered && !self.homestead.complete {
            crate::homestead::TIMBER_COST.saturating_sub(self.homestead_timbers())
        } else {
            0
        };
        rack + home
    }
    pub fn timber_winter_need(&self) -> usize {
        let first = if self.household.hearth.ordered {
            crate::hearth::FUEL_TARGET.saturating_sub(self.count(LogLocation::Home(1)))
        } else {
            0
        };
        let second = if self.homestead.arrived && self.homestead.household.hearth.ordered {
            crate::hearth::FUEL_TARGET.saturating_sub(self.count(LogLocation::Home(2)))
        } else {
            0
        };
        first + second
    }
    pub fn timber_for_fuel(&self) -> bool {
        match self.forestry.priority {
            TimberPriority::Balanced | TimberPriority::Building => {
                self.stored() > self.timber_build_need()
            }
            TimberPriority::Winter => self.stored() > 0,
        }
    }
    pub fn timber_for_build(&self) -> bool {
        match self.forestry.priority {
            TimberPriority::Winter => self.stored() > self.timber_winter_need(),
            _ => self.stored() > 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{Task, Worker};
    #[test]
    fn queued_harvests_respect_live_capacity_without_losing_existing_stock() {
        let mut s = Settlement::default();
        s.logs = vec![LogLocation::Stored; LIVE_LOG_LIMIT];
        s.forestry.origins = (0..LIVE_LOG_LIMIT).map(|i| i % 3).collect();
        s.forestry.slots = (0..LIVE_LOG_LIMIT).collect();
        s.forestry.plots[0].generation = 1;
        let old = s.logs.clone();
        assert!(!s.request_harvest(0));
        assert_eq!(s.logs, old);
        s.logs[0] = LogLocation::Burned(1);
        assert!(!s.request_harvest(0));
        s.logs[1] = LogLocation::Burned(1);
        assert!(s.request_harvest(0));
        s.forestry.plots[1].generation = 1;
        assert!(!s.request_harvest(1));
    }
    #[test]
    fn actual_replanting_creates_new_timber_without_reviving_installed_or_burned_logs() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.request_build();
        s.request_harvest(0);
        w.advance(100., &mut s);
        assert!(s.construction.complete);
        assert_eq!(s.installed(), 2);
        assert!(s.request_planting(0));
        assert!(!s.request_planting(0));
        w.advance(0.25, &mut s);
        assert!(!s.forestry.plots[0].growing);
        w.advance(30., &mut s);
        assert!(s.forestry.plots[0].growing);
        assert!(s.trees[0].felled);
        assert!(!s.request_harvest(0));
        w.advance(1000., &mut s);
        assert!(!s.trees[0].felled);
        assert_eq!(s.forestry.plots[0].generation, 1);
        let before = s.logs.clone();
        assert!(s.request_harvest(0));
        w.advance(100., &mut s);
        assert_eq!(s.logs.len(), 8);
        assert_eq!(&s.logs[..6], before);
        assert_eq!(s.stored(), 2);
        assert_eq!(s.installed(), 2);
        assert_eq!(&s.forestry.origins[6..], &[0, 0]);
        assert!(s.forestry.valid(&s.logs, &s.trees));
        // The next cycle starts while the previous crop is now real spent wood.
        for i in 6..8 {
            s.logs[i] = LogLocation::Burned(1);
        }
        assert!(s.request_planting(0));
        w.advance(1100., &mut s);
        assert!(s.request_harvest(0));
        w.advance(100., &mut s);
        assert_eq!(&s.logs[6..8], &[LogLocation::Burned(1); 2]);
        assert_eq!(s.logs.len(), 10);
        assert_eq!(s.stored(), 2);
        assert_eq!(s.installed(), 2);
    }
    #[test]
    fn winter_stops_growth_and_partitioning_preserves_maturity() {
        let mut a = Settlement::default();
        a.trees[0].felled = true;
        a.request_planting(0);
        a.plant(0);
        let mut b = a.clone();
        let mut wa = Worker::default();
        let mut wb = Worker::default();
        wa.advance(360., &mut a);
        for _ in 0..1440 {
            wb.advance(0.25, &mut b);
        }
        assert_eq!(
            a.forestry.plots[0].generation,
            b.forestry.plots[0].generation
        );
        assert_eq!(a.trees, b.trees);
        // A new sapling planted exactly at winter waits for spring.
        a.trees[1].felled = true;
        a.request_planting(1);
        a.plant(1);
        wa.advance(120., &mut a);
        assert_eq!(a.forestry.plots[1].growth, 0.);
        wa.advance(120., &mut a);
        assert_eq!(a.forestry.plots[1].growth, 120.);
        let before = a.forestry.plots[1].growth;
        for bad in [f64::NAN, f64::INFINITY, -1., 0.] {
            a.forestry.advance(bad, &mut a.trees);
        }
        assert_eq!(a.forestry.plots[1].growth, before);
    }
    #[test]
    fn timber_priorities_choose_real_work_and_do_not_reallocate_committed_wood() {
        let mut base = Settlement::default();
        base.logs = [
            LogLocation::Stored,
            LogLocation::Stored,
            LogLocation::Ground,
            LogLocation::Ground,
            LogLocation::InTree,
            LogLocation::InTree,
        ]
        .to_vec();
        base.request_build();
        base.household.hearth.request();
        assert_eq!(base.timber_build_need(), 2);
        assert_eq!(base.timber_winter_need(), 3);
        for mode in [
            TimberPriority::Balanced,
            TimberPriority::Building,
            TimberPriority::Winter,
        ] {
            let mut s = base.clone();
            s.forestry.priority = mode;
            let mut w = Worker::default();
            w.advance(0.25, &mut s);
            assert_eq!(
                w.task,
                if mode == TimberPriority::Winter {
                    Task::LoadingFuel
                } else {
                    Task::Loading
                }
            );
            assert_eq!(s.logs, base.logs);
        }
        for mode in [TimberPriority::Balanced, TimberPriority::Building] {
            let mut plentiful = base.clone();
            plentiful.logs.fill(LogLocation::Stored);
            plentiful.forestry.priority = mode;
            let mut worker = Worker::default();
            worker.advance(0.25, &mut plentiful);
            assert_eq!(
                worker.task,
                if mode == TimberPriority::Building {
                    Task::Loading
                } else {
                    Task::LoadingFuel
                }
            );
        }
        let mut s = base;
        let mut w = Worker::default();
        s.forestry.priority = TimberPriority::Building;
        w.advance(1., &mut s);
        assert!(w.carrying.is_some());
        s.forestry.priority = TimberPriority::Winter;
        w.advance(5., &mut s);
        assert_eq!(s.installed(), 1); // In-flight construction remains committed.
        assert_eq!(
            s.logs
                .iter()
                .filter(|&&l| l == LogLocation::Installed)
                .count(),
            1
        );
    }
    #[test]
    fn stable_storage_slots_reuse_space_without_moving_other_logs() {
        let mut s = Settlement::default();
        s.logs.fill(LogLocation::Carried);
        s.place_log(0, LogLocation::Stored);
        s.place_log(1, LogLocation::Stored);
        let stable = s.forestry.slots[1];
        s.logs[0] = LogLocation::Burned(1);
        s.place_log(2, LogLocation::Stored);
        assert_eq!(s.forestry.slots[2], 0);
        assert_eq!(s.forestry.slots[1], stable);
        s.place_log(1, LogLocation::Home(2));
        assert_eq!(s.forestry.slots[1], 0);
        assert_eq!(s.forestry.slots[2], 0); // Distinct physical piles may use the same local slot.
        s.forestry.origins.pop();
        assert!(!s.forestry.valid(&s.logs, &s.trees));
    }
    #[test]
    fn two_households_can_keep_harvesting_and_heating_across_multiple_years() {
        use bevy::{prelude::*, time::TimeUpdateStrategy};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(250),
            ))
            .insert_resource(Time::<Fixed>::from_seconds(0.25))
            .init_resource::<crate::camera::CameraRig>()
            .add_plugins(crate::simulation::SimulationPlugin);
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.order_homestead();
            s.request_build();
            s.request_helper();
            s.household.hearth.request();
            for i in 0..3 {
                s.request_harvest(i);
            }
        }
        app.update();
        app.update();
        let founder = app
            .world_mut()
            .query::<(Entity, &Worker)>()
            .iter(app.world())
            .find(|(_, w)| w.id == 1)
            .unwrap()
            .0;
        for frame in 0..10000 {
            {
                let mut s = app.world_mut().resource_mut::<Settlement>();
                if s.homestead.complete && !s.homestead.invited {
                    s.order_homestead();
                }
                for i in 0..3 {
                    if s.trees[i].felled {
                        s.request_planting(i);
                    } else {
                        s.request_harvest(i);
                    }
                }
            }
            if frame % 41 == 0 {
                app.world_mut()
                    .resource_mut::<crate::camera::CameraRig>()
                    .toggle();
            }
            app.update();
        }
        let s = app.world().resource::<Settlement>();
        assert!(s.homestead.arrived && s.construction.complete);
        assert!(s.logs.len() >= 18, "{:?}", s.forestry);
        assert_eq!(s.installed(), 2);
        assert_eq!(s.homestead_timbers(), 2);
        assert!(s.household.hearth.burned >= 3 && s.homestead.household.hearth.burned >= 3);
        assert!(s.forestry.valid(&s.logs, &s.trees));
        assert!(app.world().get::<Worker>(founder).is_some());
        assert_eq!(
            app.world_mut().query::<&Worker>().iter(app.world()).count(),
            3
        );
    }
}
