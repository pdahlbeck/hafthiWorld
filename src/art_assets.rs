//! Bundle the small art study so cargo run and standalone binaries use identical assets.
use bevy::{asset::io::embedded::EmbeddedAssetRegistry, prelude::*};
pub fn register(app: &mut App) {
    let registry = app.world().resource::<EmbeddedAssetRegistry>();
    macro_rules! pack {
        ($path:literal) => {
            registry.insert_asset(
                std::path::PathBuf::from(concat!("assets/", $path)),
                std::path::Path::new(concat!("hafthi_world/", $path)),
                include_bytes!(concat!("../assets/", $path)).as_slice(),
            );
        };
    }
    pack!("materials/ground-color.png");
    pack!("materials/ground-normal.png");
    pack!("materials/ground-orm.png");
    pack!("materials/limestone-color.png");
    pack!("materials/limestone-normal.png");
    pack!("materials/limestone-orm.png");
    pack!("materials/wood-color.png");
    pack!("materials/wood-normal.png");
    pack!("materials/wood-orm.png");
    pack!("materials/wool-color.png");
    pack!("materials/wool-normal.png");
    pack!("materials/wool-orm.png");
    pack!("models/coastal-resident.glb");
    pack!("shaders/coastal_water.wesl");
    pack!("shaders/coastal_ground.wesl");
    pack!("materials/meadow-color.png");
    pack!("materials/thatch-color.png");
    pack!("materials/thatch-normal.png");
    pack!("materials/pine-bough.png");
}
