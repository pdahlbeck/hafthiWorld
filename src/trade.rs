//! One finite shore offer: physically deliver wool, then carry paid salt home.
use crate::{
    livestock::WoolLocation,
    simulation::{STOCKPILE, Settlement, Worker},
};
use bevy::prelude::*;

pub const TRADER_POSITION: Vec2 = Vec2::new(-6.3, 4.4);
pub const TRADE_WORK: Vec2 = Vec2::new(-5.65, 4.4);
pub const WOOL_PRICE: usize = 2;
pub const SALT_UNITS: usize = 2;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaltLocation {
    Offered,
    Paid,
    Carried,
    Stored,
    AtSalting,
    Consumed,
}
#[derive(Debug)]
pub struct Barter {
    pub ordered: bool,
    pub salt: [SaltLocation; SALT_UNITS],
}
impl Default for Barter {
    fn default() -> Self {
        Self {
            ordered: false,
            salt: [SaltLocation::Offered; SALT_UNITS],
        }
    }
}
impl Barter {
    pub fn request(&mut self) -> bool {
        if self.ordered || !self.salt.contains(&SaltLocation::Offered) {
            return false;
        }
        self.ordered = true;
        true
    }
    pub fn count(&self, location: SaltLocation) -> usize {
        self.salt.iter().filter(|&&s| s == location).count()
    }
    pub fn wool_candidate(&self, wool: &[WoolLocation]) -> Option<usize> {
        if !self.ordered
            || self.count(SaltLocation::Offered) == 0
            || self.count(SaltLocation::Paid) + self.count(SaltLocation::Carried) > 0
        {
            return None;
        }
        let delivered = wool.iter().filter(|&&w| w == WoolLocation::Trader).count();
        let needed = WOOL_PRICE - delivered % WOOL_PRICE;
        // Do not start a new payment with less than its full remaining price in storage.
        if wool.iter().filter(|&&w| w == WoolLocation::Stored).count() < needed {
            return None;
        }
        wool.iter().position(|&w| w == WoolLocation::Stored)
    }
    pub fn deliver_wool(&mut self, index: usize, wool: &mut [WoolLocation]) -> bool {
        if !self.ordered
            || wool.get(index) != Some(&WoolLocation::Carried)
            || self.count(SaltLocation::Offered) == 0
            || self.count(SaltLocation::Paid) + self.count(SaltLocation::Carried) > 0
        {
            return false;
        }
        wool[index] = WoolLocation::Trader;
        let delivered = wool.iter().filter(|&&w| w == WoolLocation::Trader).count();
        if delivered.is_multiple_of(WOOL_PRICE) {
            let salt = self
                .salt
                .iter_mut()
                .find(|s| **s == SaltLocation::Offered)
                .expect("offer checked above");
            *salt = SaltLocation::Paid;
        }
        true
    }
    pub fn paid_salt(&self) -> Option<usize> {
        self.salt.iter().position(|&s| s == SaltLocation::Paid)
    }
}
pub fn wool_position(index: usize) -> Vec2 {
    TRADER_POSITION + Vec2::new(-0.45 + index as f32 * 0.28, -0.65)
}
pub fn salt_position(index: usize) -> Vec2 {
    TRADER_POSITION + Vec2::new(-0.25 + index as f32 * 0.5, 0.7)
}

