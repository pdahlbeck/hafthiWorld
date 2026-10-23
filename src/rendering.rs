//! Physical sky and image-based illumination. The economy never depends on render state.
use crate::{performance::VisualQuality, world::Daylight};
use bevy::{
    camera::Exposure,
    light::{Atmosphere, atmosphere::ScatteringMedium},
    prelude::*,
};
#[derive(Component)]
struct Moonlight;
pub struct CoastalRendererPlugin;
impl Plugin for CoastalRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, expose.after(crate::world::DaylightUpdate));
    }
}
fn setup(
    mut commands: Commands,
    quality: Res<VisualQuality>,
    mut media: ResMut<Assets<ScatteringMedium>>,
) {
    if *quality == VisualQuality::High {
        commands.spawn((
            Moonlight,
            DirectionalLight {
                color: Color::srgb(0.64, 0.74, 1.0),
                illuminance: 0.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::default().looking_to(Vec3::new(-0.45, -0.8, 0.3), Vec3::Y),
            Name::new("Night readability moonlight"),
        ));
        commands.spawn((
            Atmosphere::earth(media.add(ScatteringMedium::earth(256, 256))),
            Name::new("Baltic atmospheric scattering"),
        ));
    }
}
pub(crate) struct SolarFrame {
    pub rotation: Quat,
    pub exposure: f32,
    pub moonlight: f32,
    pub ambient: f32,
}
pub(crate) fn solar_frame(hour: f32) -> SolarFrame {
    let hour = hour.rem_euclid(24.0);
    let elevation = ((hour - 6.0) / 24.0 * std::f32::consts::TAU).sin();
    let t = ((elevation + 0.15) / 0.4).clamp(0.0, 1.0);
    let altitude = elevation * 1.1;
    let azimuth = -0.65 + (hour - 16.0) * 0.08;
    let rays = Vec3::new(
        -azimuth.sin() * altitude.cos(),
        -altitude.sin(),
        -azimuth.cos() * altitude.cos(),
    );
    // Keep daylight exposure while the sun reaches the horizon. The atmosphere
    // remains luminous through twilight: the former -2..13 EV curve amplified
    // that sky by thousands of times just after sunset. Limit night gain to
    // five stops and introduce it only as the sun sinks through twilight.
    let night = ((-altitude - 0.10) / 0.22).clamp(0.0, 1.0);
    let night = night * night * (3.0 - 2.0 * night);
    let exposure = 13.0 - 5.0 * night;
    // Match fill to the current exposure throughout twilight, avoiding a dark
    // trough before the camera reaches its EV 8 night floor.
    let night_gain = 2.0_f32.powf(exposure - 8.0) * (1.0 - t);
    SolarFrame {
        rotation: Transform::default().looking_to(rays, Vec3::Y).rotation,
        exposure,
        // Deliberately artistic night illumination, matched to EV 8. A real
        // 0.25 lux moon was effectively black at that camera exposure.
        moonlight: 800.0 * night_gain,
        ambient: 400.0 * night_gain,
    }
}
fn expose(
    day: Res<Daylight>,
    mut cameras: Query<&mut Exposure, With<bevy::pbr::AtmosphereSettings>>,
    mut moons: Query<&mut DirectionalLight, With<Moonlight>>,
) {
    let frame = solar_frame(day.hour);
    for mut exposure in &mut cameras {
        exposure.ev100 = frame.exposure;
    }
    for mut moon in &mut moons {
        moon.illuminance = frame.moonlight;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dusk_and_night_have_an_exposure_matched_readability_floor() {
        for hour in [18.0 + 11.0 / 60.0, 18.5, 20.3, 21.0, 0.0, 3.0] {
            let frame = solar_frame(hour);
            // Approximate diffuse signal for a 20% grey surface at the camera.
            // This is a light/exposure contract, not a GPU appearance test.
            let signal = 0.2 * (frame.moonlight * 0.5 + frame.ambient)
                / (std::f32::consts::PI * 1.2 * 2.0_f32.powf(frame.exposure));
            assert!(signal > 0.10 && signal < 0.25, "{hour}: {signal}");
        }
    }
    #[test]
    fn exposure_tracks_the_current_hour_without_a_stale_frame() {
        let mut app = App::new();
        app.init_resource::<Daylight>().add_systems(
            Update,
            (
                expose.after(crate::world::DaylightUpdate),
                (|mut day: ResMut<Daylight>| day.hour = 18.0 + 11.0 / 60.0)
                    .in_set(crate::world::DaylightUpdate),
            ),
        );
        let camera = app
            .world_mut()
            .spawn((
                Exposure { ev100: -2.0 },
                bevy::pbr::AtmosphereSettings::default(),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<Exposure>(camera).unwrap().ev100, 13.0);
    }
    #[test]
    fn bright_twilight_does_not_trigger_extreme_night_gain() {
        // Regression for the reported washed-out scene at 18:11. Both dawn
        // and dusk keep daytime exposure while atmospheric light is strong.
        for hour in [5.8, 6.0, 6.2, 17.8, 18.0, 18.0 + 11.0 / 60.0, 18.2] {
            assert_eq!(solar_frame(hour).exposure, 13.0);
        }
        assert_eq!(solar_frame(0.0).exposure, 8.0);
        let max_gain = 2.0_f32.powf(13.0 - solar_frame(0.0).exposure);
        assert_eq!(max_gain, 32.0);
    }
    #[test]
    fn physical_sun_crosses_the_horizon_and_exposure_restores_from_hour() {
        // A directional light shines down by day and points away below the horizon at night.
        assert!((solar_frame(12.0).rotation * Vec3::NEG_Z).y < -0.7);
        assert!((solar_frame(0.0).rotation * Vec3::NEG_Z).y > 0.7);
        assert_eq!(solar_frame(12.0).moonlight, 0.0);
        assert_eq!(solar_frame(0.0).moonlight, 800.0);
        for step in 0..1440 {
            let hour = step as f32 / 60.0;
            let f = solar_frame(hour);
            let restored = solar_frame(hour + 24.0);
            assert!(f.rotation.is_finite() && (8.0..=13.0).contains(&f.exposure));
            assert!((f.exposure - restored.exposure).abs() < 0.0001);
            assert!(f.rotation.abs_diff_eq(restored.rotation, 0.0001));
            assert!((f.exposure - solar_frame(hour + 1.0 / 60.0).exposure).abs() < 0.2);
        }
    }
}
