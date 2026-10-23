//! One world and simulation, observed through two camera modes.
mod art;
mod art_assets;
mod calendar;
mod camera;
mod character_motion;
mod cinematic;
mod coast;
mod farming;
mod fishing;
mod hearth;
mod household;
mod inspection;
mod landscape;
mod livestock;
mod motion;
mod orders;
mod performance;
mod presentation;
mod rendering;
mod resident;
mod session;
mod shipping;
mod simulation;
mod surfaces;
mod terrain_material;
mod trade;
mod water;
mod world;
use bevy::prelude::*;

fn main() -> AppExit {
    let options = match performance::RunOptions::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let benchmark = options.benchmark.is_some();
    let mut rig = camera::CameraRig::default();
    if options
        .benchmark
        .as_ref()
        .is_some_and(|b| b.camera == camera::CameraMode::FirstPerson)
    {
        rig.toggle();
    }
    App::new()
        .insert_resource(options.quality)
        .insert_resource(options)
        .insert_resource(rig)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hafthiWorld — Gotland, c. 900".into(),
                resolution: (1280, 800).into(),
                present_mode: if benchmark {
                    bevy::window::PresentMode::AutoNoVsync
                } else {
                    bevy::window::PresentMode::AutoVsync
                },
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.64, 0.75, 0.76)))
        .add_plugins((
            world::WorldPlugin,
            simulation::SimulationPlugin,
            camera::CameraPlugin,
            orders::OrdersPlugin,
            presentation::PresentationPlugin,
            farming::FarmingPlugin,
            fishing::FishingPlugin,
            livestock::LivestockPlugin,
            household::HouseholdPlugin,
            hearth::HearthPlugin,
            trade::TradePlugin,
            performance::PerformancePlugin,
            session::SessionPlugin,
            shipping::ShippingPlugin,
            inspection::InspectionPlugin,
        ))
        .add_plugins((
            resident::ResidentPlugin,
            rendering::CoastalRendererPlugin,
            water::WaterPlugin,
            cinematic::CinematicPlugin,
            surfaces::SurfacePlugin,
            terrain_material::GroundPlugin,
        ))
        .run()
}
