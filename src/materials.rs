use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
pub struct EngineeringMaterials {
    pub metal: Handle<StandardMaterial>,
    pub coating: Handle<StandardMaterial>,
    pub frame: Handle<StandardMaterial>,
    pub ceramic: Handle<StandardMaterial>,
    pub instrument: Handle<StandardMaterial>,
}
pub fn create(
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> EngineeringMaterials {
    // StandardMaterial uses G=roughness, B=metallic. Directional, clean tooling marks.
    let mut data = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64 {
        for x in 0..64 {
            let stripe = ((y as f32 * 0.49).sin() * 7.0 + (x as f32 * 0.1).sin() * 2.0) as i32;
            data.extend_from_slice(&[255, (88 + stripe) as u8, 255, 255]);
        }
    }
    let texture = images.add(Image::new(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    ));
    EngineeringMaterials {
        metal: materials.add(StandardMaterial {
            base_color: Color::srgb(0.67, 0.69, 0.72),
            metallic: 1.0,
            perceptual_roughness: 1.0,
            metallic_roughness_texture: Some(texture),
            ..default()
        }),
        coating: materials.add(StandardMaterial {
            base_color: Color::srgb(0.045, 0.05, 0.055),
            metallic: 0.0,
            perceptual_roughness: 0.87,
            ..default()
        }),
        frame: materials.add(StandardMaterial {
            base_color: Color::srgb(0.44, 0.48, 0.51),
            metallic: 1.0,
            perceptual_roughness: 0.46,
            ..default()
        }),
        ceramic: materials.add(StandardMaterial {
            base_color: Color::srgb(0.81, 0.79, 0.7),
            metallic: 0.0,
            perceptual_roughness: 0.62,
            ..default()
        }),
        instrument: materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.12, 0.11),
            emissive: LinearRgba::rgb(4.0, 90.0, 65.0),
            perceptual_roughness: 0.3,
            ..default()
        }),
    }
}
