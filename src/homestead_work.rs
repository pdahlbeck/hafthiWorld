//! Child of simulation: same real worker, cargo fields and fixed world tick.
use super::*;
use crate::homestead::{DOOR, GATE, Job, Phase, TIMBER_COST};
impl Worker {
    fn house_start(&mut self, job: Job, phase: Phase) {
        self.start(Task::NewHome(job, phase));
    }
    pub(super) fn choose_new_build(&mut self, s: &Settlement) -> bool {
        if self.id != 1 || !s.homestead.ordered || s.homestead.complete {
            return false;
        }
        if s.homestead_timbers() == TIMBER_COST {
            self.house_start(Job::Build, Phase::ToGate);
            return true;
        }
        if s.timber_for_build()
            && let Some(i) = s.logs.iter().position(|&l| l == LogLocation::Stored)
        {
            self.target = i;
            self.house_start(Job::Build, Phase::ToStore);
            return true;
        }
        false
    }
    pub(super) fn choose_new_home(&mut self, s: &Settlement) -> bool {
        if !s.homestead.arrived {
            return false;
        }
        let job = if crate::homestead::food(s, 2) == 0 && s.homestead.household.supply_ordered {
            s.farm
                .grain
                .iter()
                .enumerate()
                .find(|(i, g)| **g == GrainLocation::Stored && s.farm.seed_candidate() != Some(*i))
                .map(|(i, _)| (Job::Grain, i))
                .or_else(|| {
                    s.fishery
                        .first(FishLocation::PreservedStored)
                        .map(|i| (Job::Fish, i))
                })
        } else {
            None
        };
        let job = job.or_else(|| {
            let target = if s.count(LogLocation::Home(1)) == 0 {
                1
            } else {
                crate::hearth::FUEL_TARGET
            };
            if s.timber_for_fuel()
                && s.homestead.household.hearth.ordered
                && s.count(LogLocation::Home(2)) < target
            {
                s.logs
                    .iter()
                    .position(|&l| l == LogLocation::Stored)
                    .map(|i| (Job::Fuel, i))
            } else {
                None
            }
        });
        if let Some((job, i)) = job {
            self.target = i;
            self.house_start(
                job,
                if self.position == STOCKPILE {
                    Phase::ToStore
                } else {
                    Phase::FetchGate
                },
            );
            return true;
        }
        false
    }
    pub(super) fn advance_new_home(
        &mut self,
        seconds: &mut f32,
        s: &mut Settlement,
        job: Job,
        phase: Phase,
    ) {
        let route = match phase {
            Phase::FetchGate => Some((GATE, Phase::ToStore)),
            Phase::ToStore => Some((STOCKPILE, Phase::Load)),
            Phase::ToGate => Some((GATE, Phase::ToDoor)),
            Phase::ToDoor => Some((
                DOOR,
                if job == Job::Build && self.carrying.is_none() {
                    Phase::Assemble
                } else {
                    Phase::Unload
                },
            )),
            Phase::ReturnGate => Some((GATE, Phase::ReturnStore)),
            Phase::ReturnStore => Some((STOCKPILE, Phase::Load)),
            _ => None,
        };
        if let Some((goal, next)) = route {
            let delta = goal - self.position;
            let travel = delta.length() / SPEED;
            if *seconds < travel {
                self.position += delta.normalize_or_zero() * SPEED * *seconds;
                *seconds = 0.;
            } else {
                self.position = goal;
                *seconds -= travel;
                if phase == Phase::ReturnStore {
                    self.start(Task::Idle);
                } else {
                    self.house_start(job, next);
                }
            }
            return;
        }
        let duration = if phase == Phase::Assemble {
            12.0
        } else {
            HANDLING_SECONDS
        };
        let used = (*seconds).min(duration - self.handling);
        self.handling += used;
        *seconds -= used;
        if self.handling < duration {
            return;
        }
        match phase {
            Phase::Load => {
                let loaded = match job {
                    Job::Build | Job::Fuel => {
                        if s.logs[self.target] != LogLocation::Stored
                            || (job == Job::Fuel && !s.timber_for_fuel())
                            || (job == Job::Build && !s.timber_for_build())
                        {
                            false
                        } else {
                            s.logs[self.target] = LogLocation::Carried;
                            self.carrying = Some(self.target);
                            true
                        }
                    }
                    Job::Grain => {
                        if s.farm.grain[self.target] != GrainLocation::Stored
                            || s.farm.seed_candidate() == Some(self.target)
                        {
                            false
                        } else {
                            s.farm.grain[self.target] = GrainLocation::Carried;
                            self.carrying_grain = Some(self.target);
                            true
                        }
                    }
                    Job::Fish => {
                        if s.fishery.fish[self.target] != FishLocation::PreservedStored {
                            false
                        } else {
                            s.fishery.fish[self.target] = FishLocation::CarriedPreserved;
                            self.carrying_fish = Some(self.target);
                            true
                        }
                    }
                };
                if loaded {
                    self.house_start(job, Phase::ToGate);
                } else {
                    self.start(Task::Idle);
                }
            }
            Phase::Unload => {
                match job {
                    Job::Build => {
                        let i = self.carrying.take().expect("construction timber");
                        s.logs[i] = LogLocation::Homestead;
                    }
                    Job::Fuel => {
                        let i = self.carrying.take().expect("owned firewood");
                        s.place_log(i, LogLocation::Home(2));
                    }
                    Job::Grain => {
                        let i = self.carrying_grain.take().expect("owned grain");
                        s.farm.grain[i] = GrainLocation::Home(2);
                    }
                    Job::Fish => {
                        let i = self.carrying_fish.take().expect("owned fish");
                        s.fishery.fish[i] = FishLocation::Home(2);
                    }
                }
                self.house_start(job, Phase::ReturnGate);
            }
            Phase::Assemble => {
                if s.homestead_timbers() == TIMBER_COST {
                    s.homestead.complete = true;
                }
                self.house_start(job, Phase::ReturnGate);
            }
            _ => unreachable!(),
        }
    }
}
