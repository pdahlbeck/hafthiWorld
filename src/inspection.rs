//! Read-only inspection of the same settlement in both camera modes.
use crate::{
    camera::{CameraInput, CameraMode, CameraRig},
    farming::GrainLocation,
    fishing::{CoastalTask as C, FishLocation},
    livestock::WoolLocation,
    orders::{ORDER_PANEL_BOTTOM, ORDER_PANEL_HEIGHT, OrderInput},
    simulation::{LogLocation, Settlement, Task, Worker},
    trade::SaltLocation,
};
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress, Button},
    window::PrimaryWindow,
};

const PANEL_WIDTH: f32 = 330.0;
const PANEL_TOP: f32 = 20.0;
const PANEL_HEIGHT: f32 = 310.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    NewHome,
    Home,
    Resident(u32),
    Boat,
}
#[derive(Resource)]
pub struct Inspection {
    pub subject: Option<Subject>,
    pub details: bool,
    pub claimed_click: bool,
}
impl Default for Inspection {
    fn default() -> Self {
        Self {
            subject: Some(Subject::Home),
            details: false,
            claimed_click: false,
        }
    }
}
impl Inspection {
    fn cycle(&mut self, helper: bool) {
        self.subject = match self.subject {
            None => Some(Subject::Home),
            Some(Subject::Home) => Some(Subject::Resident(1)),
            Some(Subject::Resident(1)) if helper => Some(Subject::Resident(2)),
            Some(Subject::Resident(_)) => Some(Subject::Boat),
            Some(Subject::Boat | Subject::NewHome) => None,
        };
    }
    fn over_panel(&self, cursor: Vec2, window: &Window) -> bool {
        self.subject.is_some()
            && cursor.x >= (window.width() - PANEL_WIDTH - 12.0).max(0.0)
            && cursor.y >= PANEL_TOP
            && cursor.y <= PANEL_TOP + PANEL_HEIGHT
    }
}
pub struct InspectionPlugin;
impl Plugin for InspectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inspection>()
            .init_resource::<PendingInspection>()
            .add_observer(queue_button)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    input.after(CameraInput).before(OrderInput),
                    refresh.after(OrderInput),
                    sync_buttons.after(OrderInput),
                ),
            );
    }
}
#[derive(Resource, Default)]
struct PendingInspection(Vec<Option<Subject>>);
#[derive(Component, Clone, Copy)]
struct InspectButton(Option<Subject>);
#[derive(Component)]
struct InspectorPanel;
#[derive(Component)]
struct InspectorText;
#[derive(Component)]
struct InspectorNavigation;
fn queue_button(
    event: On<Activate>,
    buttons: Query<&InspectButton>,
    mut pending: ResMut<PendingInspection>,
) {
    if let Ok(button) = buttons.get(event.entity) {
        pending.0.push(button.0);
    }
}
#[derive(bevy::ecs::system::SystemParam)]
struct Controls<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    rig: Res<'w, CameraRig>,
}
fn input(
    input: Controls,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    workers: Query<&Worker>,
    settlement: Res<Settlement>,
    mut state: ResMut<Inspection>,
    mut pending: ResMut<PendingInspection>,
) {
    state.claimed_click = false;
    let actions = std::mem::take(&mut pending.0);
    if let Some(Subject::Resident(id)) = state.subject
        && !workers.iter().any(|w| w.id == id)
    {
        state.subject = Some(Subject::Home);
    }
    let Ok(window) = windows.single() else { return };
    if !window.focused {
        return;
    }
    if input.keys.just_pressed(KeyCode::F2) {
        state.details = !state.details;
    }
    if input.keys.just_pressed(KeyCode::KeyI) {
        state.cycle(workers.iter().any(|w| w.id == 2));
    }
    if input.rig.mode != CameraMode::Strategy {
        return;
    }
    let has_actions = !actions.is_empty();
    for subject in actions {
        if let Some(Subject::Resident(id)) = subject
            && !workers.iter().any(|w| w.id == id)
        {
            continue;
        }
        state.subject = subject;
        state.claimed_click = true;
    }
    if has_actions {
        return;
    }
    if !input.mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if state.over_panel(cursor, window) {
        state.claimed_click = true;
        return;
    }
    if cursor.y >= window.height() - ORDER_PANEL_HEIGHT - ORDER_PANEL_BOTTOM - 8.0 {
        return;
    }
    let Ok((camera, transform)) = camera.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };
    if let Some(subject) = pick_subject(ray.origin, *ray.direction, &settlement, workers.iter()) {
        state.subject = Some(subject);
        state.claimed_click = true;
    }
}
// Conservative axis-aligned footprints; nearest foreground tree wins over inspection.
fn ray_box(origin: Vec3, direction: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut near = 0.0f32;
    let mut far = f32::INFINITY;
    for axis in 0..3 {
        let offset = center[axis] - origin[axis];
        if direction[axis].abs() < 1e-6 {
            if offset.abs() > half[axis] {
                return None;
            }
            continue;
        }
        let a = (offset - half[axis]) / direction[axis];
        let b = (offset + half[axis]) / direction[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return None;
        }
    }
    Some(near)
}
fn pick_subject<'a>(
    origin: Vec3,
    direction: Vec3,
    s: &Settlement,
    workers: impl Iterator<Item = &'a Worker>,
) -> Option<Subject> {
    let mut hits = Vec::new();
    for worker in workers {
        if let Some(distance) = ray_box(
            origin,
            direction,
            Vec3::new(worker.position.x, 1.0, worker.position.y),
            Vec3::new(0.4, 0.85, 0.4),
        ) {
            hits.push((Subject::Resident(worker.id), distance));
        }
    }
    if let Some(distance) = ray_box(
        origin,
        direction,
        Vec3::new(2., 1.6, 0.),
        Vec3::new(1.85, 1.45, 2.55),
    ) {
        hits.push((Subject::Home, distance));
    }
    if let Some(distance) = ray_box(
        origin,
        direction,
        Vec3::new(crate::homestead::CENTER.x, 1.2, crate::homestead::CENTER.y),
        Vec3::new(1.5, 1.2, 2.0),
    ) {
        hits.push((Subject::NewHome, distance));
    }
    if s.voyage.visible() {
        let p = s.voyage.position();
        if let Some(distance) = ray_box(
            origin,
            direction,
            Vec3::new(p.x, 0.8, p.y),
            Vec3::new(2.1, 1.0, 0.8),
        ) {
            hits.push((Subject::Boat, distance));
        }
    }
    let hit = hits.into_iter().min_by(|a, b| a.1.total_cmp(&b.1))?;
    if crate::orders::pick_tree_hit(origin, direction, s)
        .is_some_and(|(_, distance)| distance <= hit.1)
    {
        return None;
    }
    Some(hit.0)
}
fn cargo(worker: &Worker, s: &Settlement) -> String {
    for (slot, label) in [
        (worker.carrying, "timber"),
        (worker.carrying_grain, "grain sack"),
        (worker.carrying_wool, "wool bundle"),
        (worker.carrying_salt, "salt pouch"),
    ] {
        if let Some(slot) = slot {
            return format!("{label} #{}", slot + 1);
        }
    }
    if let Some(slot) = worker.carrying_fish {
        let kind = if s.fishery.fish[slot] == FishLocation::CarriedRaw {
            "raw"
        } else {
            "salted"
        };
        return format!("{kind} fish #{}", slot + 1);
    }
    if s.farm.seeds == crate::farming::SeedLocation::Carried && worker.id == 1 {
        return "seed batch".into();
    }
    "empty hands".into()
}
fn job(task: Task) -> &'static str {
    match task {
        Task::Idle => "Resting / waiting for work",
        Task::ToCrewRest => "Returning to coastal rest spot",
        Task::Coastal(task) => match task {
            C::ToExportPile => "Walking to visitor's wool",
            C::LoadingExportWool => "Picking up paid wool",
            C::ToBoat => "Carrying wool to the landing",
            C::UnloadingExportWool => "Loading wool aboard",
            C::ReturnFromBoat => "Returning from the landing",
            C::ToFishingSpot => "Walking to fishing spot",
            C::Fishing => "Catching fish",
            C::LoadingFresh => "Picking up fresh catch",
            C::ReturningFresh => "Carrying raw fish to storage",
            C::StoringFresh => "Storing raw fish",
            C::ToBrineStore => "Collecting salt for salting",
            C::LoadingBrineSalt => "Picking up salting salt",
            C::ToTableWithSalt => "Carrying salt to the table",
            C::DepositSalt => "Putting salt on the table",
            C::BackForRaw => "Collecting raw fish",
            C::LoadingRaw => "Picking up raw fish",
            C::ToTableWithRaw => "Carrying raw fish to the table",
            C::DepositRaw => "Putting fish on the table",
            C::Salting => "Salting a two-fish batch",
            C::LoadingPreserved => "Picking up salted fish",
            C::BackWithPreserved => "Carrying salted fish to storage",
            C::StoringPreserved => "Storing salted fish",
            C::ToPickPreserved => "Returning for salted fish",
            C::ToMealStore => "Collecting salted fish for home",
            C::LoadingMeal => "Picking up a fish ration",
            C::ToFishHome => "Carrying salted fish home",
            C::UnloadingMeal => "Delivering fish to home",
        },
        Task::ToTree => "Walking to a tree",
        Task::Felling => "Felling a tree",
        Task::ToLog => "Collecting timber",
        Task::Loading => "Picking up timber",
        Task::ToStockpile => "Carrying timber to storage",
        Task::Unloading => "Storing timber",
        Task::ToSite => "Walking to the timber rack",
        Task::Installing => "Installing rack timber",
        Task::Assembling => "Finishing the timber rack",
        Task::ToFarmGateway | Task::ToField => "Walking to the field",
        Task::LoadingSeeds => "Collecting seed batch",
        Task::ToSeedStore => "Collecting grain for seed",
        Task::PreparingSeeds => "Preparing next seed batch",
        Task::Sowing => "Sowing the field",
        Task::Reaping => "Reaping ripe crops",
        Task::LoadingGrain => "Picking up harvested grain",
        Task::DeliveringGrain => "Carrying grain to storage",
        Task::StoringGrain => "Storing grain",
        Task::FromField => "Returning from the field",
        Task::ToPastureGateway | Task::ToPastureEdge | Task::ToPasture | Task::ToSheep => {
            "Walking into the pasture"
        }
        Task::Shearing => "Shearing a sheep",
        Task::ToWool => "Collecting clipped wool",
        Task::LoadingWool => "Picking up clipped wool",
        Task::FromPasture | Task::FromPastureEdge => "Leaving the pasture",
        Task::DeliveringWool => "Carrying wool to storage",
        Task::StoringWool => "Storing wool",
        Task::ToFoodStore => "Collecting grain for home",
        Task::LoadingFood => "Picking up grain ration",
        Task::ToHome => "Carrying grain home",
        Task::UnloadingFood => "Delivering grain to home",
        Task::FromHome => "Returning from home",
        Task::ToFuelStore => "Collecting stored firewood",
        Task::LoadingFuel => "Picking up firewood",
        Task::ToFuelHome => "Carrying firewood home",
        Task::UnloadingFuel => "Delivering home firewood",
        Task::ToTradeStore => "Collecting wool for barter",
        Task::LoadingTradeWool => "Picking up barter wool",
        Task::ToTrader => "Carrying wool to visitor",
        Task::NewHome(_, _) => "Working for the second farmstead",
        Task::HandingTradeWool => "Paying wool for salt",
        Task::LoadingSalt => "Picking up paid salt",
        Task::FromTrader => "Returning from visitor",
        Task::StoringSalt => "Storing imported salt",
    }
}
fn description(subject: Subject, s: &Settlement, workers: &[&Worker]) -> String {
    match subject {
        Subject::NewHome => {
            let h = &s.homestead;
            let pantry = crate::homestead::food(s, 2);
            format!(
                "SECOND FARMSTEAD\nTimber delivered: {} / {}\nBuilt: {} | resident: {}\n\nPantry: {} portions\nFood used: {} | missed: {}\nFirewood: {} | burned: {}\n\nG: order building / invite resident\nThe new resident carries real food and fuel to this household.",
                s.homestead_timbers(),
                crate::homestead::TIMBER_COST,
                h.complete,
                h.arrived,
                pantry,
                h.household.food_used,
                h.household.missed_rations,
                s.count(LogLocation::Home(2)),
                h.household.hearth.burned
            )
        }
        Subject::Resident(id) => {
            let Some(w) = workers.iter().find(|w| w.id == id) else {
                return "Resident unavailable".into();
            };
            format!(
                "RESIDENT {id}\nRole: {}\nHome: {}\n\n{}\nCarrying: {}\n\nCoastal work held by: {}",
                if w.role == crate::simulation::WorkerRole::Coastal {
                    "coastal worker"
                } else if w.home_id == 2 {
                    "household 2 food and fuel provider"
                } else {
                    "founder / general work"
                },
                w.home_id,
                job(w.task),
                cargo(w, s),
                s.coastal_owner
                    .map_or("nobody".into(), |id| format!("resident {id}"))
            )
        }
        Subject::Home => {
            let id = s.household.id;
            let grain = s.farm.count(GrainLocation::Home(id));
            let fish = s.fishery.count(FishLocation::Home(id));
            format!(
                "HOUSEHOLD {id}\nResidents: {}\n{}\n\nPantry: {grain} grain / {fish} salted fish\nFood used: {} | missed: {}\nNext ration: {}\n\nOwned firewood: {} / 3\nBurned: {} | missed: {}\n{}\n\nH: supply food | K: supply fuel",
                workers.iter().filter(|w| w.home_id == id).count(),
                s.household.status(grain + fish),
                s.household.food_used,
                s.household.missed_rations,
                if s.household.started {
                    format!("{:.0}s", s.household.next_use())
                } else {
                    "after first delivery".into()
                },
                s.count(LogLocation::Home(id)),
                s.household.hearth.burned,
                s.household.hearth.missed,
                s.household
                    .hearth
                    .status(s.calendar.season(), s.count(LogLocation::Home(id)))
            )
        }
        Subject::Boat => {
            let mut text = format!(
                "WOOL COLLECTION BOAT\n{}\nVisiting sailors: {}\n\nCollection manifest",
                s.voyage.label(),
                if s.voyage.visible() {
                    "2 aboard"
                } else {
                    "off scene"
                }
            );
            for (i, location) in s.flock.wool.iter().enumerate() {
                let label = match location {
                    WoolLocation::Fleece => "not yet clipped",
                    WoolLocation::Ground => "in pasture",
                    WoolLocation::Stored => "shared storage",
                    WoolLocation::Trader => "paid at visitor pile",
                    WoolLocation::Carried => "resident carrying",
                    WoolLocation::Aboard => "aboard",
                    WoolLocation::Exported => "exported",
                };
                text.push_str(&format!("\nWool #{}: {label}", i + 1));
            }
            text.push_str(&format!(
                "\n\nSalt: {} stored / {} used\nU: request collection",
                s.barter.count(SaltLocation::Stored),
                s.barter.count(SaltLocation::Consumed)
            ));
            text
        }
    }
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            InspectorPanel,
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(PANEL_TOP),
                width: px(PANEL_WIDTH),
                height: px(PANEL_HEIGHT),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.10, 0.10, 0.94)),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    InspectorNavigation,
                    Node {
                        column_gap: px(6),
                        ..default()
                    },
                ))
                .with_children(|nav| {
                    for (label, subject) in [
                        ("Home", Some(Subject::Home)),
                        ("H2", Some(Subject::NewHome)),
                        ("R1", Some(Subject::Resident(1))),
                        ("R2", Some(Subject::Resident(2))),
                        ("Boat", Some(Subject::Boat)),
                        ("Close", None),
                    ] {
                        nav.spawn((
                            Button,
                            ActivateOnPress,
                            Hovered::default(),
                            InspectButton(subject),
                            Node {
                                padding: UiRect::axes(px(8), px(5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.28, 0.23)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(label),
                                TextFont {
                                    font_size: px(15).into(),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
            panel.spawn((
                InspectorText,
                Text::new(""),
                TextFont {
                    font_size: px(15).into(),
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.94, 0.84)),
            ));
        });
}
fn refresh(
    state: Res<Inspection>,
    s: Res<Settlement>,
    workers: Query<&Worker>,
    rig: Res<CameraRig>,
    mut panels: Query<&mut Node, (With<InspectorPanel>, Without<InspectorNavigation>)>,
    mut navigation: Query<&mut Node, (With<InspectorNavigation>, Without<InspectorPanel>)>,
    mut text: Query<&mut Text, With<InspectorText>>,
) {
    for mut node in &mut panels {
        node.display = if state.subject.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut node in &mut navigation {
        node.display = if rig.mode == CameraMode::Strategy {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Some(subject) = state.subject {
        let crew: Vec<_> = workers.iter().collect();
        for mut text in &mut text {
            text.0 = description(subject, &s, &crew);
        }
    }
}

fn sync_buttons(
    state: Res<Inspection>,
    workers: Query<&Worker>,
    mut buttons: Query<(&InspectButton, &Hovered, &mut BackgroundColor)>,
) {
    for (button, hovered, mut color) in &mut buttons {
        let absent =
            matches!(button.0,Some(Subject::Resident(id)) if !workers.iter().any(|w|w.id==id));
        color.0 = if absent {
            Color::srgb(0.13, 0.16, 0.15)
        } else if button.0 == state.subject {
            Color::srgb(0.43, 0.35, 0.16)
        } else if hovered.get() {
            Color::srgb(0.32, 0.37, 0.29)
        } else {
            Color::srgb(0.25, 0.28, 0.23)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_nearest_live_subject_without_selecting_through_house_or_tree() {
        let mut s = Settlement::default();
        let mut worker = Worker::default();
        let down = Vec3::NEG_Y;
        assert_eq!(
            pick_subject(
                Vec3::new(worker.position.x, 10., worker.position.y),
                down,
                &s,
                std::iter::once(&worker)
            ),
            Some(Subject::Resident(1))
        );
        assert_eq!(
            pick_subject(Vec3::new(2., 10., 0.), down, &s, std::iter::once(&worker)),
            Some(Subject::Home)
        );
        worker.position = Vec2::new(2., -4.);
        assert_eq!(
            pick_subject(
                Vec3::new(2., 1., 10.),
                Vec3::NEG_Z,
                &s,
                std::iter::once(&worker)
            ),
            Some(Subject::Home)
        );
        worker.position = Vec2::new(2., 4.);
        assert_eq!(
            pick_subject(
                Vec3::new(2., 1., 10.),
                Vec3::NEG_Z,
                &s,
                std::iter::once(&worker)
            ),
            Some(Subject::Resident(1))
        );
        worker.position = crate::simulation::TREE_POSITIONS[0];
        assert_eq!(
            pick_subject(
                Vec3::new(worker.position.x, 10., worker.position.y),
                down,
                &s,
                std::iter::once(&worker)
            ),
            None
        );
        let berth = crate::shipping::BERTH;
        let origin = Vec3::new(berth.x, 10., berth.y);
        assert_eq!(pick_subject(origin, down, &s, std::iter::empty()), None);
        s.voyage.phase = crate::shipping::VoyagePhase::Loading;
        assert_eq!(
            pick_subject(origin, down, &s, std::iter::empty()),
            Some(Subject::Boat)
        );
        s.voyage.phase = crate::shipping::VoyagePhase::Completed;
        assert_eq!(pick_subject(origin, down, &s, std::iter::empty()), None);
        assert_eq!(
            pick_subject(Vec3::splat(40.), down, &s, std::iter::empty()),
            None
        );
    }
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CameraRig>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_plugins((
                crate::simulation::SimulationPlugin,
                crate::orders::OrdersPlugin,
                InspectionPlugin,
            ));
        app.world_mut()
            .resource_mut::<crate::session::Session>()
            .paused = true;
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app.update();
        app
    }
    fn key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
    }
    fn activate(app: &mut App, subject: Option<Subject>) {
        let button = app
            .world_mut()
            .query::<(Entity, &InspectButton)>()
            .iter(app.world())
            .find(|(_, button)| button.0 == subject)
            .unwrap()
            .0;
        app.world_mut().trigger(Activate { entity: button });
        app.update();
    }
    #[test]
    fn focused_ui_cycles_without_mutating_world_and_cannot_leak_clicks_or_buttons() {
        let mut app = app();
        let window = app
            .world_mut()
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .single(app.world())
            .unwrap();
        let before = serde_json::to_vec(app.world().resource::<Settlement>()).unwrap();
        for mode in [CameraMode::Strategy, CameraMode::FirstPerson] {
            app.world_mut().resource_mut::<CameraRig>().mode = mode;
            app.world_mut().resource_mut::<CameraRig>().captured = false;
            app.world_mut().resource_mut::<Inspection>().subject = Some(Subject::Home);
            key(&mut app, KeyCode::KeyI);
            assert_eq!(
                app.world().resource::<Inspection>().subject,
                Some(Subject::Resident(1))
            );
            key(&mut app, KeyCode::KeyI);
            assert_eq!(
                app.world().resource::<Inspection>().subject,
                Some(Subject::Boat)
            );
            key(&mut app, KeyCode::KeyI);
            assert_eq!(app.world().resource::<Inspection>().subject, None);
            key(&mut app, KeyCode::KeyI);
            assert_eq!(
                app.world().resource::<Inspection>().subject,
                Some(Subject::Home)
            );
            key(&mut app, KeyCode::F2);
            assert!(app.world().resource::<Inspection>().details);
            key(&mut app, KeyCode::F2);
            assert!(!app.world().resource::<Inspection>().details);
        }
        app.world_mut().resource_mut::<CameraRig>().mode = CameraMode::Strategy;
        activate(&mut app, Some(Subject::Boat));
        assert_eq!(
            app.world().resource::<Inspection>().subject,
            Some(Subject::Boat)
        );
        activate(&mut app, Some(Subject::Resident(2)));
        assert_eq!(
            app.world().resource::<Inspection>().subject,
            Some(Subject::Boat)
        );
        activate(&mut app, None);
        assert_eq!(app.world().resource::<Inspection>().subject, None);
        assert!(app.world().resource::<Inspection>().claimed_click);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        key(&mut app, KeyCode::KeyI);
        key(&mut app, KeyCode::F2);
        activate(&mut app, Some(Subject::Home));
        assert_eq!(app.world().resource::<Inspection>().subject, None);
        assert!(!app.world().resource::<Inspection>().details);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
        assert_eq!(app.world().resource::<Inspection>().subject, None);
        activate(&mut app, Some(Subject::Home));
        let mut helper = Worker::default();
        helper.id = 2;
        helper.role = crate::simulation::WorkerRole::Coastal;
        let helper = app.world_mut().spawn(helper).id();
        key(&mut app, KeyCode::KeyI);
        key(&mut app, KeyCode::KeyI);
        assert_eq!(
            app.world().resource::<Inspection>().subject,
            Some(Subject::Resident(2))
        );
        app.world_mut().despawn(helper);
        app.update();
        assert_eq!(
            app.world().resource::<Inspection>().subject,
            Some(Subject::Home)
        );
        assert_eq!(
            serde_json::to_vec(app.world().resource::<Settlement>()).unwrap(),
            before
        );
        assert_eq!(
            app.world()
                .resource::<crate::simulation::SimulationClock>()
                .0,
            0.0
        );
        let mut w = app.world_mut().get_mut::<Window>(window).unwrap();
        w.set_cursor_position(Some(Vec2::new(1200., 100.)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().resource::<Inspection>().claimed_click);
    }
    #[test]
    fn descriptions_follow_actual_owned_food_and_the_current_cargo_slot() {
        let mut s = Settlement::default();
        let mut worker = Worker::default();
        worker.carrying_fish = Some(2);
        worker.task = Task::Coastal(C::ToFishHome);
        s.fishery.fish[2] = FishLocation::CarriedPreserved;
        let text = description(Subject::Resident(1), &s, &[&worker]);
        assert!(text.contains("Carrying salted fish home") && text.contains("salted fish #3"));
        s.farm.grain[0] = GrainLocation::Home(1);
        s.farm.grain[1] = GrainLocation::Home(99);
        s.fishery.fish[0] = FishLocation::Home(1);
        s.logs[0] = LogLocation::Home(1);
        let home = description(Subject::Home, &s, &[&worker]);
        assert!(
            home.contains("Pantry: 1 grain / 1 salted fish")
                && home.contains("Owned firewood: 1 / 3")
        );
        s.flock.wool = [
            WoolLocation::Trader,
            WoolLocation::Carried,
            WoolLocation::Aboard,
            WoolLocation::Exported,
        ];
        let text = description(Subject::Boat, &s, &[&worker]);
        for phrase in [
            "Wool #1: paid at visitor pile",
            "Wool #2: resident carrying",
            "Wool #3: aboard",
            "Wool #4: exported",
        ] {
            assert!(text.contains(phrase));
        }
    }
}
