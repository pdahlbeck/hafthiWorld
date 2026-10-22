//! The coastal job's transport steps. Child of simulation, sharing its worker/cargo invariants.
use super::{
    HANDLING_SECONDS, HOME_DELIVERY, SPEED, STOCKPILE, SaltLocation, Settlement, Task, Worker,
};
use crate::fishing::{BATCH_SIZE, CoastalTask as C, FISHING_SPOT, FishLocation as F, SALTING_WORK};
use bevy::prelude::Vec2;
impl Worker {
    fn coastal_start(&mut self, task: C) {
        self.start(Task::Coastal(task));
    }
    fn coastal_move(&mut self, seconds: &mut f32, goal: Vec2, next: C) {
        let delta = goal - self.position;
        let travel = delta.length() / SPEED;
        if *seconds < travel {
            self.position += delta.normalize_or_zero() * SPEED * *seconds;
            *seconds = 0.0;
        } else {
            self.position = goal;
            *seconds -= travel;
            self.coastal_start(next);
        }
    }
    pub(super) fn advance_coastal(&mut self, seconds: &mut f32, s: &mut Settlement, task: C) {
        match task {
            C::ToFishingSpot => return self.coastal_move(seconds, FISHING_SPOT, C::Fishing),
            C::ReturningFresh => return self.coastal_move(seconds, STOCKPILE, C::StoringFresh),
            C::ToBrineStore => return self.coastal_move(seconds, STOCKPILE, C::LoadingBrineSalt),
            C::ToTableWithSalt => return self.coastal_move(seconds, SALTING_WORK, C::DepositSalt),
            C::BackForRaw => return self.coastal_move(seconds, STOCKPILE, C::LoadingRaw),
            C::ToTableWithRaw => return self.coastal_move(seconds, SALTING_WORK, C::DepositRaw),
            C::BackWithPreserved => {
                return self.coastal_move(seconds, STOCKPILE, C::StoringPreserved);
            }
            C::ToPickPreserved => {
                return self.coastal_move(seconds, SALTING_WORK, C::LoadingPreserved);
            }
            C::ToMealStore => return self.coastal_move(seconds, STOCKPILE, C::LoadingMeal),
            C::ToFishHome => return self.coastal_move(seconds, HOME_DELIVERY, C::UnloadingMeal),
            _ => {}
        }
        let duration = match task {
            C::Fishing => 8.0,
            C::Salting => 6.0,
            _ => HANDLING_SECONDS,
        };
        let used = (*seconds).min(duration - self.handling);
        *seconds -= used;
        self.handling += used;
        if self.handling < duration {
            return;
        }
        match task {
            C::Fishing => {
                assert!(s.fishery.catch(self.target));
                self.coastal_start(C::LoadingFresh);
            }
            C::LoadingFresh => {
                assert_eq!(s.fishery.fish[self.target], F::FreshAtShore);
                s.fishery.fish[self.target] = F::CarriedRaw;
                self.carrying_fish = Some(self.target);
                self.coastal_start(C::ReturningFresh);
            }
            C::StoringFresh => {
                let fish = self.carrying_fish.take().expect("fresh catch cargo");
                s.fishery.fish[fish] = F::RawStored;
                self.start(Task::Idle);
            }
            C::LoadingBrineSalt => {
                assert_eq!(s.fishery.salting_salt(&s.barter.salt), Some(self.target));
                s.barter.salt[self.target] = SaltLocation::Carried;
                self.carrying_salt = Some(self.target);
                self.coastal_start(C::ToTableWithSalt);
            }
            C::DepositSalt => {
                let salt = self.carrying_salt.take().expect("salting salt cargo");
                s.barter.salt[salt] = SaltLocation::AtSalting;
                self.target = s.fishery.first(F::RawStored).expect("reserved raw fish");
                self.coastal_start(C::BackForRaw);
            }
            C::LoadingRaw => {
                assert_eq!(s.fishery.fish[self.target], F::RawStored);
                s.fishery.fish[self.target] = F::CarriedRaw;
                self.carrying_fish = Some(self.target);
                self.coastal_start(C::ToTableWithRaw);
            }
            C::DepositRaw => {
                let fish = self.carrying_fish.take().expect("raw salting cargo");
                s.fishery.fish[fish] = F::RawAtTable;
                if s.fishery.count(F::RawAtTable) == BATCH_SIZE {
                    self.coastal_start(C::Salting);
                } else {
                    self.target = s.fishery.first(F::RawStored).expect("second raw fish");
                    self.coastal_start(C::BackForRaw);
                }
            }
            C::Salting => {
                assert!(s.fishery.preserve(&mut s.barter.salt));
                self.target = s.fishery.first(F::PreservedAtTable).expect("salted output");
                self.coastal_start(C::LoadingPreserved);
            }
            C::LoadingPreserved => {
                assert_eq!(s.fishery.fish[self.target], F::PreservedAtTable);
                s.fishery.fish[self.target] = F::CarriedPreserved;
                self.carrying_fish = Some(self.target);
                self.coastal_start(C::BackWithPreserved);
            }
            C::StoringPreserved => {
                let fish = self.carrying_fish.take().expect("preserved fish cargo");
                s.fishery.fish[fish] = F::PreservedStored;
                if let Some(next) = s.fishery.first(F::PreservedAtTable) {
                    self.target = next;
                    self.coastal_start(C::ToPickPreserved);
                } else {
                    self.start(Task::Idle);
                }
            }
            C::LoadingMeal => {
                assert_eq!(s.fishery.fish[self.target], F::PreservedStored);
                s.fishery.fish[self.target] = F::CarriedPreserved;
                self.carrying_fish = Some(self.target);
                self.coastal_start(C::ToFishHome);
            }
            C::UnloadingMeal => {
                let fish = self.carrying_fish.take().expect("home fish cargo");
                s.fishery.fish[fish] = F::Home(self.home_id);
                self.start(Task::FromHome);
            }
            _ => unreachable!("movement handled above"),
        }
    }
}
