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
        app.add_systems(Startup, setup).add_systems(Update, expose);
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
    SolarFrame {
        rotation: Transform::default().looking_to(rays, Vec3::Y).rotation,
        exposure: -2.0 + 15.0 * t,
        moonlight: 0.25 * (1.0 - t),
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
    fn physical_sun_crosses_the_horizon_and_exposure_restores_from_hour() {
        // A directional light shines down by day and points away below the horizon at night.
        assert!((solar_frame(12.0).rotation * Vec3::NEG_Z).y < -0.7);
        assert!((solar_frame(0.0).rotation * Vec3::NEG_Z).y > 0.7);
        assert_eq!(solar_frame(12.0).moonlight, 0.0);
        assert_eq!(solar_frame(0.0).moonlight, 0.25);
        for step in 0..1440 {
            let hour = step as f32 / 60.0;
            let f = solar_frame(hour);
            let restored = solar_frame(hour + 24.0);
            assert!(f.rotation.is_finite() && (-2.0..=13.0).contains(&f.exposure));
            assert!((f.exposure - restored.exposure).abs() < 0.0001);
            assert!(f.rotation.abs_diff_eq(restored.rotation, 0.0001));
            assert!((f.exposure - solar_frame(hour + 1.0 / 60.0).exposure).abs() < 0.2);
        }
    }
}