pub struct TradePlugin;
impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, sync);
    }
}
#[derive(Component)]
struct ShoreTrader;
#[derive(Component)]
struct SaltVisual(usize);
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tunic = materials.add(Color::srgb(0.30, 0.34, 0.43));
    let skin = materials.add(Color::srgb(0.68, 0.48, 0.34));
    let leather = materials.add(Color::srgb(0.25, 0.18, 0.12));
    let cloth = materials.add(Color::srgb(0.75, 0.69, 0.51));
    let pale = materials.add(Color::srgb(0.85, 0.83, 0.74));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands
        .spawn((
            ShoreTrader,
            Name::new("Shore visitor — finite wool/salt offer"),
            Transform::from_xyz(TRADER_POSITION.x, 0.2, TRADER_POSITION.y),
            Visibility::Inherited,
        ))
        .with_children(|trader| {
            trader.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(tunic),
                Transform::from_xyz(0.0, 0.8, 0.0).with_scale(Vec3::new(0.48, 0.85, 0.36)),
            ));
            trader.spawn((
                Mesh3d(meshes.add(Sphere::new(0.19))),
                MeshMaterial3d(skin),
                Transform::from_xyz(0.0, 1.42, 0.0),
            ));
            trader.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(leather.clone()),
                Transform::from_xyz(0.0, 0.64, 0.0).with_scale(Vec3::new(0.5, 0.06, 0.38)),
            ));
            for x in [-0.13, 0.13] {
                trader.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(leather.clone()),
                    Transform::from_xyz(x, 0.25, 0.0).with_scale(Vec3::new(0.14, 0.5, 0.18)),
                ));
            }
        });
    // A small cloth spread identifies the visitor's stock without a fictional harbour build.
    commands.spawn((
        Mesh3d(cube),
        MeshMaterial3d(cloth),
        Transform::from_xyz(TRADER_POSITION.x, 0.22, TRADER_POSITION.y + 0.7)
            .with_scale(Vec3::new(1.35, 0.025, 0.7)),
    ));
    let pouch = meshes.add(Sphere::new(1.0).mesh().ico(2).expect("valid salt pouch"));
    for i in 0..SALT_UNITS {
        commands.spawn((
            SaltVisual(i),
            Name::new(format!("Salt pouch {}", i + 1)),
            Mesh3d(pouch.clone()),
            MeshMaterial3d(pale.clone()),
            Transform::default(),
            Visibility::Visible,
        ));
    }
}
fn sync(
    settlement: Res<Settlement>,
    workers: Query<&Worker>,
    mut salt: Query<(&SaltVisual, &mut Transform, &mut Visibility)>,
) {
    let Ok(worker) = workers.single() else {
        return;
    };
    for (visual, mut transform, mut visibility) in &mut salt {
        if settlement.barter.salt[visual.0] == SaltLocation::Consumed {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        let (p, y) = match settlement.barter.salt[visual.0] {
            SaltLocation::Consumed => unreachable!(),
            SaltLocation::AtSalting => (crate::fishing::SALTING_TABLE + Vec2::new(0.45, 0.0), 1.0),
            SaltLocation::Offered | SaltLocation::Paid => (salt_position(visual.0), 0.4),
            SaltLocation::Carried => (worker.position + Vec2::new(0.3, 0.0), 1.0),
            SaltLocation::Stored => (STOCKPILE + Vec2::new(visual.0 as f32 * 0.45, 1.25), 0.4),
        };
        transform.translation = Vec3::new(p.x, y, p.y);
        transform.scale = Vec3::new(0.20, 0.24, 0.20);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        farming::{CropPhase, GRAIN_UNITS, GrainLocation},
        livestock::{LivestockPlugin, WOOL_UNITS},
        simulation::{BUILD_COST, SimulationPlugin, TOTAL_LOGS, Task},
    };
    fn balance(s: &Settlement, w: &Worker) {
        assert_eq!(s.flock.wool.len(), WOOL_UNITS);
        assert_eq!(
            [
                WoolLocation::Fleece,
                WoolLocation::Ground,
                WoolLocation::Carried,
                WoolLocation::Stored,
                WoolLocation::Trader
            ]
            .iter()
            .map(|&l| s.flock.count(l))
            .sum::<usize>(),
            WOOL_UNITS
        );
        assert_eq!(
            s.flock.count(WoolLocation::Carried),
            usize::from(w.carrying_wool.is_some())
        );
        assert_eq!(
            s.barter.count(SaltLocation::Carried),
            usize::from(w.carrying_salt.is_some())
        );
        assert_eq!(
            [
                SaltLocation::Offered,
                SaltLocation::Paid,
                SaltLocation::Carried,
                SaltLocation::Stored
            ]
            .iter()
            .map(|&l| s.barter.count(l))
            .sum::<usize>(),
            SALT_UNITS
        );
        assert_eq!(
            s.flock.count(WoolLocation::Trader) / WOOL_PRICE,
            SALT_UNITS - s.barter.count(SaltLocation::Offered)
        );
        assert!(
            usize::from(w.carrying.is_some())
                + usize::from(w.carrying_grain.is_some())
                + usize::from(w.carrying_wool.is_some())
                + usize::from(w.carrying_salt.is_some())
                <= 1
        );
        assert!(
            crate::world::walkable(w.position),
            "blocked {:?} at {:?}",
            w.task,
            w.position
        );
    }
    #[test]
    fn offer_requires_an_order_and_full_physical_payment_before_releasing_salt() {
        let mut b = Barter::default();
        let mut wool = [WoolLocation::Stored; WOOL_UNITS];
        assert_eq!(b.wool_candidate(&wool), None);
        assert!(!b.deliver_wool(0, &mut wool));
        assert!(b.request());
        assert!(!b.request());
        assert_eq!(b.count(SaltLocation::Offered), 2);
        assert!(!b.deliver_wool(usize::MAX, &mut wool));
        assert!(!b.deliver_wool(0, &mut wool)); // Shared store is not physical handover.
        wool[0] = WoolLocation::Carried;
        assert!(b.deliver_wool(0, &mut wool));
        assert!(!b.deliver_wool(0, &mut wool));
        assert_eq!(b.paid_salt(), None);
        assert_eq!(b.count(SaltLocation::Offered), 2);
        wool[1] = WoolLocation::Carried;
        assert!(b.deliver_wool(1, &mut wool));
        assert_eq!(b.paid_salt(), Some(0));
        assert_eq!(b.count(SaltLocation::Offered), 1);
        assert_eq!(b.wool_candidate(&wool), None); // Retrieve the paid salt first.
        b.salt[0] = SaltLocation::Stored;
        assert_eq!(b.wool_candidate(&wool), Some(2));
    }
    #[test]
    fn standing_order_waits_for_two_real_stored_wool_without_taking_a_partial_price() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.barter.request();
        w.advance(100.0, &mut s);
        assert_eq!(w.task, Task::Idle);
        assert_eq!(s.barter.count(SaltLocation::Offered), 2);
        s.flock.wool[0] = WoolLocation::Stored;
        w.advance(100.0, &mut s);
        assert_eq!(s.flock.count(WoolLocation::Stored), 1);
        assert_eq!(s.flock.count(WoolLocation::Trader), 0);
        s.flock.wool[1] = WoolLocation::Stored;
        w.advance(0.1, &mut s);
        assert_eq!(w.task, Task::LoadingTradeWool);
        assert_eq!(s.flock.count(WoolLocation::Stored), 2); // Still there until pickup completes.
        w.advance(1.0, &mut s);
        assert_eq!(w.task, Task::ToTrader);
        assert_eq!(s.flock.count(WoolLocation::Carried), 1);
        assert_eq!(s.barter.count(SaltLocation::Offered), 2);
    }
    #[test]
    fn shearing_and_trade_deliver_two_finite_salt_pouches_over_clear_routes() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.barter.request();
        s.flock.request_shear();
        let mut carried_wool = false;
        let mut carried_salt = false;
        let mut partial_payment = false;
        for _ in 0..4000 {
            w.advance(0.1, &mut s);
            balance(&s, &w);
            carried_wool |= w.task == Task::ToTrader && w.carrying_wool.is_some();
            carried_salt |= w.task == Task::FromTrader && w.carrying_salt.is_some();
            partial_payment |= s.flock.count(WoolLocation::Trader) == 1
                && s.barter.count(SaltLocation::Offered) == 2;
        }
        assert!(carried_wool && carried_salt && partial_payment);
        assert_eq!(s.flock.count(WoolLocation::Trader), WOOL_UNITS);
        assert_eq!(s.flock.count(WoolLocation::Stored), 0);
        assert_eq!(s.barter.count(SaltLocation::Stored), SALT_UNITS);
        assert_eq!(w.task, Task::Idle);
        for _ in 0..10 {
            assert!(!s.barter.request());
            assert!(!s.flock.request_shear());
            w.advance(100.0, &mut s);
        }
        balance(&s, &w);
        assert_eq!(s.barter.count(SaltLocation::Stored), SALT_UNITS);
    }
    #[test]
    fn no_trade_order_preserves_wool_and_trade_is_stable_across_frame_partitions() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.flock.request_shear();
        w.advance(250.0, &mut s);
        assert_eq!(s.flock.count(WoolLocation::Stored), WOOL_UNITS);
        assert_eq!(s.barter.count(SaltLocation::Offered), SALT_UNITS);
        let mut a = Settlement::default();
        let mut b = Settlement::default();
        a.flock.wool.fill(WoolLocation::Stored);
        b.flock.wool.fill(WoolLocation::Stored);
        a.barter.request();
        b.barter.request();
        let mut wa = Worker::default();
        let mut wb = Worker::default();
        wa.advance(100.0, &mut a);
        for _ in 0..1000 {
            wb.advance(0.1, &mut b);
        }
        assert_eq!(a.flock.wool, b.flock.wool);
        assert_eq!(a.barter.salt, b.barter.salt);
        assert_eq!(wa.task, wb.task);
        assert!(wa.position.distance(wb.position) < 0.001);
        assert_eq!(a.barter.count(SaltLocation::Stored), SALT_UNITS);
    }
    #[test]
    fn barter_coexists_with_timbers_sheep_seed_saving_and_two_food_harvests() {
        let mut s = Settlement::default();
        let mut w = Worker::default();
        s.barter.request();
        s.flock.request_shear();
        s.farm.request_reserve();
        s.household.request_supply();
        s.request_build();
        for i in 0..3 {
            s.request_harvest(i);
        }
        for _ in 0..12000 {
            if s.farm.harvests < 2 {
                if matches!(s.farm.phase, CropPhase::Fallow | CropPhase::Harvested) {
                    s.farm.request_sow();
                }
                s.farm.request_reap();
            }
            w.advance(0.1, &mut s);
            balance(&s, &w);
            let live = s
                .farm
                .grain
                .iter()
                .filter(|g| {
                    matches!(
                        g,
                        GrainLocation::Ground
                            | GrainLocation::Carried
                            | GrainLocation::Stored
                            | GrainLocation::Home(_)
                    )
                })
                .count();
            assert_eq!(
                s.farm.harvests * GRAIN_UNITS as u64,
                live as u64 + s.household.food_used + s.farm.seed_batches_saved
            );
        }
        assert!(s.construction.complete);
        assert_eq!(s.installed(), BUILD_COST);
        assert_eq!(s.stored(), TOTAL_LOGS - BUILD_COST);
        assert_eq!(s.farm.harvests, 2);
        assert_eq!(s.household.food_used, 6);
        assert_eq!(s.farm.seed_batches_saved, 2);
        assert_eq!(s.barter.count(SaltLocation::Stored), SALT_UNITS);
    }
    #[test]
    fn visitor_wool_salt_and_worker_entities_persist_in_both_camera_modes() {
        use bevy::time::TimeUpdateStrategy;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<CameraRig>()
            .add_plugins((SimulationPlugin, LivestockPlugin, TradePlugin));
        app.update();
        let visitor = app
            .world_mut()
            .query_filtered::<Entity, With<ShoreTrader>>()
            .single(app.world())
            .unwrap();
        let worker = app
            .world_mut()
            .query_filtered::<Entity, With<Worker>>()
            .single(app.world())
            .unwrap();
        let salt: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<SaltVisual>>()
            .iter(app.world())
            .collect();
        assert_eq!(salt.len(), SALT_UNITS);
        let entity_count = app.world().entities().len();
        {
            let mut s = app.world_mut().resource_mut::<Settlement>();
            s.barter.request();
            s.flock.request_shear();
        }
        for frame in 0..4000 {
            if frame % 11 == 0 {
                app.world_mut().resource_mut::<CameraRig>().toggle();
            }
            app.update();
            balance(
                app.world().resource::<Settlement>(),
                app.world().get::<Worker>(worker).unwrap(),
            );
        }
        assert!(app.world().get::<ShoreTrader>(visitor).is_some());
        assert_eq!(app.world().entities().len(), entity_count);
        assert_eq!(
            app.world()
                .resource::<Settlement>()
                .barter
                .count(SaltLocation::Stored),
            SALT_UNITS
        );
        for id in salt {
            let i = app.world().get::<SaltVisual>(id).unwrap().0;
            let p = STOCKPILE + Vec2::new(i as f32 * 0.45, 1.25);
            assert_eq!(
                app.world().get::<Transform>(id).unwrap().translation,
                Vec3::new(p.x, 0.4, p.y)
            );
        }
    }
}
