//! Accelerated gameplay seasons, independent of presentation sunlight.
pub const SEASON_SECONDS: f64 = 120.0;
pub const YEAR_SECONDS: f64 = SEASON_SECONDS * 4.0;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}
impl Season {
    pub fn label(self) -> &'static str {
        match self {
            Self::Spring => "spring",
            Self::Summer => "summer",
            Self::Autumn => "autumn",
            Self::Winter => "winter",
        }
    }
}
#[derive(Debug)]
pub struct Calendar {
    pub year: u64,
    elapsed: f64,
}
impl Default for Calendar {
    fn default() -> Self {
        Self {
            year: 1,
            elapsed: 0.0,
        }
    }
}
impl Calendar {
    pub fn season(&self) -> Season {
        match (self.elapsed / SEASON_SECONDS) as u32 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
    pub fn can_sow(&self) -> bool {
        matches!(self.season(), Season::Spring | Season::Summer)
    }
    pub fn remaining(&self) -> f64 {
        SEASON_SECONDS - self.elapsed % SEASON_SECONDS
    }
    /// Returns only nonwinter growth time, even when a long update spans years.
    pub fn advance(&mut self, seconds: f64) -> f64 {
        if !seconds.is_finite() || seconds <= 0.0 {
            return 0.0;
        }
        let total = self.elapsed + seconds;
        let years = (total / YEAR_SECONDS).floor();
        let end = total % YEAR_SECONDS;
        let active = years * (3.0 * SEASON_SECONDS) + end.min(3.0 * SEASON_SECONDS)
            - self.elapsed.min(3.0 * SEASON_SECONDS);
        self.elapsed = end;
        self.year = self.year.saturating_add(years as u64);
        active.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_calendar_uses_world_ticks_with_no_workers_and_ignores_camera_and_sun() {
        use crate::{
            camera::CameraRig,
            simulation::{Settlement, SimulationClock, SimulationPlugin, Worker},
            world::Daylight,
        };
        use bevy::{prelude::*, time::TimeUpdateStrategy};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(250),
            ))
            .init_resource::<CameraRig>()
            .init_resource::<Daylight>()
            .add_plugins(SimulationPlugin);
        app.update();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        app.world_mut().despawn(worker);
        for frame in 0..2000 {
            if frame % 9 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            {
                let mut sun = app.world_mut().resource_mut::<Daylight>();
                sun.running = false;
                sun.hour = (frame % 24) as f32;
            }
            app.update();
        }
        let seconds = app.world().resource::<SimulationClock>().0;
        let c = &app.world().resource::<Settlement>().calendar;
        assert!(seconds >= YEAR_SECONDS);
        assert_eq!(c.year, 2);
        assert_eq!(
            app.world().resource::<Settlement>().household.hearth.missed,
            3
        );
        assert_eq!(c.season(), Season::Spring);
        assert!((c.remaining() - (SEASON_SECONDS - seconds % YEAR_SECONDS)).abs() < 0.001);
        app.world_mut().spawn(Worker::default());
        app.world_mut().spawn(Worker::for_home(2));
        for _ in 0..480 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Settlement>().calendar.season(),
            Season::Summer
        );
        assert_eq!(app.world().resource::<Settlement>().calendar.year, 2);
    }
    #[test]
    fn boundaries_wrap_years_and_reject_invalid_time() {
        let mut c = Calendar::default();
        for season in [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            assert_eq!(c.season(), season);
            assert_eq!(c.remaining(), SEASON_SECONDS);
            assert_eq!(
                c.can_sow(),
                matches!(season, Season::Spring | Season::Summer)
            );
            assert_eq!(
                c.advance(SEASON_SECONDS),
                if season == Season::Winter {
                    0.0
                } else {
                    SEASON_SECONDS
                }
            );
        }
        assert_eq!(c.year, 2);
        assert_eq!(c.season(), Season::Spring);
        for invalid in [f64::NAN, f64::INFINITY, -2.0, 0.0] {
            assert_eq!(c.advance(invalid), 0.0);
        }
        assert_eq!(c.year, 2);
        assert_eq!(c.remaining(), SEASON_SECONDS);
    }
    #[test]
    fn growth_time_integrates_winter_boundaries_and_multiple_years() {
        let mut c = Calendar::default();
        assert_eq!(c.advance(350.0), 350.0);
        assert_eq!(c.advance(140.0), 20.0); // 10 autumn + 120 winter + 10 spring.
        assert_eq!(c.season(), Season::Spring);
        assert_eq!(c.year, 2);
        let mut a = Calendar::default();
        let mut b = Calendar::default();
        let whole = a.advance(1445.0);
        let mut partitioned = 0.0;
        for _ in 0..5780 {
            partitioned += b.advance(0.25);
        }
        assert_eq!(whole, 1085.0);
        assert_eq!(whole, partitioned);
        assert_eq!(a.year, b.year);
        assert_eq!(a.remaining(), b.remaining());
    }
    #[test]
    fn winter_keeps_crops_and_stocks_while_sow_order_waits_until_spring() {
        use crate::{
            farming::{CropPhase, GrainLocation, SeedLocation},
            simulation::{Settlement, Task, Worker},
        };
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.calendar.advance(360.0);
        assert!(s.farm.request_sow());
        w.advance(100.0, &mut s);
        assert_eq!(s.farm.phase, CropPhase::SowOrdered);
        assert_eq!(s.farm.seeds, SeedLocation::Stored);
        assert_eq!(w.task, Task::Idle);
        assert_eq!(s.farm.count(GrainLocation::Unproduced), 4);
        w.advance(45.0, &mut s);
        assert_eq!(s.calendar.year, 2);
        assert_eq!(s.farm.seeds, SeedLocation::Sown);
        assert_eq!(s.farm.phase, CropPhase::Growing);
        let mut winter = Settlement::default();
        let mut idle = Worker::default();
        winter.calendar.advance(360.0);
        winter.farm.phase = CropPhase::Growing;
        winter.farm.growth = 12.0;
        winter.farm.seeds = SeedLocation::Sown;
        idle.advance(119.0, &mut winter);
        assert_eq!(winter.farm.growth, 12.0);
        idle.advance(3.0, &mut winter);
        assert_eq!(winter.farm.growth, 14.0);
    }
}
