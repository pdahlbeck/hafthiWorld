//! Player intent only: orders change shared state; FixedUpdate performs the work.
use crate::{
    camera::{CameraInput, CameraMode, CameraRig},
    simulation::{Settlement, TREE_POSITIONS},
};
use bevy::{prelude::*, ui_widgets::Activate, window::PrimaryWindow};

pub const ORDER_PANEL_HEIGHT: f32 = 100.0;
pub const ORDER_PANEL_BOTTOM: f32 = 12.0;

pub struct OrdersPlugin;
impl Plugin for OrdersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrderState>()
            .init_resource::<crate::inspection::Inspection>()
            .init_resource::<PendingOrders>()
            .add_observer(queue_button)
            .add_systems(Update, issue_orders.after(CameraInput).in_set(OrderInput));
    }
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderInput;
#[derive(Resource)]
pub struct OrderState {
    pub selected: usize,
    pub feedback: &'static str,
}
impl Default for OrderState {
    fn default() -> Self {
        Self {
            selected: 0,
            feedback: "Select a tree, then order harvest. Order the rack when ready.",
        }
    }
}
#[derive(Component, Clone, Copy)]
pub enum OrderButton {
    Select(usize),
    Harvest,
    Build,
    Sow,
    Reap,
    Shear,
    SupplyHome,
    ReserveSeed,
    Barter,
    SupplyFuel,
    Fish,
    SaltFish,
    InviteHelper,
    CollectWool,
}
fn apply(order: OrderButton, ui: &mut OrderState, settlement: &mut Settlement) {
    match order {
        OrderButton::Select(i) if i < TREE_POSITIONS.len() => {
            ui.selected = i;
            ui.feedback = "Tree selected. F / Harvest orders felling.";
        }
        OrderButton::Select(_) => {}
        OrderButton::Harvest => {
            ui.feedback = if settlement.request_harvest(ui.selected) {
                {
                    info!("Harvest ordered for tree {}", ui.selected + 1);
                    "Harvest ordered. Worker will fell the tree and store its logs."
                }
            } else {
                "This tree is already ordered or felled."
            };
        }
        OrderButton::CollectWool => {
            ui.feedback = match settlement.request_shipment() {
                Ok(false) => {
                    "Boat requested: waits for both completed salt trades, then collects the visitor's four paid wool bundles."
                }
                Ok(true) => {
                    "New coastal round: shear regrown fleece, catch fish, barter for salt and ship wool. Home supply keeps its existing order."
                }
                Err(reason) => reason,
            };
        }
        OrderButton::InviteHelper => {
            ui.feedback = if settlement.request_helper() {
                "Coastal helper invited. The resident finishes the current coastal job before handing it over."
            } else {
                "The coastal helper has already been invited; there are two residents at most."
            };
        }
        OrderButton::Fish => {
            ui.feedback = if settlement.fishery.request_fishing() {
                "Fishing ordered: the worker catches and stores up to four real fish at the shore."
            } else {
                "Fishing already ordered or this coastal round's four-fish catch is exhausted."
            };
        }
        OrderButton::SaltFish => {
            ui.feedback = if settlement.fishery.request_salting() {
                "Salting ordered: waits for two stored raw fish and one stored salt, then carries ingredients to the table."
            } else {
                "Salting already ordered; it waits for real raw fish and stored salt."
            };
        }
        OrderButton::SupplyFuel => {
            ui.feedback = if settlement.household.hearth.request() {
                "Firewood ordered: carry stored logs home, up to three; winter burns owned fuel only."
            } else {
                "Firewood supply already ordered. It waits for stored timber or room in the home stack."
            };
        }
        OrderButton::Barter => {
            ui.feedback = if settlement.barter.request() {
                "Barter ordered: carry two stored wool per salt pouch, then bring the paid salt home."
            } else {
                "Barter already ordered or salt offer exhausted. The visitor has only two pouches."
            };
        }
        OrderButton::ReserveSeed => {
            ui.feedback = if settlement.farm.request_reserve() {
                "Seed saving ordered: one stored sack per harvest becomes the next seed batch, before food supply."
            } else {
                "Seed saving is already ordered. It waits for a stored sack from each harvest."
            };
        }
        OrderButton::SupplyHome => {
            ui.feedback = if settlement.household.request_supply() {
                "Home supply ordered. Worker carries stored grain or salted fish to the house."
            } else {
                "Home supply already ordered; it waits for stored grain or salted fish."
            };
        }
        OrderButton::Shear => {
            ui.feedback = if settlement.flock.request_shear() {
                "Shearing ordered. Sheep hold still; the worker clips and stores each wool bundle."
            } else {
                "All sheep are already ordered or shorn. Start a later coastal round with U after regrowth and delivery."
            };
        }
        OrderButton::Sow => {
            ui.feedback = if settlement.farm.request_sow() {
                if settlement.calendar.can_sow() {
                    "Sowing ordered. Worker collects four seed units and sows the field."
                } else {
                    "Sowing queued for spring. Seeds stay stored until the worker starts in spring or summer."
                }
            } else {
                "Need stored seeds and an unplanted or harvested field. V saves a stored sack for the next sowing."
            };
        }
        OrderButton::Reap => {
            ui.feedback = if settlement.farm.request_reap() {
                "Reaping ordered. Worker cuts the ripe crop, then carries each sack to storage."
            } else {
                "Need a ripe crop and all previous grain used or saved. Wait, then press R again."
            };
        }
        OrderButton::Build => {
            ui.feedback = if settlement.request_build() {
                {
                    info!("Timber rack ordered");
                    "Rack ordered. Worker waits for two stored logs, then carries them to site."
                }
            } else {
                "This rack is already ordered or complete."
            };
        }
    }
}
// A deterministic ray/sphere pick of the visible crown (or stump after felling).
fn pick_tree(origin: Vec3, direction: Vec3, settlement: &Settlement) -> Option<usize> {
    pick_tree_hit(origin, direction, settlement).map(|(i, _)| i)
}
pub(crate) fn pick_tree_hit(
    origin: Vec3,
    direction: Vec3,
    settlement: &Settlement,
) -> Option<(usize, f32)> {
    TREE_POSITIONS
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let felled = settlement.trees[i].felled;
            let center = Vec3::new(p.x, if felled { 0.4 } else { 2.25 }, p.y);
            let radius = if felled { 0.3 } else { 0.85 };
            let offset = origin - center;
            let b = offset.dot(direction);
            let disc = b * b - offset.length_squared() + radius * radius;
            if disc < 0.0 {
                return None;
            }
            let near = -b - disc.sqrt();
            let far = -b + disc.sqrt();
            let distance = if near >= 0.0 { near } else { far };
            (distance >= 0.0).then_some((i, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}
#[derive(bevy::ecs::system::SystemParam)]
struct OrderControls<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    rig: Res<'w, CameraRig>,
    inspection: Res<'w, crate::inspection::Inspection>,
}
#[derive(Resource, Default)]
struct PendingOrders(Vec<OrderButton>);
fn queue_button(
    event: On<Activate>,
    buttons: Query<&OrderButton>,
    mut pending: ResMut<PendingOrders>,
) {
    if let Ok(order) = buttons.get(event.entity) {
        pending.0.push(*order);
    }
}
fn issue_orders(
    input: OrderControls,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut pending: ResMut<PendingOrders>,
    mut ui: ResMut<OrderState>,
    mut settlement: ResMut<Settlement>,
) {
    let actions = std::mem::take(&mut pending.0);
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused || (input.rig.mode == CameraMode::FirstPerson && !input.rig.captured) {
        return;
    }
    if input.rig.mode == CameraMode::Strategy {
        if input.mouse.just_pressed(MouseButton::Left)
            && !input.inspection.claimed_click
            && let Some(cursor) = window.cursor_position()
            && cursor.y < window.height() - ORDER_PANEL_HEIGHT - ORDER_PANEL_BOTTOM - 8.0
            && let Ok((camera, transform)) = camera.single()
            && let Ok(ray) = camera.viewport_to_world(transform, cursor)
            && let Some(i) = pick_tree(ray.origin, *ray.direction, &settlement)
        {
            apply(OrderButton::Select(i), &mut ui, &mut settlement);
        }
        for order in actions {
            apply(order, &mut ui, &mut settlement);
        }
    }
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]
        .into_iter()
        .enumerate()
    {
        if input.keys.just_pressed(key) {
            apply(OrderButton::Select(i), &mut ui, &mut settlement);
        }
    }
    if input.keys.just_pressed(KeyCode::KeyF) {
        apply(OrderButton::Harvest, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyB) {
        apply(OrderButton::Build, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyU) {
        apply(OrderButton::CollectWool, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyN) {
        apply(OrderButton::InviteHelper, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyP) {
        apply(OrderButton::Fish, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyL) {
        apply(OrderButton::SaltFish, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyK) {
        apply(OrderButton::SupplyFuel, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyJ) {
        apply(OrderButton::Barter, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyV) {
        apply(OrderButton::ReserveSeed, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyH) {
        apply(OrderButton::SupplyHome, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyC) {
        apply(OrderButton::Shear, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyO) {
        apply(OrderButton::Sow, &mut ui, &mut settlement);
    }
    if input.keys.just_pressed(KeyCode::KeyR) {
        apply(OrderButton::Reap, &mut ui, &mut settlement);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_production_and_home_orders_obey_mode_and_focus_guards() {
        use crate::farming::{CropPhase, Farm};
        let mut app = App::new();
        app.init_resource::<Settlement>()
            .init_resource::<CameraRig>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_plugins(OrdersPlugin);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let collect = app.world_mut().spawn(OrderButton::CollectWool).id();
        app.world_mut().trigger(Activate { entity: collect });
        app.update();
        assert_eq!(
            app.world().resource::<Settlement>().voyage.phase,
            crate::shipping::VoyagePhase::WaitingForTrade
        );
        let invite = app.world_mut().spawn(OrderButton::InviteHelper).id();
        app.world_mut().trigger(Activate { entity: invite });
        app.update();
        assert!(app.world().resource::<Settlement>().helper_requested);
        for order in [OrderButton::Fish, OrderButton::SaltFish] {
            let button = app.world_mut().spawn(order).id();
            app.world_mut().trigger(Activate { entity: button });
            app.update();
        }
        assert!(app.world().resource::<Settlement>().fishery.fishing_ordered);
        assert!(app.world().resource::<Settlement>().fishery.salting_ordered);
        let fuel = app.world_mut().spawn(OrderButton::SupplyFuel).id();
        app.world_mut().trigger(Activate { entity: fuel });
        app.update();
        assert!(
            app.world()
                .resource::<Settlement>()
                .household
                .hearth
                .ordered
        );
        let barter = app.world_mut().spawn(OrderButton::Barter).id();
        app.world_mut().trigger(Activate { entity: barter });
        app.update();
        assert!(app.world().resource::<Settlement>().barter.ordered);
        let reserve = app.world_mut().spawn(OrderButton::ReserveSeed).id();
        app.world_mut().trigger(Activate { entity: reserve });
        app.update();
        assert!(app.world().resource::<Settlement>().farm.reserve_ordered);
        let supply = app.world_mut().spawn(OrderButton::SupplyHome).id();
        app.world_mut().trigger(Activate { entity: supply });
        app.update();
        assert!(
            app.world()
                .resource::<Settlement>()
                .household
                .supply_ordered
        );
        let shear = app.world_mut().spawn(OrderButton::Shear).id();
        app.world_mut().trigger(Activate { entity: shear });
        app.update();
        assert!(
            app.world()
                .resource::<Settlement>()
                .flock
                .sheep
                .iter()
                .all(|s| s.ordered)
        );
        let sow = app.world_mut().spawn(OrderButton::Sow).id();
        let reap = app.world_mut().spawn(OrderButton::Reap).id();
        app.world_mut().trigger(Activate { entity: sow });
        app.update();
        assert_eq!(
            app.world().resource::<Settlement>().farm.phase,
            CropPhase::SowOrdered
        );
        app.world_mut().resource_mut::<Settlement>().farm.phase = CropPhase::Ripe;
        app.world_mut().trigger(Activate { entity: reap });
        app.update();
        assert!(app.world().resource::<Settlement>().farm.harvest_ordered);
        for (mode, focused, captured, allowed) in [
            (CameraMode::Strategy, true, false, true),
            (CameraMode::FirstPerson, true, true, true),
            (CameraMode::FirstPerson, true, false, false),
            (CameraMode::Strategy, false, false, false),
            (CameraMode::FirstPerson, false, true, false),
        ] {
            {
                let mut rig = app.world_mut().resource_mut::<CameraRig>();
                rig.mode = mode;
                rig.captured = captured;
            }
            app.world_mut().get_mut::<Window>(window).unwrap().focused = focused;
            app.world_mut().resource_mut::<Settlement>().farm = Farm::default();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyO);
            }
            app.update();
            assert_eq!(
                app.world().resource::<Settlement>().farm.phase == CropPhase::SowOrdered,
                allowed
            );
            app.world_mut().resource_mut::<Settlement>().farm.phase = CropPhase::Ripe;
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyR);
            }
            app.update();
            assert_eq!(
                app.world().resource::<Settlement>().farm.harvest_ordered,
                allowed
            );
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyV);
            }
            app.update();
            assert_eq!(
                app.world().resource::<Settlement>().farm.reserve_ordered,
                allowed
            );
            app.world_mut().resource_mut::<Settlement>().barter = crate::trade::Barter::default();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyJ);
            }
            app.update();
            assert_eq!(app.world().resource::<Settlement>().barter.ordered, allowed);
            app.world_mut().resource_mut::<Settlement>().household =
                crate::household::Household::default();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyH);
            }
            app.update();
            assert_eq!(
                app.world()
                    .resource::<Settlement>()
                    .household
                    .supply_ordered,
                allowed
            );
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyK);
            }
            app.update();
            assert_eq!(
                app.world()
                    .resource::<Settlement>()
                    .household
                    .hearth
                    .ordered,
                allowed
            );
            app.world_mut()
                .resource_mut::<Settlement>()
                .helper_requested = false;
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyN);
            }
            app.update();
            assert_eq!(
                app.world().resource::<Settlement>().helper_requested,
                allowed
            );
            app.world_mut().resource_mut::<Settlement>().voyage =
                crate::shipping::Voyage::default();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyU);
            }
            app.update();
            assert_eq!(
                app.world().resource::<Settlement>().voyage.phase
                    == crate::shipping::VoyagePhase::WaitingForTrade,
                allowed
            );
            app.world_mut().resource_mut::<Settlement>().fishery =
                crate::fishing::Fishery::default();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyP);
                keys.press(KeyCode::KeyL);
            }
            app.update();
            assert_eq!(
                app.world().resource::<Settlement>().fishery.fishing_ordered,
                allowed
            );
            assert_eq!(
                app.world().resource::<Settlement>().fishery.salting_ordered,
                allowed
            );
            app.world_mut().resource_mut::<Settlement>().flock = crate::livestock::Flock::default();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.reset_all();
                keys.press(KeyCode::KeyC);
            }
            app.update();
            assert_eq!(
                app.world()
                    .resource::<Settlement>()
                    .flock
                    .sheep
                    .iter()
                    .all(|s| s.ordered),
                allowed
            );
        }
    }
    #[test]
    fn ray_selects_visible_tree_and_misses_empty_space() {
        let s = Settlement::default();
        let p = TREE_POSITIONS[1];
        assert_eq!(
            pick_tree(Vec3::new(p.x, 2.25, p.y + 4.0), Vec3::NEG_Z, &s),
            Some(1)
        );
        assert_eq!(pick_tree(Vec3::new(20.0, 2.0, 20.0), Vec3::NEG_Z, &s), None);
        let mut felled = s;
        felled.trees[1].felled = true;
        assert_eq!(
            pick_tree(Vec3::new(p.x, 2.25, p.y + 4.0), Vec3::NEG_Z, &felled),
            None
        );
    }
    #[test]
    fn button_activation_is_consumed_once_and_cannot_leak_through_focus_loss() {
        let mut app = App::new();
        app.init_resource::<Settlement>()
            .init_resource::<CameraRig>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_plugins(OrdersPlugin);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let harvest = app.world_mut().spawn(OrderButton::Harvest).id();
        app.world_mut().trigger(Activate { entity: harvest });
        app.update();
        assert!(app.world().resource::<Settlement>().trees[0].ordered);
        app.world_mut().resource_mut::<OrderState>().selected = 1;
        app.update();
        assert!(!app.world().resource::<Settlement>().trees[1].ordered);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(Activate { entity: harvest });
        app.update();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
        assert!(!app.world().resource::<Settlement>().trees[1].ordered);
    }
    #[test]
    fn keyboard_orders_work_in_both_modes_but_not_after_mouse_release_or_focus_loss() {
        let mut app = App::new();
        app.init_resource::<Settlement>()
            .init_resource::<CameraRig>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_plugins(OrdersPlugin);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::Digit2);
            keys.press(KeyCode::KeyF);
            keys.press(KeyCode::KeyB);
        }
        app.update();
        assert!(app.world().resource::<Settlement>().trees[1].ordered);
        assert!(app.world().resource::<Settlement>().construction.ordered);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().resource_mut::<CameraRig>().toggle();
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::Digit3);
            keys.press(KeyCode::KeyF);
        }
        app.update();
        assert!(app.world().resource::<Settlement>().trees[2].ordered);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().resource_mut::<CameraRig>().captured = false;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::Digit1);
            keys.press(KeyCode::KeyF);
        }
        app.update();
        assert!(!app.world().resource::<Settlement>().trees[0].ordered);
        app.world_mut().resource_mut::<CameraRig>().captured = true;
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.update();
        assert!(!app.world().resource::<Settlement>().trees[0].ordered);
    }
}
