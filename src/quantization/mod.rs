pub mod pipeline;
use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};
pub use pipeline::QuantizationPlugin;
#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct QuantizeUniform {
    /// grid width, grid height, tonal levels, dither strength
    pub parameters: Vec4,
    /// spatial quantization enabled, tonal quantization enabled, reserved, reserved
    pub flags: Vec4,
}
