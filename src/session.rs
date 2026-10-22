//! Versioned local snapshots. Scene entities and camera controls remain live on restore.
use crate::{
    camera::CameraInput,
    simulation::{Settlement, SimulationClock, Worker, WorkerRole},
    world::Daylight,
};
use bevy::{prelude::*, window::PrimaryWindow};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const SAVE_FILE: &str = "hafthi-save.json";
const MAX_SAVE_BYTES: u64 = 1024 * 1024;
#[derive(Resource, Default)]
pub struct Session {
    pub paused: bool,
    message: String,
}
pub fn running(session: Res<Session>) -> bool {
    !session.paused
}
pub struct SessionPlugin;
impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Session>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, controls.after(bevy::input::InputSystems))
            .add_systems(Update, status.after(CameraInput));
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    settlement: Settlement,
    workers: Vec<Worker>,
    seconds: f64,
    daylight: Daylight,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    version: u32,
    checksum: u64,
    // A checksum of the exact payload detects truncation and accidental edits.
    payload: String,
}
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |hash, b| {
        (hash ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
impl Snapshot {
    fn capture(world: &mut World) -> Self {
        let mut workers: Vec<Worker> = world.query::<&Worker>().iter(world).cloned().collect();
        workers.sort_by_key(|w| w.id);
        Self {
            settlement: world.resource::<Settlement>().clone(),
            workers,
            seconds: world.resource::<SimulationClock>().0,
            daylight: world.resource::<Daylight>().clone(),
        }
    }
    fn validate(&self) -> Result<(), String> {
        let s = &self.settlement;
        if !self.daylight.hour.is_finite()
            || !(0.0..24.0).contains(&self.daylight.hour)
            || !s.farm.growth.is_finite()
            || s.farm.growth < 0.0
            || !s.calendar.remaining().is_finite()
            || s.calendar.year == 0
            || !s.household.next_use().is_finite()
            || !s.household.hearth.next_use().is_finite()
            || s.flock
                .sheep
                .iter()
                .any(|sheep| !sheep.position.is_finite() || !sheep.yaw.is_finite())
            || s.flock.ground.iter().any(|v| !v.is_finite())
            || !self.seconds.is_finite()
            || self.seconds < 0.0
            || s.household.id != 1
            || s.household.fish_used > s.household.food_used
            || self.workers.len() != if s.coastal_assigned { 2 } else { 1 }
            || !self
                .workers
                .iter()
                .any(|w| w.id == 1 && w.role == WorkerRole::General)
            || (s.coastal_assigned
                && !self
                    .workers
                    .iter()
                    .any(|w| w.id == 2 && w.role == WorkerRole::Coastal))
            || s.coastal_owner
                .is_some_and(|id| !self.workers.iter().any(|w| w.id == id))
            || self.workers.iter().any(|w| !w.valid_snapshot())
        {
            return Err("The saved session has invalid residents or time.".into());
        }
        // Every physical carried item must belong to exactly one resident.
        let cargo = [
            (
                s.logs
                    .iter()
                    .map(|l| *l == crate::simulation::LogLocation::Carried)
                    .collect::<Vec<_>>(),
                self.workers.iter().map(|w| w.carrying).collect::<Vec<_>>(),
            ),
            (
                s.farm
                    .grain
                    .iter()
                    .map(|l| *l == crate::farming::GrainLocation::Carried)
                    .collect(),
                self.workers.iter().map(|w| w.carrying_grain).collect(),
            ),
            (
                s.flock
                    .wool
                    .iter()
                    .map(|l| *l == crate::livestock::WoolLocation::Carried)
                    .collect(),
                self.workers.iter().map(|w| w.carrying_wool).collect(),
            ),
            (
                s.barter
                    .salt
                    .iter()
                    .map(|l| *l == crate::trade::SaltLocation::Carried)
                    .collect(),
                self.workers.iter().map(|w| w.carrying_salt).collect(),
            ),
            (
                s.fishery
                    .fish
                    .iter()
                    .map(|l| {
                        matches!(
                            l,
                            crate::fishing::FishLocation::CarriedRaw
                                | crate::fishing::FishLocation::CarriedPreserved
                        )
                    })
                    .collect(),
                self.workers.iter().map(|w| w.carrying_fish).collect(),
            ),
        ];
        for (slots, carriers) in cargo {
            if carriers.iter().flatten().any(|&i| i >= slots.len())
                || slots.iter().enumerate().any(|(i, &carried)| {
                    carriers.iter().filter(|&&slot| slot == Some(i)).count() != usize::from(carried)
                })
            {
                return Err("The saved session has inconsistent cargo.".into());
            }
        }
        Ok(())
    }
    fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let payload = serde_json::to_string(self).map_err(|e| e.to_string())?;
        serde_json::to_vec_pretty(&SaveFile {
            version: 1,
            checksum: checksum(payload.as_bytes()),
            payload,
        })
        .map_err(|e| e.to_string())
    }
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let file: SaveFile = serde_json::from_slice(bytes)
            .map_err(|_| "The save file is incomplete or unreadable.".to_string())?;
        if file.version != 1 {
            return Err("This save version is not supported.".into());
        }
        if checksum(file.payload.as_bytes()) != file.checksum {
            return Err("The save file is damaged.".into());
        }
        let snapshot: Self = serde_json::from_str(&file.payload)
            .map_err(|_| "The saved session is unreadable.".to_string())?;
        snapshot.validate()?;
        Ok(snapshot)
    }
    fn restore(self, world: &mut World) {
        let mut existing: Vec<_> = world
            .query::<(Entity, &Worker)>()
            .iter(world)
            .map(|(e, w)| (e, w.id))
            .collect();
        for worker in self.workers {
            if let Some(index) = existing.iter().position(|(_, id)| *id == worker.id) {
                let (entity, _) = existing.swap_remove(index);
                world.entity_mut(entity).insert(worker);
            } else {
                world.spawn(worker);
            }
        }
        for (entity, _) in existing {
            world.despawn(entity);
        }
        world.insert_resource(self.settlement);
        world.resource_mut::<SimulationClock>().0 = self.seconds;
        world.insert_resource(self.daylight);
        world.resource_mut::<Session>().paused = true;
    }
}
fn write_save(path: &Path, snapshot: &Snapshot) -> Result<(), String> {
    let bytes = snapshot.encode()?;
    let temp = PathBuf::from(format!("{}.tmp", path.display()));
    // create_new avoids following or overwriting an existing temporary file.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| format!("Cannot create save: {e}"))?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e| format!("Cannot finish save: {e}"))
}
fn read_save(path: &Path) -> Result<Snapshot, String> {
    let file = fs::File::open(path).map_err(|e| format!("Cannot open save: {e}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_SAVE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SAVE_BYTES {
        return Err("The save file is too large.".into());
    }
    Snapshot::decode(&bytes)
}
fn controls(world: &mut World) {
    if !world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused)
    {
        return;
    }
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let pause = keys.just_pressed(KeyCode::Space);
    let save = keys.just_pressed(KeyCode::F5);
    let load = keys.just_pressed(KeyCode::F9);
    if pause {
        let mut session = world.resource_mut::<Session>();
        session.paused = !session.paused;
    }
    // Loading takes priority if both keys arrive in the same frame.
    let result = if load {
        match read_save(Path::new(SAVE_FILE)) {
            Ok(snapshot) => {
                snapshot.restore(world);
                Ok("Session loaded (paused)".to_string())
            }
            Err(error) => Err(error),
        }
    } else if save {
        write_save(Path::new(SAVE_FILE), &Snapshot::capture(world))
            .map(|_| "Session saved".to_string())
    } else {
        return;
    };
    world.resource_mut::<Session>().message = match result {
        Ok(message) => message,
        Err(error) => {
            warn!("{error}");
            error
        }
    };
}
#[derive(Component)]
struct SessionStatus;
fn setup(mut commands: Commands) {
    commands.spawn((
        SessionStatus,
        Text::new(""),
        TextFont {
            font_size: px(16.0).into(),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.06, 0.10, 0.10, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            bottom: px(125),
            padding: UiRect::all(px(8)),
            ..default()
        },
    ));
}
fn status(session: Res<Session>, mut text: Query<&mut Text, With<SessionStatus>>) {
    if let Ok(mut text) = text.single_mut() {
        text.0 = format!(
            "{} | Space: pause | F5: save | F9: load\n{}",
            if session.paused { "PAUSED" } else { "RUNNING" },
            session.message
        );
    }
}

