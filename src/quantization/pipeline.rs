use super::QuantizeUniform;
use bevy::{
    core_pipeline::{Core3dSystems, FullscreenShader, schedule::Core3d, tonemapping::tonemapping},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        diagnostic::RecordDiagnostics,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, ViewQuery},
        view::ViewTarget,
    },
};
pub struct QuantizationPlugin;
impl Plugin for QuantizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<QuantizeUniform>::default(),
            UniformComponentPlugin::<QuantizeUniform>::default(),
        ));
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render.add_systems(RenderStartup, init).add_systems(
            Core3d,
            quantize
                .after(tonemapping)
                .in_set(Core3dSystems::PostProcess),
        );
    }
}
#[derive(Resource)]
struct Pipeline {
    layout: BindGroupLayoutDescriptor,
    reduce: CachedRenderPipelineId,
    expand: CachedRenderPipelineId,
}
fn init(
    mut commands: Commands,
    assets: Res<AssetServer>,
    fullscreen: Res<FullscreenShader>,
    cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "quantize layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer::<QuantizeUniform>(true),
            ),
        ),
    );
    let make = |entry: &'static str| {
        cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(format!("quantization {entry}").into()),
            layout: vec![layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: assets.load("shaders/quantize.wgsl"),
                entry_point: Some(entry.into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        })
    };
    let reduce = make("reduce");
    let expand = make("expand");
    commands.insert_resource(Pipeline {
        layout,
        reduce,
        expand,
    });
}
#[derive(Default)]
struct GridCache {
    size: UVec2,
    texture: Option<Texture>,
    view: Option<TextureView>,
    bindings: Vec<(TextureViewId, BindGroup)>,
    expand: Option<BindGroup>,
}
fn quantize(
    view: ViewQuery<(
        &ViewTarget,
        &QuantizeUniform,
        &DynamicUniformIndex<QuantizeUniform>,
    )>,
    pipeline: Option<Res<Pipeline>>,
    cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<QuantizeUniform>>,
    mut grid: Local<GridCache>,
    mut ctx: RenderContext,
) {
    let Some(p) = pipeline else { return };
    let (Some(reduce), Some(expand)) = (
        cache.get_render_pipeline(p.reduce),
        cache.get_render_pipeline(p.expand),
    ) else {
        return;
    };
    let (target, settings, index) = view.into_inner();
    if settings.flags.x < 0.5 && settings.flags.y < 0.5 {
        return;
    }
    let Some(uniform) = uniforms.uniforms().binding() else {
        return;
    };
    let size = if settings.flags.x > 0.5 {
        UVec2::new(settings.parameters.x as u32, settings.parameters.y as u32)
    } else {
        UVec2::new(
            target.main_texture().width(),
            target.main_texture().height(),
        )
    };
    if grid.size != size || grid.view.is_none() {
        let texture = ctx.render_device().create_texture(&TextureDescriptor {
            label: Some("intentional presentation grid"),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&default());
        grid.expand = Some(ctx.render_device().create_bind_group(
            "grid expand",
            &cache.get_bind_group_layout(&p.layout),
            &BindGroupEntries::sequential((&view, uniform.clone())),
        ));
        grid.size = size;
        grid.texture = Some(texture);
        grid.view = Some(view);
        grid.bindings.clear();
    }
    let write = target.post_process_write();
    let pos = grid
        .bindings
        .iter()
        .position(|e| e.0 == write.source.id())
        .unwrap_or_else(|| {
            if grid.bindings.len() >= 4 {
                grid.bindings.clear();
            }
            let bind = ctx.render_device().create_bind_group(
                "grid reduce",
                &cache.get_bind_group_layout(&p.layout),
                &BindGroupEntries::sequential((write.source, uniform)),
            );
            grid.bindings.push((write.source.id(), bind));
            grid.bindings.len() - 1
        });
    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();
    let span = diagnostics.time_span(ctx.command_encoder(), "quantization");
    for (gpu, destination, bind) in [
        (reduce, grid.view.as_ref().unwrap(), &grid.bindings[pos].1),
        (expand, write.destination, grid.expand.as_ref().unwrap()),
    ] {
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("quantization"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                ..default()
            });
        pass.set_pipeline(gpu);
        pass.set_bind_group(0, bind, &[index.index()]);
        pass.draw(0..3, 0..1);
    }
    span.end(ctx.command_encoder());
}
