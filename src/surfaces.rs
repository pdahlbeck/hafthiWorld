//! One-time CPU mip chains for small authored surface maps, before render extraction.
//! Color is averaged in linear light; normal maps are renormalized. No font/UI textures.
use bevy::{
    asset::AssetId,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat},
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy)]
enum Map {
    Color,
    Linear,
    Normal,
}
pub struct SurfacePlugin;
impl Plugin for SurfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, prepare);
    }
}
fn prepare(
    materials: Res<Assets<StandardMaterial>>,
    ground: Res<Assets<crate::terrain_material::GroundMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut done: Local<HashSet<AssetId<Image>>>,
) {
    let mut maps = HashMap::new();
    for (_, m) in materials.iter() {
        if let Some(h) = &m.base_color_texture {
            maps.entry(h.id()).or_insert(Map::Color);
        }
        if let Some(h) = &m.metallic_roughness_texture {
            maps.entry(h.id()).or_insert(Map::Linear);
        }
        if let Some(h) = &m.normal_map_texture {
            maps.insert(h.id(), Map::Normal);
        }
    }
    for (_, g) in ground.iter() {
        maps.insert(g.extension.color.id(), Map::Color);
        if let Some(h) = &g.base.base_color_texture {
            maps.entry(h.id()).or_insert(Map::Color);
        }
        if let Some(h) = &g.base.normal_map_texture {
            maps.insert(h.id(), Map::Normal);
        }
        if let Some(h) = &g.base.metallic_roughness_texture {
            maps.entry(h.id()).or_insert(Map::Linear);
        }
    }
    for (id, kind) in maps {
        if done.contains(&id) {
            continue;
        }
        if let Some(mut image) = images.get_mut(id) {
            build_mips(&mut image, kind);
            done.insert(id);
        }
    }
}
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1. / 2.4) - 0.055
    }
}
fn build_mips(image: &mut Image, kind: Map) {
    let descriptor = &image.texture_descriptor;
    if descriptor.dimension != TextureDimension::D2
        || descriptor.size.depth_or_array_layers != 1
        || descriptor.mip_level_count != 1
        || !matches!(
            descriptor.format,
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb
        )
    {
        return;
    }
    let (mut width, mut height) = (descriptor.size.width, descriptor.size.height);
    if width == 0 || height == 0 {
        return;
    }
    let Some(data) = image.data.as_mut() else {
        return;
    };
    if data.len() != width as usize * height as usize * 4 {
        return;
    }
    let color = matches!(kind, Map::Color) && descriptor.format == TextureFormat::Rgba8UnormSrgb;
    let mut previous = data.clone();
    let mut levels = 1;
    while width > 1 || height > 1 {
        let (nw, nh) = ((width / 2).max(1), (height / 2).max(1));
        let mut next = Vec::with_capacity((nw * nh * 4) as usize);
        for y in 0..nh {
            for x in 0..nw {
                let mut pixel = [0.; 4];
                let x0 = x as f32 * width as f32 / nw as f32;
                let x1 = (x + 1) as f32 * width as f32 / nw as f32;
                let y0 = y as f32 * height as f32 / nh as f32;
                let y1 = (y + 1) as f32 * height as f32 / nh as f32;
                for sy in y0.floor() as u32..(y1.ceil() as u32).min(height) {
                    for sx in x0.floor() as u32..(x1.ceil() as u32).min(width) {
                        let weight = ((sx as f32 + 1.).min(x1) - (sx as f32).max(x0))
                            * ((sy as f32 + 1.).min(y1) - (sy as f32).max(y0))
                            / ((x1 - x0) * (y1 - y0));
                        let offset = ((sy * width + sx) * 4) as usize;
                        for channel in 0..4 {
                            let mut c = previous[offset + channel] as f32 / 255.;
                            if color && channel < 3 {
                                c = linear(c);
                            }
                            pixel[channel] += c * weight;
                        }
                    }
                }
                if matches!(kind, Map::Normal) {
                    let n = (Vec3::new(pixel[0], pixel[1], pixel[2]) * 2. - Vec3::ONE)
                        .normalize_or_zero()
                        * 0.5
                        + Vec3::splat(0.5);
                    pixel[..3].copy_from_slice(&n.to_array());
                }
                for (channel, mut c) in pixel.into_iter().enumerate() {
                    if color && channel < 3 {
                        c = srgb(c);
                    }
                    next.push((c.clamp(0., 1.) * 255.).round() as u8);
                }
            }
        }
        data.extend_from_slice(&next);
        previous = next;
        width = nw;
        height = nh;
        levels += 1;
    }
    image.texture_descriptor.mip_level_count = levels;
    if let ImageSampler::Descriptor(sampler) = &mut image.sampler {
        sampler.anisotropy_clamp = 8;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::RenderAssetUsages, render::render_resource::Extent3d};
    #[test]
    fn mip_chain_averages_srgb_in_linear_light_and_reaches_one_texel() {
        let mut image = Image::new_fill(
            Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let data = image.data.as_mut().unwrap();
        data[4..8].copy_from_slice(&[255; 4]);
        data[12..16].copy_from_slice(&[255; 4]);
        build_mips(&mut image, Map::Color);
        assert_eq!(image.texture_descriptor.mip_level_count, 2);
        assert_eq!(&image.data.as_ref().unwrap()[16..], &[188, 188, 188, 255]);
    }
    #[test]
    fn odd_mip_dimensions_keep_the_whole_surface_instead_of_cropping_edges() {
        let mut image = Image::new_fill(
            Extent3d {
                width: 6,
                height: 6,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        for y in 0..6 {
            for x in 0..6 {
                if x >= 4 || y >= 4 {
                    let p = (y * 6 + x) * 4;
                    image.data.as_mut().unwrap()[p..p + 4].copy_from_slice(&[255; 4]);
                }
            }
        }
        build_mips(&mut image, Map::Color);
        assert_eq!(image.texture_descriptor.mip_level_count, 3);
        assert_eq!(image.data.as_ref().unwrap().len(), (36 + 9 + 1) * 4);
        assert_eq!(&image.data.as_ref().unwrap()[180..], &[197, 197, 197, 255]);
    }
    #[test]
    fn normal_mips_are_unit_length_and_dont_process_twice() {
        let mut image = Image::new_fill(
            Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[128, 128, 255, 255],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        build_mips(&mut image, Map::Normal);
        assert_eq!(image.texture_descriptor.mip_level_count, 3);
        assert_eq!(image.data.as_ref().unwrap().len(), (16 + 4 + 1) * 4);
        let last = &image.data.as_ref().unwrap()[80..83];
        let n = Vec3::new(last[0] as f32, last[1] as f32, last[2] as f32) / 255. * 2. - Vec3::ONE;
        assert!((n.length() - 1.).abs() < 0.01);
        let before = image.data.clone();
        build_mips(&mut image, Map::Normal);
        assert_eq!(image.data, before);
    }
}