// Keep serialization local to the simulation instead of enabling it across Bevy assets.
pub(crate) mod vector {
    use super::*;
    pub fn serialize<S: serde::Serializer>(v: &Vec2, serializer: S) -> Result<S::Ok, S::Error> {
        v.to_array().serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec2, D::Error> {
        <[f32; 2]>::deserialize(deserializer).map(Vec2::from_array)
    }
}
pub(crate) mod vectors {
    use super::*;
    pub fn serialize<S: serde::Serializer, const N: usize>(
        v: &[Vec2; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        v.iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<[Vec2; N], D::Error> {
        Vec::<[f32; 2]>::deserialize(deserializer)?
            .into_iter()
            .map(Vec2::from_array)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| serde::de::Error::custom("wrong vector count"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        camera::CameraRig,
        simulation::{SimulationPlugin, Task},
    };
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .insert_resource(Time::<Fixed>::from_seconds(0.1))
            .init_resource::<Daylight>()
            .init_resource::<CameraRig>()
            .add_plugins(SimulationPlugin);
        app.update();
        app
    }
    fn bytes(app: &mut App) -> Vec<u8> {
        Snapshot::capture(app.world_mut()).encode().unwrap()
    }
    #[test]
    fn roundtrip_busy_two_resident_chain_resumes_identically() {
        let mut original = app();
        {
            let mut s = original.world_mut().resource_mut::<Settlement>();
            s.request_helper();
            s.fishery.request_fishing();
            s.fishery.request_salting();
            s.flock.request_shear();
            s.barter.request();
            s.household.request_supply();
            s.request_harvest(0);
            s.request_harvest(1);
            s.request_harvest(2);
            s.request_build();
            s.farm.request_sow();
            s.farm.request_reserve();
            s.household.hearth.request();
        }
        let mut seen_cargo = false;
        let mut seen_salting = false;
        let mut resumed = app();
        for tick in 0..7500 {
            original
                .world_mut()
                .resource_mut::<Settlement>()
                .farm
                .request_reap();
            original.update();
            let snap = Snapshot::capture(original.world_mut());
            seen_cargo |= snap.workers.iter().any(|w| {
                w.carrying.is_some()
                    || w.carrying_fish.is_some()
                    || w.carrying_wool.is_some()
                    || w.carrying_salt.is_some()
            });
            seen_salting |= snap
                .workers
                .iter()
                .any(|w| w.task == Task::Coastal(crate::fishing::CoastalTask::Salting));
            if tick % 37 == 0 {
                let encoded = snap.encode().unwrap();
                Snapshot::decode(&encoded)
                    .unwrap()
                    .restore(resumed.world_mut());
                assert!(resumed.world().resource::<Session>().paused);
                assert_eq!(encoded, bytes(&mut resumed));
                // Advance both from the exact same saved job, timer and cargo state.
                resumed.world_mut().resource_mut::<Session>().paused = false;
                resumed.update();
                original.update();
                assert_eq!(bytes(&mut original), bytes(&mut resumed));
            }
        }
        assert!(seen_cargo && seen_salting);
        assert_eq!(
            original
                .world()
                .resource::<Settlement>()
                .household
                .fish_used,
            4
        );
    }
    #[test]
    fn pause_holds_work_and_environment_without_blocking_camera_changes() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<Settlement>()
            .request_helper();
        app.world_mut().resource_mut::<Session>().paused = true;
        let before = bytes(&mut app);
        for _ in 0..50 {
            app.world_mut().resource_mut::<CameraRig>().toggle();
            app.update();
        }
        assert_eq!(before, bytes(&mut app));
        app.world_mut().resource_mut::<Session>().paused = false;
        app.update();
        assert_eq!(
            app.world_mut().query::<&Worker>().iter(app.world()).count(),
            2
        );
        assert_ne!(before, bytes(&mut app));
    }
    #[test]
    fn restoring_roster_both_directions_reuses_primary_and_never_duplicates() {
        let mut app = app();
        let solo = bytes(&mut app);
        let primary = app
            .world_mut()
            .query::<(Entity, &Worker)>()
            .iter(app.world())
            .next()
            .unwrap()
            .0;
        app.world_mut()
            .resource_mut::<Settlement>()
            .request_helper();
        app.update();
        let crew = bytes(&mut app);
        for _ in 0..3 {
            Snapshot::decode(&solo).unwrap().restore(app.world_mut());
            assert_eq!(
                app.world_mut().query::<&Worker>().iter(app.world()).count(),
                1
            );
            assert_eq!(app.world().get::<Worker>(primary).unwrap().id, 1);
            Snapshot::decode(&crew).unwrap().restore(app.world_mut());
            assert_eq!(
                app.world_mut().query::<&Worker>().iter(app.world()).count(),
                2
            );
        }
    }
    #[test]
    fn session_keys_require_focus_and_work_with_released_first_person_mouse() {
        let mut app = app();
        app.init_resource::<ButtonInput<KeyCode>>();
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: false,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        controls(app.world_mut());
        assert!(!app.world().resource::<Session>().paused);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().resource_mut::<CameraRig>().toggle();
        app.world_mut().resource_mut::<CameraRig>().captured = false;
        controls(app.world_mut());
        assert!(app.world().resource::<Session>().paused);
    }
    #[test]
    fn restored_roster_removes_helper_body_and_children_then_recreates_once() {
        let mut app = app();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<crate::orders::OrderState>()
            .add_plugins(crate::presentation::PresentationPlugin);
        app.update();
        let solo = bytes(&mut app);
        app.world_mut()
            .resource_mut::<Settlement>()
            .request_helper();
        app.update();
        app.update();
        let crew = bytes(&mut app);
        let helper = app
            .world_mut()
            .query::<(Entity, &Name)>()
            .iter(app.world())
            .find(|(_, name)| name.as_str() == "Resident 2 coastal worker")
            .unwrap()
            .0;
        let children = app
            .world()
            .get::<Children>(helper)
            .unwrap()
            .iter()
            .collect::<Vec<_>>();
        Snapshot::decode(&solo).unwrap().restore(app.world_mut());
        app.update();
        assert!(app.world().get_entity(helper).is_err());
        for child in children {
            assert!(app.world().get_entity(child).is_err());
        }
        for _ in 0..3 {
            Snapshot::decode(&crew).unwrap().restore(app.world_mut());
            app.update();
            app.update();
            assert_eq!(
                app.world_mut()
                    .query::<&Name>()
                    .iter(app.world())
                    .filter(|n| n.as_str() == "Resident 2 coastal worker")
                    .count(),
                1
            );
        }
    }
    #[test]
    fn atomic_files_preserve_previous_save_on_failure_and_reject_damage() {
        let mut app = app();
        let directory = std::env::temp_dir().join(format!("hafthi-session-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("save.json");
        let snapshot = Snapshot::capture(app.world_mut());
        write_save(&path, &snapshot).unwrap();
        let original = fs::read(&path).unwrap();
        fs::write(directory.join("save.json.tmp"), b"occupied").unwrap();
        assert!(write_save(&path, &snapshot).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        fs::remove_file(directory.join("save.json.tmp")).unwrap();
        write_save(&path, &snapshot).unwrap();
        assert_eq!(read_save(&path).unwrap().encode().unwrap(), original);
        let before = bytes(&mut app);
        for broken in [b"{".as_slice(), &original[..original.len() / 2]] {
            assert!(Snapshot::decode(broken).is_err());
        }
        let mut file: SaveFile = serde_json::from_slice(&original).unwrap();
        file.payload.push(' ');
        assert!(Snapshot::decode(&serde_json::to_vec(&file).unwrap()).is_err());
        file.version = 99;
        assert!(Snapshot::decode(&serde_json::to_vec(&file).unwrap()).is_err());
        assert_eq!(before, bytes(&mut app));
        fs::remove_dir_all(directory).unwrap();
    }
}
