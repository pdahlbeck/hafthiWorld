//! Fixed-step logistics, independent of cameras, input and rendering.
#[path = "coastal_work.rs"]
mod coastal_work;
use crate::calendar::Calendar;
use crate::farming::{CropPhase, FIELD_WORK, Farm, GrainLocation, SeedLocation, grain_position};
use crate::fishing::{CoastalTask, FishLocation, Fishery};
use crate::household::{HOME_DELIVERY, HOME_ID, Household};
use crate::livestock::{Flock, PASTURE_GATE, WOOL_PER_SHEEP, WoolLocation};
use crate::trade::{Barter, SaltLocation, TRADE_WORK};
use bevy::prelude::*;

pub const COASTAL_REST: Vec2 = Vec2::new(-3.2, 3.5);
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
            .init_resource::<crate::session::Session>()
            .add_systems(Startup, spawn_worker)
            .add_systems(
                FixedUpdate,
                (invite_helper, simulate)
                    .chain()
                    .run_if(crate::session::running),
            );
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tree {
    pub ordered: bool,
    pub felled: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LogLocation {
    InTree,
    Ground,
    Carried,
    Stored,
    Installed,
    Home(u32),
    Burned(u32),
}
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct Construction {
    pub ordered: bool,
    pub complete: bool,
}
#[derive(Resource, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Settlement {
    pub trees: [Tree; TREE_POSITIONS.len()],
    pub logs: [LogLocation; TOTAL_LOGS],
    pub construction: Construction,
    pub farm: Farm,
    pub flock: Flock,
    pub household: Household,
    pub barter: Barter,
    pub calendar: Calendar,
    pub fishery: Fishery,
    #[serde(default)]
    pub voyage: crate::shipping::Voyage,
    pub helper_requested: bool,
    pub coastal_assigned: bool,
    pub coastal_owner: Option<u32>,
}
impl Default for Settlement {
    fn default() -> Self {
        Self {
            trees: [Tree::default(); TREE_POSITIONS.len()],
            logs: [LogLocation::InTree; TOTAL_LOGS],
            construction: Construction::default(),
            farm: Farm::default(),
            flock: Flock::default(),
            household: Household::default(),
            barter: Barter::default(),
            calendar: Calendar::default(),
            fishery: Fishery::default(),
            voyage: crate::shipping::Voyage::default(),
            helper_requested: false,
            coastal_assigned: false,
            coastal_owner: None,
        }
    }
}
impl Settlement {
    pub fn request_helper(&mut self) -> bool {
        if self.helper_requested {
            return false;
        }
        self.helper_requested = true;
        true
    }
    fn advance_environment(&mut self, seconds: f64) {
        self.voyage
            .advance(seconds, &mut self.flock.wool, &self.barter.salt);
        let growth_seconds = self.calendar.advance(seconds);
        self.farm.grow(growth_seconds);
        self.household.hearth.advance(
            (seconds - growth_seconds).max(0.0),
            self.household.id,
            &mut self.logs,
        );
        self.flock.graze(seconds);
        self.household
            .advance_food(seconds, &mut self.farm.grain, &mut self.fishery.fish);
    }
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
#[derive(Resource, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct SimulationClock(pub f64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Task {
    Coastal(CoastalTask),
    ToCrewRest,
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
    ToFarmGateway,
    LoadingSeeds,
    ToSeedStore,
    PreparingSeeds,
    ToField,
    Sowing,
    Reaping,
    LoadingGrain,
    DeliveringGrain,
    StoringGrain,
    FromField,
    ToPastureGateway,
    ToPastureEdge,
    ToPasture,
    ToSheep,
    Shearing,
    ToWool,
    LoadingWool,
    FromPasture,
    FromPastureEdge,
    DeliveringWool,
    StoringWool,
    ToFoodStore,
    LoadingFood,
    ToHome,
    UnloadingFood,
    FromHome,
    ToFuelStore,
    LoadingFuel,
    ToFuelHome,
    UnloadingFuel,
    ToTradeStore,
    LoadingTradeWool,
    ToTrader,
    HandingTradeWool,
    LoadingSalt,
    FromTrader,
    StoringSalt,
}
impl Task {
    pub fn trading(self) -> bool {
        matches!(
            self,
            Self::ToTradeStore
                | Self::LoadingTradeWool
                | Self::ToTrader
                | Self::HandingTradeWool
                | Self::LoadingSalt
                | Self::FromTrader
                | Self::StoringSalt
        )
    }
    pub fn tending_sheep(self) -> bool {
        matches!(
            self,
            Self::ToPastureGateway
                | Self::ToPastureEdge
                | Self::ToPasture
                | Self::ToSheep
                | Self::Shearing
                | Self::ToWool
                | Self::LoadingWool
                | Self::FromPasture
                | Self::FromPastureEdge
                | Self::DeliveringWool
                | Self::StoringWool
        )
    }
}
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum FarmJob {
    Sow,
    Reap,
    Haul,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum WorkerRole {
    #[default]
    General,
    Coastal,
}
#[derive(Component, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Worker {
    pub id: u32,
    pub role: WorkerRole,
    #[serde(with = "crate::session::vector")]
    pub position: Vec2,
    pub home_id: u32,
    pub task: Task,
    pub carrying: Option<usize>,
    pub carrying_grain: Option<usize>,
    pub carrying_wool: Option<usize>,
    pub carrying_salt: Option<usize>,
    pub carrying_fish: Option<usize>,
    sheep_hauling: bool,
    farm_job: FarmJob,
    target: usize,
    handling: f32,
    building_delivery: bool,
}
impl Default for Worker {
    fn default() -> Self {
        Self {
            id: 1,
            role: WorkerRole::General,
            position: STOCKPILE,
            home_id: HOME_ID,
            task: Task::Idle,
            carrying: None,
            carrying_grain: None,
            carrying_wool: None,
            carrying_salt: None,
            carrying_fish: None,
            sheep_hauling: false,
            farm_job: FarmJob::Sow,
            target: 0,
            handling: 0.0,
            building_delivery: false,
        }
    }
}
impl Worker {
    pub(crate) fn valid_snapshot(&self) -> bool {
        let limit = match self.task {
            Task::ToTree | Task::Felling => TREE_POSITIONS.len(),
            Task::ToSheep | Task::Shearing => crate::livestock::SHEEP_COUNT,
            Task::LoadingSalt => crate::trade::SALT_UNITS,
            Task::Coastal(
                crate::fishing::CoastalTask::ToExportPile
                | crate::fishing::CoastalTask::LoadingExportWool
                | crate::fishing::CoastalTask::ToBoat
                | crate::fishing::CoastalTask::UnloadingExportWool
                | crate::fishing::CoastalTask::ReturnFromBoat,
            ) => crate::livestock::WOOL_UNITS,
            Task::Coastal(crate::fishing::CoastalTask::LoadingBrineSalt) => {
                crate::trade::SALT_UNITS
            }
            Task::Coastal(_) => crate::fishing::FISH_UNITS,
            Task::ToWool | Task::LoadingWool | Task::ToTradeStore | Task::LoadingTradeWool => {
                crate::livestock::WOOL_UNITS
            }
            Task::LoadingGrain | Task::ToFoodStore | Task::LoadingFood | Task::PreparingSeeds => {
                crate::farming::GRAIN_UNITS
            }
            _ => TOTAL_LOGS,
        };
        self.target < limit
            && self.position.is_finite()
            && self.handling.is_finite()
            && self.handling >= 0.0
            && self.home_id == HOME_ID
            && [
                self.carrying,
                self.carrying_grain,
                self.carrying_wool,
                self.carrying_salt,
                self.carrying_fish,
            ]
            .iter()
            .filter(|v| v.is_some())
            .count()
                <= 1
    }

