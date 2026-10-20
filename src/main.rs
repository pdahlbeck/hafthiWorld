//! One world and simulation, observed through two camera modes.
mod camera;
mod presentation;
mod simulation;
mod world;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hafthiWorld — Gotland, c. 900".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.69, 0.84, 0.94)))
        .add_plugins((
            world::WorldPlugin,
            simulation::SimulationPlugin,
            camera::CameraPlugin,
            presentation::PresentationPlugin,
        ))
        .run();
}
