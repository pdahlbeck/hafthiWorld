//! Fixed-step logistics, independent of cameras, input and rendering.
use bevy::prelude::*;

pub const STOCKPILE: Vec2 = Vec2::new(-2.5, 3.5);
pub const BUILD_SITE: Vec2 = Vec2::new(-4.4, 1.5);
const BUILD_WORK: Vec2 = Vec2::new(-3.3, 1.5);
pub const TREE_POSITIONS: [Vec2; 3] = [
    Vec2::new(-5.0, -2.5),
    Vec2::new(-3.5, -4.5),
    Vec2::new(-1.7, -4.5),
];
pub const LOGS_PER_TREE: usize = 2;
pub const TOTAL_LOGS: usize = TREE_POSITIONS.len() * LOGS_PER_TREE;
pub const BUILD_COST: usize = 2;
const SPEED: f32 = 1.2;
const HANDLING_SECONDS: f32 = 0.8;
const FELL_SECONDS: f32 = 3.0;
const BUILD_SECONDS: f32 = 3.0;

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settlement>()
            .init_resource::<SimulationClock>()
            .add_systems(Startup, spawn_worker)
            .add_systems(FixedUpdate, simulate);
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    pub ordered: bool,
    pub felled: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLocation {
    InTree,
    Ground,
    Carried,
    Stored,
    Installed,
}
#[derive(Debug, Default)]
pub struct Construction {
    pub ordered: bool,
    pub complete: bool,
}
#[derive(Resource, Debug)]
pub struct Settlement {
    pub trees: [Tree; TREE_POSITIONS.len()],
    pub logs: [LogLocation; TOTAL_LOGS],
    pub construction: Construction,
}
impl Default for Settlement {
    fn default() -> Self {
        Self {
            trees: [Tree::default(); TREE_POSITIONS.len()],
            logs: [LogLocation::InTree; TOTAL_LOGS],
            construction: Construction::default(),
        }
    }
}
impl Settlement {
    pub fn count(&self, location: LogLocation) -> usize {
        self.logs.iter().filter(|&&v| v == location).count()
    }
    pub fn stored(&self) -> usize {
        self.count(LogLocation::Stored)
    }
    pub fn installed(&self) -> usize {
        self.count(LogLocation::Installed)
    }
    pub fn request_harvest(&mut self, tree: usize) -> bool {
        let Some(t) = self.trees.get_mut(tree) else {
            return false;
        };
        if t.ordered || t.felled {
            return false;
        }
        t.ordered = true;
        true
    }
    pub fn request_build(&mut self) -> bool {
        if self.construction.ordered {
            return false;
        }
        self.construction.ordered = true;
        true
    }
}
pub fn ground_log_position(log: usize) -> Vec2 {
    TREE_POSITIONS[log / LOGS_PER_TREE]
        + Vec2::new((log % LOGS_PER_TREE) as f32 * 0.45 - 0.225, 0.5)
}
#[derive(Resource, Default)]
pub struct SimulationClock(pub f64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Idle,
    ToTree,
    Felling,
    ToLog,
    Loading,
    ToStockpile,
    Unloading,
    ToSite,
    Installing,
    Assembling,
}
#[derive(Component, Debug)]
pub struct Worker {
    pub position: Vec2,
    pub task: Task,
    pub carrying: Option<usize>,
    target: usize,
    handling: f32,
    building_delivery: bool,
}
impl Default for Worker {
    fn default() -> Self {
        Self {
            position: STOCKPILE,
            task: Task::Idle,
            carrying: None,
            target: 0,
            handling: 0.0,
            building_delivery: false,
        }
    }
}
impl Worker {
    fn start(&mut self, task: Task) {
        self.task = task;
        self.handling = 0.0;
    }
    fn choose_task(&mut self, settlement: &Settlement) -> bool {
        if settlement.construction.ordered && !settlement.construction.complete {
            if settlement.installed() == BUILD_COST {
                self.start(Task::ToSite);
                return true;
            }
            if let Some(log) = settlement
                .logs
                .iter()
                .position(|&v| v == LogLocation::Stored)
            {
                self.target = log;
                self.building_delivery = true;
                self.start(Task::ToLog);
                return true;
            }
        }
        if let Some(log) = settlement
            .logs
            .iter()
            .position(|&v| v == LogLocation::Ground)
        {
            self.target = log;
            self.building_delivery = false;
            self.start(Task::ToLog);
            return true;
        }
        if let Some(tree) = settlement.trees.iter().position(|t| t.ordered && !t.felled) {
            self.target = tree;
            self.start(Task::ToTree);
            return true;
        }
        false
    }
    pub fn advance(&mut self, mut seconds: f32, settlement: &mut Settlement) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        while seconds > 0.0 {
            match self.task {
                Task::Idle => {
                    if !self.choose_task(settlement) {
                        break;
                    }
                }
                Task::ToTree | Task::ToLog | Task::ToStockpile | Task::ToSite => {
                    let goal = match self.task {
                        Task::ToTree => TREE_POSITIONS[self.target] + Vec2::new(0.65, 0.45),
                        Task::ToLog if self.building_delivery => STOCKPILE,
                        Task::ToLog => ground_log_position(self.target),
                        Task::ToStockpile => STOCKPILE,
                        _ => BUILD_WORK,
                    };
                    let delta = goal - self.position;
                    let travel = delta.length() / SPEED;
                    if seconds < travel {
                        self.position += delta.normalize_or_zero() * SPEED * seconds;
                        break;
                    }
                    self.position = goal;
                    seconds -= travel;
                    self.start(match self.task {
                        Task::ToTree => Task::Felling,
                        Task::ToLog => Task::Loading,
                        Task::ToStockpile => Task::Unloading,
                        _ if self.carrying.is_some() => Task::Installing,
                        _ => Task::Assembling,
                    });
                }
                _ => {
                    let duration = match self.task {
                        Task::Felling => FELL_SECONDS,
                        Task::Assembling => BUILD_SECONDS,
                        _ => HANDLING_SECONDS,
                    };
                    let used = seconds.min(duration - self.handling);
                    self.handling += used;
                    seconds -= used;
                    if self.handling < duration {
                        break;
                    }
                    match self.task {
                        Task::Felling => {
                            settlement.trees[self.target].felled = true;
                            for log in
                                self.target * LOGS_PER_TREE..(self.target + 1) * LOGS_PER_TREE
                            {
                                settlement.logs[log] = LogLocation::Ground;
                            }
                            self.start(Task::Idle);
                        }
                        Task::Loading => {
                            let expected = if self.building_delivery {
                                LogLocation::Stored
                            } else {
                                LogLocation::Ground
                            };
                            if settlement.logs[self.target] != expected {
                                self.start(Task::Idle);
                                continue;
                            }
                            settlement.logs[self.target] = LogLocation::Carried;
                            self.carrying = Some(self.target);
                            self.start(if self.building_delivery {
                                Task::ToSite
                            } else {
                                Task::ToStockpile
                            });
                        }
                        Task::Unloading | Task::Installing => {
                            let log = self
                                .carrying
                                .take()
                                .expect("delivery requires a carried log");
                            settlement.logs[log] = if self.task == Task::Installing {
                                LogLocation::Installed
                            } else {
                                LogLocation::Stored
                            };
                            self.start(Task::Idle);
                        }
                        Task::Assembling => {
                            settlement.construction.complete = true;
                            self.start(Task::Idle);
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}
fn spawn_worker(mut commands: Commands) {
    commands.spawn(Worker::default());
}
fn simulate(
    time: Res<Time<Fixed>>,
    mut clock: ResMut<SimulationClock>,
    mut settlement: ResMut<Settlement>,
    mut workers: Query<&mut Worker>,
) {
    clock.0 += time.delta_secs_f64();
    for mut worker in &mut workers {
        worker.advance(time.delta_secs(), &mut settlement);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn assert_balance(s: &Settlement, w: &Worker) {
        assert_eq!(
            s.count(LogLocation::Carried),
            usize::from(w.carrying.is_some())
        );
        for (i, t) in s.trees.iter().enumerate() {
            let logs = &s.logs[i * LOGS_PER_TREE..(i + 1) * LOGS_PER_TREE];
            assert_eq!(
                logs.iter().filter(|&&l| l == LogLocation::InTree).count(),
                if t.felled { 0 } else { LOGS_PER_TREE }
            );
        }
        assert_eq!(
            [
                LogLocation::InTree,
                LogLocation::Ground,
                LogLocation::Carried,
                LogLocation::Stored,
                LogLocation::Installed
            ]
            .iter()
            .map(|&l| s.count(l))
            .sum::<usize>(),
            TOTAL_LOGS
        );
        assert!(s.installed() <= BUILD_COST);
        assert!(crate::world::walkable(w.position));
    }
    #[test]
    fn orders_are_required_and_repeated_orders_do_not_duplicate_material() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        w.advance(100.0, &mut s);
        assert_eq!(w.task, Task::Idle);
        assert_eq!(s.count(LogLocation::InTree), TOTAL_LOGS);
        assert!(!s.request_harvest(99));
        assert!(s.request_harvest(0));
        assert!(!s.request_harvest(0));
        w.advance(100.0, &mut s);
        assert_eq!(s.stored(), LOGS_PER_TREE);
        assert!(!s.request_harvest(0));
        assert_balance(&s, &w);
    }
    #[test]
    fn construction_waits_for_real_material_and_consumes_it_only_once() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        assert!(s.request_build());
        assert!(!s.request_build());
        w.advance(100.0, &mut s);
        assert!(!s.construction.complete);
        assert_eq!(s.installed(), 0);
        s.request_harvest(0);
        let mut carried_ground = false;
        let mut carried_stored = false;
        let mut assembling = false;
        for _ in 0..1600 {
            w.advance(0.1, &mut s);
            assert_balance(&s, &w);
            carried_ground |= w.task == Task::ToStockpile && w.carrying.is_some();
            carried_stored |= w.task == Task::ToSite && w.carrying.is_some();
            if w.task == Task::Assembling {
                assembling = true;
                assert!(!s.construction.complete);
                assert_eq!(s.installed(), BUILD_COST);
            }
        }
        assert!(carried_ground && carried_stored && assembling);
        assert!(s.construction.complete);
        assert_eq!(s.stored(), 0);
        assert_eq!(s.installed(), BUILD_COST);
        assert!(!s.request_build());
        w.advance(100.0, &mut s);
        assert_balance(&s, &w);
    }
    #[test]
    fn all_three_orders_preserve_material_and_stay_on_traversable_routes() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        for i in 0..TREE_POSITIONS.len() {
            s.request_harvest(i);
        }
        s.request_build();
        for _ in 0..2400 {
            w.advance(0.1, &mut s);
            assert_balance(&s, &w);
        }
        assert_eq!(s.stored(), TOTAL_LOGS - BUILD_COST);
        assert!(s.construction.complete);
    }
    #[test]
    fn frame_partition_and_camera_changes_do_not_reset_tasks_or_entities() {
        let mut a = Settlement::default();
        a.request_harvest(0);
        a.request_build();
        let mut wa = Worker::default();
        let mut b = Settlement::default();
        b.request_harvest(0);
        b.request_build();
        let mut wb = Worker::default();
        wa.advance(21.0, &mut a);
        for _ in 0..210 {
            wb.advance(0.1, &mut b);
        }
        assert_eq!(a.logs, b.logs);
        assert_eq!(wa.task, wb.task);
        assert!(wa.position.distance(wb.position) < 0.001);
        use crate::camera::CameraRig;
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<CameraRig>()
            .add_plugins(SimulationPlugin);
        app.update();
        let id = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.request_harvest(0);
            s.request_build();
        }
        for frame in 0..900 {
            if frame % 7 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            assert!(app.world().get::<Worker>(id).is_some());
        }
        assert!(app.world().resource::<Settlement>().construction.complete);
        assert!(app.world().resource::<SimulationClock>().0 > 80.0);
    }
}
