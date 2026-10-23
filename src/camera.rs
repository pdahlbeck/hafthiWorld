//! Two views of one running world. Camera transitions never touch simulation state.
use crate::world;
use bevy::{
    camera::{Exposure, Hdr},
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRig>()
            .add_systems(Startup, setup)
            .add_systems(Update, controls.in_set(CameraInput));
    }
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CameraInput;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    Strategy,
    FirstPerson,
}
#[derive(Clone, Debug, PartialEq)]
struct StrategyView {
    yaw: f32,
    pitch: f32,
    distance: f32,
    focus: Vec3,
}
#[derive(Resource)]
pub struct CameraRig {
    pub mode: CameraMode,
    pub captured: bool,
    strategy: StrategyView,
    position: Vec2,
    yaw: f32,
    pitch: f32,
}
impl Default for CameraRig {
    fn default() -> Self {
        Self {
            mode: CameraMode::Strategy,
            captured: false,
            strategy: StrategyView {
                yaw: -1.0,
                pitch: 0.50,
                distance: 32.0,
                focus: Vec3::new(0., 0., 1.5),
            },
            position: Vec2::new(-2.5, 6.5),
            yaw: 0.0,
            pitch: -0.25,
        }
    }
}
impl CameraRig {
    pub fn toggle(&mut self) {
        self.mode = match self.mode {
            CameraMode::Strategy => CameraMode::FirstPerson,
            CameraMode::FirstPerson => CameraMode::Strategy,
        };
        self.captured = self.mode == CameraMode::FirstPerson;
    }
    fn transform(&self) -> Transform {
        match self.mode {
            CameraMode::Strategy => {
                let s = &self.strategy;
                let offset = Vec3::new(
                    s.pitch.cos() * s.yaw.sin(),
                    s.pitch.sin(),
                    s.pitch.cos() * s.yaw.cos(),
                ) * s.distance;
                Transform::from_translation(s.focus + offset).looking_at(s.focus, Vec3::Y)
            }
            CameraMode::FirstPerson => Transform::from_xyz(
                self.position.x,
                crate::landscape::height(self.position) + 1.65,
                self.position.y,
            )
            .with_rotation(Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)),
        }
    }
}
#[derive(Component)]
struct WorldCamera;
fn setup(
    mut commands: Commands,
    rig: Res<CameraRig>,
    quality: Option<Res<crate::performance::VisualQuality>>,
) {
    let mut camera = commands.spawn((
        Camera3d::default(),
        rig.transform(),
        WorldCamera,
        DistanceFog {
            color: Color::srgb(0.64, 0.75, 0.76),
            falloff: FogFalloff::Linear {
                start: 38.0,
                end: 135.0,
            },
            ..default()
        },
    ));
    if quality.is_some_and(|q| *q == crate::performance::VisualQuality::High) {
        camera.remove::<DistanceFog>();
        camera.insert((
            Hdr,
            Exposure { ev100: 13.0 },
            bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
            bevy::pbr::AtmosphereSettings::default(),
            bevy::light::AtmosphereEnvironmentMapLight::default(),
            Msaa::Off,
            bevy::light::ShadowFilteringMethod::Temporal,
            bevy::pbr::ScreenSpaceAmbientOcclusion {
                radius: 0.6,
                constant_object_thickness: 0.10,
                quality_level: bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::Medium,
            },
            bevy::anti_alias::taa::TemporalAntiAliasing::default(),
            bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening {
                sharpening_strength: 0.45,
                ..default()
            },
        ));
    }
}
fn axis(keys: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    f32::from(keys.pressed(positive)) - f32::from(keys.pressed(negative))
}
#[derive(bevy::ecs::system::SystemParam)]
struct CameraControls<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    buttons: Res<'w, ButtonInput<MouseButton>>,
    motion: Res<'w, AccumulatedMouseMotion>,
    scroll: Res<'w, AccumulatedMouseScroll>,
    time: Res<'w, Time>,
}
fn controls(
    input: CameraControls,
    mut rig: ResMut<CameraRig>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut camera: Query<&mut Transform, With<WorldCamera>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok((window, mut cursor)) = windows.single_mut() else {
        return;
    };
    if !window.focused {
        rig.captured = false;
    }
    if window.focused {
        if input.keys.just_pressed(KeyCode::Tab) {
            rig.toggle();
        }
        if input.keys.just_pressed(KeyCode::Escape) {
            if rig.captured {
                rig.captured = false;
            } else if rig.mode == CameraMode::FirstPerson {
                rig.toggle();
            } else {
                exit.write(AppExit::Success);
            }
        } else if rig.mode == CameraMode::FirstPerson
            && input.buttons.just_pressed(MouseButton::Left)
        {
            rig.captured = true;
        }
        let dt = input.time.delta_secs().min(0.1);
        match rig.mode {
            CameraMode::Strategy => {
                let s = &mut rig.strategy;
                s.yaw += axis(&input.keys, KeyCode::KeyD, KeyCode::KeyA) * dt;
                s.pitch += axis(&input.keys, KeyCode::KeyW, KeyCode::KeyS) * dt * 0.7;
                if input.buttons.pressed(MouseButton::Right) {
                    s.yaw -= input.motion.delta.x * 0.003;
                    s.pitch += input.motion.delta.y * 0.003;
                }
                s.distance += axis(&input.keys, KeyCode::KeyE, KeyCode::KeyQ) * dt * 12.0
                    - input.scroll.delta.y * 2.0;
                s.focus.x += axis(&input.keys, KeyCode::ArrowRight, KeyCode::ArrowLeft) * dt * 5.0;
                s.focus.z -= axis(&input.keys, KeyCode::ArrowUp, KeyCode::ArrowDown) * dt * 5.0;
                s.focus.x = s.focus.x.clamp(-45.0, 45.0);
                s.focus.z = s.focus.z.clamp(-65.0, 14.0);
                s.pitch = s.pitch.clamp(0.15, 1.3);
                s.distance = s.distance.clamp(6.0, 90.0);
            }
            CameraMode::FirstPerson if rig.captured => {
                rig.yaw -= input.motion.delta.x * 0.003;
                rig.pitch = (rig.pitch - input.motion.delta.y * 0.003).clamp(-1.3, 1.3);
                let forward = Vec2::new(-rig.yaw.sin(), -rig.yaw.cos());
                let right = Vec2::new(rig.yaw.cos(), -rig.yaw.sin());
                let movement = (forward * axis(&input.keys, KeyCode::KeyW, KeyCode::KeyS)
                    + right * axis(&input.keys, KeyCode::KeyD, KeyCode::KeyA))
                .normalize_or_zero();
                let speed = if input.keys.pressed(KeyCode::ShiftLeft) {
                    5.0
                } else {
                    3.0
                };
                rig.position = world::slide(rig.position, movement * dt * speed);
            }
            _ => {}
        }
    }
    cursor.visible = !rig.captured;
    cursor.grab_mode = if rig.captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if let Ok(mut transform) = camera.single_mut() {
        *transform = rig.transform();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_preserves_strategy_and_ground_positions() {
        let mut rig = CameraRig::default();
        rig.strategy.focus = Vec3::new(-3.0, 0.0, 2.0);
        rig.strategy.yaw = 1.2;
        let strategy = rig.strategy.clone();
        let transform = rig.transform();
        assert!(world::walkable(rig.position));
        rig.toggle();
        assert!(rig.captured);
        rig.position = Vec2::new(-4.0, 2.0);
        rig.toggle();
        assert!(!rig.captured);
        assert_eq!(rig.strategy, strategy);
        assert_eq!(rig.transform(), transform);
        rig.toggle();
        assert_eq!(rig.position, Vec2::new(-4.0, 2.0));
    }
    #[test]
    fn focus_loss_releases_cursor_and_escape_does_not_quit_captured_view() {
        let mut app = App::new();
        app.init_resource::<CameraRig>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .insert_resource(Time::<()>::default())
            .add_message::<AppExit>()
            .add_systems(Update, controls);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                CursorOptions::default(),
                PrimaryWindow,
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::Locked
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.update();
        assert!(!app.world().resource::<CameraRig>().captured);
        assert!(app.world().get::<CursorOptions>(window).unwrap().visible);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert_eq!(
            app.world().resource::<CameraRig>().mode,
            CameraMode::FirstPerson
        );
        assert!(!app.world().resource::<CameraRig>().captured);
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    }
}
