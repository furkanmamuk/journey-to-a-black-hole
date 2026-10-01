use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType, Default)]
pub struct BlackHoleUniform {
    pub camera: Vec4,
    pub right: Vec4,
    pub up: Vec4,
    pub forward: Vec4,
    /// rs, inner radius, outer radius, maximum affine distance
    pub geometry: Vec4,
    /// steps, step scale, exposure multiplier, reserved (no animation time)
    pub integration: Vec4,
    /// lensing, disk, stars, Doppler temperature shift
    pub features: Vec4,
    /// beaming, gravitational redshift, exhausted-ray diagnostic, reserved
    pub relativity: Vec4,
    /// Disk plane normal, emission multiplier. Disk intersects in world space.
    pub disk: Vec4,
    /// Center of the gravitational field, reserved.
    pub center: Vec4,
    /// star seed, sky mode (stars/grid/both), ray-class diagnostic, reserved
    pub sky: UVec4,
    /// Signed a/M, Kerr enabled, temperature scale K, constraint diagnostic.
    pub model: Vec4,
    pub axes_x: Vec4,
    pub axes_y: Vec4,
    pub axes_z: Vec4,
}
