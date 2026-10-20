//! Player intent only: orders change shared state; FixedUpdate performs the work.
use crate::{
    camera::{CameraInput, CameraMode, CameraRig},
    simulation::{Settlement, TREE_POSITIONS},
};
use bevy::{prelude::*, ui_widgets::Activate, window::PrimaryWindow};

pub struct OrdersPlugin;
impl Plugin for OrdersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrderState>()
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
        .map(|(i, _)| i)
}
#[derive(bevy::ecs::system::SystemParam)]
struct OrderControls<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    rig: Res<'w, CameraRig>,
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
            && let Some(cursor) = window.cursor_position()
            && cursor.y < window.height() - 76.0
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
}
#[cfg(test)]
mod tests {
    use super::*;
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
