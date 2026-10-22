//! Screenshot/viewing mode changes UI visibility only, never simulation or camera state.
use bevy::{prelude::*, window::PrimaryWindow};
pub struct CinematicPlugin;
impl Plugin for CinematicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cinematic>().add_systems(Update, toggle);
    }
}
#[derive(Resource, Default)]
struct Cinematic(bool);
#[derive(Component)]
struct PreviousVisibility(Visibility);
type UiRoots<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Visibility,
        Option<&'static PreviousVisibility>,
    ),
    (With<Node>, Without<ChildOf>),
>;
fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mode: ResMut<Cinematic>,
    mut commands: Commands,
    mut roots: UiRoots,
) {
    if !windows.single().is_ok_and(|w| w.focused) || !keys.just_pressed(KeyCode::F4) {
        return;
    }
    mode.0 = !mode.0;
    for (entity, mut visibility, previous) in &mut roots {
        if mode.0 {
            commands
                .entity(entity)
                .insert(PreviousVisibility(*visibility));
            *visibility = Visibility::Hidden;
        } else if let Some(previous) = previous {
            *visibility = previous.0;
            commands.entity(entity).remove::<PreviousVisibility>();
        }
    }
}