    pub fn for_home(home_id: u32) -> Self {
        Self {
            home_id,
            ..Self::default()
        }
    }
    fn start(&mut self, task: Task) {
        self.task = task;
        self.handling = 0.0;
    }
    fn can_take_coastal(&self, s: &Settlement) -> bool {
        (!s.coastal_assigned || self.role == WorkerRole::Coastal)
            && (s.coastal_owner.is_none() || s.coastal_owner == Some(self.id))
    }
    fn coastal_job(&mut self, s: &mut Settlement, target: usize, task: CoastalTask) -> bool {
        s.coastal_owner = Some(self.id);
        self.target = target;
        self.start(Task::Coastal(task));
        true
    }
    fn choose_coastal_supply(&mut self, s: &mut Settlement) -> bool {
        if self.can_take_coastal(s)
            && s.household.supply_ordered
            && self.home_id == s.household.id
            && let Some(fish) = s.fishery.first(FishLocation::PreservedStored)
        {
            return self.coastal_job(s, fish, CoastalTask::ToMealStore);
        }
        false
    }
    fn choose_salting(&mut self, s: &mut Settlement) -> bool {
        if self.can_take_coastal(s)
            && let Some(salt) = s.fishery.salting_salt(&s.barter.salt)
        {
            return self.coastal_job(s, salt, CoastalTask::ToBrineStore);
        }
        false
    }
    fn choose_fishing(&mut self, s: &mut Settlement) -> bool {
        if self.can_take_coastal(s)
            && s.fishery.fishing_ordered
            && let Some(fish) = s.fishery.first(FishLocation::Uncaught)
        {
            return self.coastal_job(s, fish, CoastalTask::ToFishingSpot);
        }
        false
    }
    fn choose_shipment(&mut self, s: &mut Settlement) -> bool {
        if self.can_take_coastal(s)
            && s.voyage.phase == crate::shipping::VoyagePhase::Loading
            && let Some(wool) = s.flock.wool.iter().position(|&w| w == WoolLocation::Trader)
        {
            return self.coastal_job(s, wool, CoastalTask::ToExportPile);
        }
        false
    }
    fn choose_task(&mut self, settlement: &mut Settlement) -> bool {
        if settlement.coastal_owner == Some(self.id) {
            settlement.coastal_owner = None;
        }
        if self.role == WorkerRole::Coastal {
            if self.choose_coastal_supply(settlement)
                || self.choose_salting(settlement)
                || self.choose_shipment(settlement)
                || self.choose_fishing(settlement)
            {
                return true;
            }
            if self.position.distance_squared(COASTAL_REST) > 0.0001 {
                self.start(Task::ToCrewRest);
                return true;
            }
            return false;
        }
        if settlement.household.hearth.ordered
            && self.home_id == settlement.household.id
            && settlement.count(LogLocation::Home(self.home_id)) < crate::hearth::FUEL_TARGET
            && let Some(log) = settlement
                .logs
                .iter()
                .position(|&l| l == LogLocation::Stored)
        {
            self.target = log;
            self.start(Task::ToFuelStore);
            return true;
        }
        if let Some(grain) = settlement.farm.seed_candidate() {
            self.target = grain;
            self.start(Task::ToSeedStore);
            return true;
        }
        if settlement.household.supply_ordered
            && self.home_id == settlement.household.id
            && let Some(grain) = settlement
                .farm
                .grain
                .iter()
                .position(|&g| g == GrainLocation::Stored)
        {
            self.target = grain;
            self.start(Task::ToFoodStore);
            return true;
        }
        if self.choose_coastal_supply(settlement)
            || self.choose_salting(settlement)
            || self.choose_shipment(settlement)
        {
            return true;
        }
        if let Some(wool) = settlement.barter.wool_candidate(&settlement.flock.wool) {
            self.target = wool;
            self.start(Task::ToTradeStore);
            return true;
        }
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
        let farm = &settlement.farm;
        if let Some(grain) = farm.grain.iter().position(|&g| g == GrainLocation::Ground) {
            self.target = grain;
            self.farm_job = FarmJob::Haul;
        } else if farm.phase == CropPhase::SowOrdered
            && farm.seeds == SeedLocation::Stored
            && settlement.calendar.can_sow()
        {
            self.farm_job = FarmJob::Sow;
        } else if farm.phase == CropPhase::Ripe && farm.harvest_ordered {
            self.farm_job = FarmJob::Reap;
        } else {
            return self.choose_sheep_task(settlement);
        }
        self.start(Task::ToFarmGateway);
        true
    }
    fn choose_sheep_task(&mut self, settlement: &mut Settlement) -> bool {
        if let Some(wool) = settlement
            .flock
            .wool
            .iter()
            .position(|&w| w == WoolLocation::Ground)
        {
            self.target = wool;
            self.sheep_hauling = true;
        } else if let Some(sheep) = settlement
            .flock
            .sheep
            .iter()
            .position(|s| s.ordered && !s.shorn)
        {
            self.target = sheep;
            self.sheep_hauling = false;
        } else {
            return self.choose_fishing(settlement);
        }
        self.start(Task::ToPastureGateway);
        true
    }
    // Pure single-worker harness; the runtime advances the environment once per world tick.
    #[cfg(test)]
    pub fn advance(&mut self, mut seconds: f32, settlement: &mut Settlement) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        while seconds > 0.0 {
            if self.task == Task::Idle
                && !self.choose_task(settlement)
                && settlement.farm.phase != CropPhase::SowOrdered
            {
                settlement.advance_environment(seconds as f64);
                return;
            }
            let step = seconds.min(1.0 / 64.0);
            settlement.advance_environment(step as f64);
            self.advance_work(step, settlement);
            seconds -= step;
        }
    }
    fn advance_work(&mut self, mut seconds: f32, settlement: &mut Settlement) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        while seconds > 0.0 {
            match self.task {
                Task::Coastal(task) => self.advance_coastal(&mut seconds, settlement, task),
                Task::Idle => {
                    if !self.choose_task(settlement) {
                        break;
                    }
                }
                Task::ToCrewRest
                | Task::ToTree
                | Task::ToLog
                | Task::ToStockpile
                | Task::ToSite
                | Task::ToFarmGateway
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
                | Task::ToTradeStore
                | Task::ToTrader
                | Task::FromTrader
                | Task::ToFuelStore
                | Task::ToFuelHome
                | Task::ToSeedStore
                | Task::ToFoodStore
                | Task::ToHome
                | Task::FromHome => {
                    let goal = match self.task {
                        Task::ToCrewRest => COASTAL_REST,
                        Task::ToTradeStore | Task::FromTrader => STOCKPILE,
                        Task::ToTrader => TRADE_WORK,
                        Task::ToSeedStore | Task::ToFoodStore | Task::FromHome => STOCKPILE,
                        Task::ToFuelStore => STOCKPILE,
                        Task::ToHome | Task::ToFuelHome => HOME_DELIVERY,
                        Task::ToPastureGateway | Task::DeliveringWool => STOCKPILE,
                        Task::ToPastureEdge | Task::FromPastureEdge => FIELD_WORK,
                        Task::ToPasture | Task::FromPasture => PASTURE_GATE,
                        Task::ToSheep => {
                            settlement.flock.sheep[self.target].position + Vec2::new(0.0, 0.75)
                        }
                        Task::ToWool => settlement.flock.ground[self.target],
                        Task::ToTree => TREE_POSITIONS[self.target] + Vec2::new(0.65, 0.45),
                        Task::ToLog if self.building_delivery => STOCKPILE,
                        Task::ToLog => ground_log_position(self.target),
                        Task::ToStockpile
                        | Task::ToFarmGateway
                        | Task::DeliveringGrain
                        | Task::FromField => STOCKPILE,
                        Task::ToField if matches!(self.farm_job, FarmJob::Haul) => {
                            grain_position(self.target)
                        }
                        Task::ToField => FIELD_WORK,
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
                        Task::ToCrewRest => Task::Idle,
                        Task::ToTradeStore => Task::LoadingTradeWool,
                        Task::ToTrader => Task::HandingTradeWool,
                        Task::FromTrader if self.carrying_salt.is_some() => Task::StoringSalt,
                        Task::FromTrader => Task::Idle,
                        Task::ToFuelStore => Task::LoadingFuel,
                        Task::ToFuelHome => Task::UnloadingFuel,
                        Task::ToSeedStore => Task::PreparingSeeds,
                        Task::ToFoodStore => Task::LoadingFood,
                        Task::ToHome => Task::UnloadingFood,
                        Task::FromHome => Task::Idle,
                        Task::ToPastureGateway => Task::ToPastureEdge,
                        Task::ToPastureEdge => Task::ToPasture,
                        Task::ToPasture if self.sheep_hauling => Task::ToWool,
                        Task::ToPasture => Task::ToSheep,
                        Task::ToSheep => Task::Shearing,
                        Task::ToWool => Task::LoadingWool,
                        Task::FromPasture => Task::FromPastureEdge,
                        Task::FromPastureEdge => Task::DeliveringWool,
                        Task::DeliveringWool => Task::StoringWool,
                        Task::ToFarmGateway if matches!(self.farm_job, FarmJob::Sow) => {
                            Task::LoadingSeeds
                        }
                        Task::ToFarmGateway => Task::ToField,
                        Task::ToField => match self.farm_job {
                            FarmJob::Sow => Task::Sowing,
                            FarmJob::Reap => Task::Reaping,
                            FarmJob::Haul => Task::LoadingGrain,
                        },
                        Task::DeliveringGrain => Task::StoringGrain,
                        Task::FromField => Task::Idle,
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
                        Task::PreparingSeeds | Task::Sowing | Task::Reaping | Task::Shearing => 4.0,
                        _ => HANDLING_SECONDS,
                    };
                    let used = seconds.min(duration - self.handling);
                    self.handling += used;
                    seconds -= used;
                    if self.handling < duration {
                        break;
                    }
                    match self.task {
                        Task::LoadingFuel => {
                            if settlement.logs[self.target] != LogLocation::Stored
                                || settlement.count(LogLocation::Home(self.home_id))
                                    >= crate::hearth::FUEL_TARGET
                            {
                                self.start(Task::Idle);
                                continue;
                            }
                            settlement.logs[self.target] = LogLocation::Carried;
                            self.carrying = Some(self.target);
                            self.start(Task::ToFuelHome);
                        }
                        Task::UnloadingFuel => {
                            let log = self.carrying.take().expect("fuel delivery requires a log");
                            settlement.logs[log] = LogLocation::Home(self.home_id);
                            self.start(Task::FromHome);
                        }
                        Task::LoadingTradeWool => {
                            if settlement.barter.wool_candidate(&settlement.flock.wool)
                                != Some(self.target)
                            {
                                self.start(Task::Idle);
                                continue;
                            }
                            settlement.flock.wool[self.target] = WoolLocation::Carried;
                            self.carrying_wool = Some(self.target);
                            self.start(Task::ToTrader);
                        }
                        Task::HandingTradeWool => {
                            let wool = self
                                .carrying_wool
                                .take()
                                .expect("trade delivery requires wool");
                            assert!(
                                settlement
                                    .barter
                                    .deliver_wool(wool, &mut settlement.flock.wool)
                            );
                            if let Some(salt) = settlement.barter.paid_salt() {
                                self.target = salt;
                                self.start(Task::LoadingSalt);
                            } else {
                                self.start(Task::FromTrader);
                            }
                        }
                        Task::LoadingSalt => {
                            assert_eq!(settlement.barter.salt[self.target], SaltLocation::Paid);
                            settlement.barter.salt[self.target] = SaltLocation::Carried;
                            self.carrying_salt = Some(self.target);
                            self.start(Task::FromTrader);
                        }
                        Task::StoringSalt => {
                            let salt = self
                                .carrying_salt
                                .take()
                                .expect("salt delivery requires cargo");
                            settlement.barter.salt[salt] = SaltLocation::Stored;
                            self.start(Task::Idle);
                        }
                        Task::PreparingSeeds => {
                            settlement.farm.save_seed(self.target);
                            self.start(Task::Idle);
                        }
                        Task::LoadingFood => {
                            if settlement.farm.grain[self.target] != GrainLocation::Stored {
                                self.start(Task::Idle);
                                continue;
                            }
                            settlement.farm.grain[self.target] = GrainLocation::Carried;
                            self.carrying_grain = Some(self.target);
                            self.start(Task::ToHome);
                        }
                        Task::UnloadingFood => {
                            let grain = self
                                .carrying_grain
                                .take()
                                .expect("home delivery requires cargo");
                            settlement.farm.grain[grain] = GrainLocation::Home(self.home_id);
                            self.start(Task::FromHome);
                        }
                        Task::Shearing => {
                            assert!(
                                settlement.flock.clip(self.target),
                                "ordered fleece must be available"
                            );
                            self.target *= WOOL_PER_SHEEP;
                            self.sheep_hauling = true;
                            self.start(Task::ToWool);
                        }
                        Task::LoadingWool => {
                            settlement.flock.wool[self.target] = WoolLocation::Carried;
                            self.carrying_wool = Some(self.target);
                            self.start(Task::FromPasture);
                        }
                        Task::StoringWool => {
                            let wool = self
                                .carrying_wool
                                .take()
                                .expect("wool delivery needs cargo");
                            settlement.flock.wool[wool] = WoolLocation::Stored;
                            self.start(Task::Idle);
                        }
                        Task::LoadingSeeds => {
                            settlement.farm.seeds = SeedLocation::Carried;
                            self.start(Task::ToField);
                        }
                        Task::Sowing => {
                            settlement.farm.seeds = SeedLocation::Sown;
                            settlement.farm.phase = CropPhase::Growing;
                            self.start(Task::FromField);
                        }
                        Task::Reaping => {
                            assert!(settlement.farm.reap(), "reaping cannot replace live grain");
                            self.target = 0;
                            self.farm_job = FarmJob::Haul;
                            self.start(Task::ToField);
                        }
                        Task::LoadingGrain => {
                            settlement.farm.grain[self.target] = GrainLocation::Carried;
                            self.carrying_grain = Some(self.target);
                            self.start(Task::DeliveringGrain);
                        }
                        Task::StoringGrain => {
                            let grain = self
                                .carrying_grain
                                .take()
                                .expect("grain delivery requires cargo");
                            settlement.farm.grain[grain] = GrainLocation::Stored;
                            self.start(Task::Idle);
                        }
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
fn spawn_worker(mut commands: Commands, settlement: Res<Settlement>) {
    commands.spawn(Worker::for_home(settlement.household.id));
}
fn invite_helper(mut commands: Commands, mut settlement: ResMut<Settlement>) {
    if settlement.helper_requested && !settlement.coastal_assigned {
        let mut helper = Worker::for_home(settlement.household.id);
        helper.id = 2;
        helper.role = WorkerRole::Coastal;
        helper.position = TRADE_WORK;
        helper.start(Task::FromHome); // Walk the clear shore-to-store approach before taking work.
        commands.spawn(helper);
        settlement.coastal_assigned = true;
    }
}
fn simulate(
    time: Res<Time<Fixed>>,
    mut clock: ResMut<SimulationClock>,
    mut settlement: ResMut<Settlement>,
    mut workers: Query<&mut Worker>,
) {
    clock.0 += time.delta_secs_f64();
    settlement.advance_environment(time.delta_secs_f64());
    let mut crew: Vec<_> = workers.iter_mut().collect();
    crew.sort_by_key(|w| w.id);
    for mut worker in crew {
        worker.advance_work(time.delta_secs(), &mut settlement);
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
    fn two_tick(s: &mut Settlement, a: &mut Worker, b: &mut Worker, dt: f32) {
        s.advance_environment(dt as f64);
        a.advance_work(dt, s);
        b.advance_work(dt, s);
        assert_eq!(
            s.fishery.count(FishLocation::CarriedRaw)
                + s.fishery.count(FishLocation::CarriedPreserved),
            usize::from(a.carrying_fish.is_some()) + usize::from(b.carrying_fish.is_some())
        );
        assert_eq!(
            s.barter.count(SaltLocation::Carried),
            usize::from(a.carrying_salt.is_some()) + usize::from(b.carrying_salt.is_some())
        );
        assert!(!(matches!(a.task, Task::Coastal(_)) && matches!(b.task, Task::Coastal(_))));
        for w in [&*a, &*b] {
            assert!(
                usize::from(w.carrying.is_some())
                    + usize::from(w.carrying_grain.is_some())
                    + usize::from(w.carrying_wool.is_some())
                    + usize::from(w.carrying_salt.is_some())
                    + usize::from(w.carrying_fish.is_some())
                    <= 1
            );
            assert!(
                crate::world::walkable(w.position),
                "blocked {:?} at {:?}",
                w.task,
                w.position
            );
        }
    }
    #[test]
    fn inviting_a_resident_does_not_double_world_household_or_winter_clocks() {
        use bevy::time::TimeUpdateStrategy;
        fn app(invite: bool) -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .insert_resource(TimeUpdateStrategy::ManualDuration(
                    std::time::Duration::from_millis(250),
                ))
                .add_plugins(SimulationPlugin);
            {
                let mut s = app.world_mut().resource_mut::<Settlement>();
                s.helper_requested = invite;
                s.farm.grain.fill(GrainLocation::Home(1));
                s.logs.fill(LogLocation::Home(1));
            }
            app
        }
        let mut single = app(false);
        let mut crew = app(true);
        for _ in 0..2001 {
            single.update();
            crew.update();
        }
        assert_eq!(
            single.world().resource::<SimulationClock>().0,
            crew.world().resource::<SimulationClock>().0
        );
        let a = single.world().resource::<Settlement>();
        let b = crew.world().resource::<Settlement>();
        assert_eq!(a.calendar.year, b.calendar.year);
        assert_eq!(a.calendar.remaining(), b.calendar.remaining());
        assert_eq!(a.household.food_used, b.household.food_used);
        assert_eq!(a.household.missed_rations, b.household.missed_rations);
        assert_eq!(a.household.hearth.burned, b.household.hearth.burned);
        assert_eq!(b.household.hearth.burned, 3);
        assert_eq!(a.logs, b.logs);
        assert_eq!(a.farm.grain, b.farm.grain);
        assert_eq!(
            crew.world_mut()
                .query::<&Worker>()
                .iter(crew.world())
                .count(),
            2
        );
        assert!(
            !crew
                .world_mut()
                .resource_mut::<Settlement>()
                .request_helper()
        );
        for _ in 0..10 {
            crew.update();
        }
        assert_eq!(
            crew.world_mut()
                .query::<&Worker>()
                .iter(crew.world())
                .count(),
            2
        );
    }
    #[test]
    fn coastal_handoff_finishes_existing_catch_ingredients_processing_and_meal_transport() {
        for handoff in [
            CoastalTask::Fishing,
            CoastalTask::ToTableWithRaw,
            CoastalTask::Salting,
            CoastalTask::ToFishHome,
        ] {
            let mut s = Settlement::default();
            let mut a = Worker::default();
            s.fishery.request_fishing();
            s.fishery.request_salting();
            s.household.request_supply();
            s.barter.salt.fill(SaltLocation::Stored);
            for _ in 0..5000 {
                a.advance(0.1, &mut s);
                if a.task == Task::Coastal(handoff) {
                    break;
                }
            }
            assert_eq!(a.task, Task::Coastal(handoff));
            assert_eq!(s.coastal_owner, Some(1));
            s.coastal_assigned = true;
            let mut b = Worker {
                id: 2,
                role: WorkerRole::Coastal,
                ..Worker::default()
            };
            let mut helper_worked = false;
            for _ in 0..6000 {
                let primary_still_owns = s.coastal_owner == Some(1);
                two_tick(&mut s, &mut a, &mut b, 0.1);
                if primary_still_owns && s.coastal_owner == Some(1) {
                    assert!(!matches!(b.task, Task::Coastal(_)));
                }
                helper_worked |= matches!(b.task, Task::Coastal(_));
                if s.coastal_owner == Some(2) {
                    assert!(!matches!(a.task, Task::Coastal(_)));
                }
            }
            assert!(helper_worked);
            assert_eq!(s.household.fish_used, 4);
            assert_eq!(s.fishery.batches, 2);
            assert_eq!(s.barter.count(SaltLocation::Consumed), 2);
            assert_eq!(a.task, Task::Idle);
            assert_eq!(b.task, Task::Idle);
            assert_eq!(b.position, COASTAL_REST);
            assert_eq!(s.coastal_owner, None);
        }
    }
    #[test]
    fn helper_is_once_only_and_takes_only_coastal_jobs() {
        let mut s = Settlement::default();
        assert!(s.request_helper());
        assert!(!s.request_helper());
        s.coastal_assigned = true;
        s.request_harvest(0);
        s.farm.request_sow();
        s.flock.request_shear();
        s.household.hearth.request();
        let mut a = Worker::default();
        let mut b = Worker {
            id: 2,
            role: WorkerRole::Coastal,
            ..Worker::default()
        };
        for _ in 0..1000 {
            two_tick(&mut s, &mut a, &mut b, 0.1);
        }
        assert_eq!(b.task, Task::Idle);
        assert_eq!(b.position, COASTAL_REST);
        assert!(!b.carrying.is_some() && !b.carrying_grain.is_some() && !b.carrying_wool.is_some());
        assert!(s.trees[0].felled);
        assert!(s.farm.phase != CropPhase::Fallow);
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
