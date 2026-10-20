//! Camera-independent material transport proof. No rendering or input belongs here.
use bevy::prelude::*;

pub const SUPPLY: Vec2 = Vec2::new(-5.0, -2.5);
pub const STOCKPILE: Vec2 = Vec2::new(-2.5, 3.5);
pub const INITIAL_LOGS: u32 = 6;
const SPEED: f32 = 1.2;
const HANDLING_SECONDS: f32 = 0.8;

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stocks>()
            .init_resource::<SimulationClock>()
            .add_systems(Startup, spawn_worker)
            .add_systems(FixedUpdate, simulate);
    }
}
#[derive(Resource, Debug)]
pub struct Stocks {
    pub supply: u32,
    pub stored: u32,
}
impl Default for Stocks {
    fn default() -> Self {
        Self {
            supply: INITIAL_LOGS,
            stored: 0,
        }
    }
}
#[derive(Resource, Default)]
pub struct SimulationClock(pub f64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    ToSupply,
    Loading,
    ToStockpile,
    Unloading,
    Complete,
}
#[derive(Component, Debug)]
pub struct Worker {
    pub position: Vec2,
    pub task: Task,
    pub carrying: bool,
    handling: f32,
}
impl Default for Worker {
    fn default() -> Self {
        Self {
            position: STOCKPILE,
            task: Task::ToSupply,
            carrying: false,
            handling: 0.0,
        }
    }
}
impl Worker {
    pub fn advance(&mut self, mut seconds: f32, stocks: &mut Stocks) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        while seconds > 0.0 {
            match self.task {
                Task::ToSupply | Task::ToStockpile => {
                    if self.task == Task::ToSupply && stocks.supply == 0 {
                        self.task = Task::Complete;
                        continue;
                    }
                    let goal = if self.task == Task::ToSupply {
                        SUPPLY
                    } else {
                        STOCKPILE
                    };
                    let delta = goal - self.position;
                    let travel = delta.length() / SPEED;
                    if seconds < travel {
                        self.position += delta.normalize_or_zero() * SPEED * seconds;
                        break;
                    }
                    self.position = goal;
                    seconds -= travel;
                    self.task = if self.task == Task::ToSupply {
                        Task::Loading
                    } else {
                        Task::Unloading
                    };
                    self.handling = 0.0;
                }
                Task::Loading | Task::Unloading => {
                    let used = seconds.min(HANDLING_SECONDS - self.handling);
                    self.handling += used;
                    seconds -= used;
                    if self.handling < HANDLING_SECONDS {
                        break;
                    }
                    if self.task == Task::Loading {
                        if stocks.supply == 0 {
                            self.task = Task::Complete;
                            continue;
                        }
                        stocks.supply -= 1;
                        self.carrying = true;
                        self.task = Task::ToStockpile;
                    } else {
                        stocks.stored += 1;
                        self.carrying = false;
                        self.task = if stocks.supply == 0 {
                            Task::Complete
                        } else {
                            Task::ToSupply
                        };
                    }
                }
                Task::Complete => break,
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
    mut stocks: ResMut<Stocks>,
    mut workers: Query<&mut Worker>,
) {
    clock.0 += time.delta_secs_f64();
    for mut worker in &mut workers {
        worker.advance(time.delta_secs(), &mut stocks);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_material_is_carried_before_being_stored() {
        let mut worker = Worker::default();
        let mut stocks = Stocks::default();
        let mut saw_carry = false;
        for _ in 0..2000 {
            worker.advance(0.1, &mut stocks);
            saw_carry |= worker.carrying;
            assert_eq!(
                stocks.supply + stocks.stored + u32::from(worker.carrying),
                INITIAL_LOGS
            );
        }
        assert!(saw_carry);
        assert_eq!(stocks.stored, INITIAL_LOGS);
        assert_eq!(worker.task, Task::Complete);
        worker.advance(1000.0, &mut stocks);
        assert_eq!(stocks.stored, INITIAL_LOGS);
    }
    #[test]
    fn elapsed_time_is_independent_of_frame_partition() {
        let mut a = Worker::default();
        let mut b = Worker::default();
        let mut sa = Stocks::default();
        let mut sb = Stocks::default();
        a.advance(41.0, &mut sa);
        for _ in 0..410 {
            b.advance(0.1, &mut sb);
        }
        assert_eq!(
            (sa.supply, sa.stored, a.carrying, a.task),
            (sb.supply, sb.stored, b.carrying, b.task)
        );
        assert!(a.position.distance(b.position) < 0.001);
    }
    #[test]
    fn camera_switches_do_not_replace_worker_or_reset_simulation() {
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
        for frame in 0..650 {
            if frame % 7 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            assert!(app.world().get::<Worker>(id).is_some());
            let s = app.world().resource::<Stocks>();
            let w = app.world().get::<Worker>(id).unwrap();
            assert_eq!(s.supply + s.stored + u32::from(w.carrying), INITIAL_LOGS);
        }
        assert!(app.world().resource::<Stocks>().stored > 0);
        assert!(app.world().resource::<SimulationClock>().0 > 60.0);
    }
}
