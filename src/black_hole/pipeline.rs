use super::uniforms::BlackHoleUniform;
use bevy::{
    core_pipeline::{Core3dSystems, FullscreenShader, schedule::Core3d},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        diagnostic::RecordDiagnostics,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{texture_2d, texture_depth_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, ViewQuery},
        view::{ViewDepthTexture, ViewTarget},
    },
};
pub struct BlackHolePlugin;
impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<BlackHoleUniform>::default(),
            UniformComponentPlugin::<BlackHoleUniform>::default(),
        ));
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render.add_systems(RenderStartup, init).add_systems(
            Core3d,
            render_background
                .after(Core3dSystems::EarlyPostProcess)
                .before(Core3dSystems::PostProcess),
        );
    }
}
#[derive(Resource)]
struct Pipeline {
    layout: BindGroupLayoutDescriptor,
    id: CachedRenderPipelineId,
}
fn init(
    mut commands: Commands,
    assets: Res<AssetServer>,
    fullscreen: Res<FullscreenShader>,
    cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "black hole layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                texture_depth_2d(),
                uniform_buffer::<BlackHoleUniform>(true),
            ),
        ),
    );
    let id = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("Relativistic HDR background".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: assets.load("shaders/black_hole.wgsl"),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });
    commands.insert_resource(Pipeline { layout, id });
}
#[derive(Default)]
struct BindingCache {
    entries: Vec<(TextureViewId, TextureViewId, BindGroup)>,
}
fn render_background(
    view: ViewQuery<(
        &ViewTarget,
        &ViewDepthTexture,
        &DynamicUniformIndex<BlackHoleUniform>,
    )>,
    pipeline: Option<Res<Pipeline>>,
    cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<BlackHoleUniform>>,
    mut bindings: Local<BindingCache>,
    mut ctx: RenderContext,
) {
    let Some(p) = pipeline else { return };
    let Some(gpu) = cache.get_render_pipeline(p.id) else {
        return;
    };
    let Some(uniform) = uniforms.uniforms().binding() else {
        return;
    };
    let (target, depth, index) = view.into_inner();
    let write = target.post_process_write();
    let ids = (write.source.id(), depth.view().id());
    let pos = bindings
        .entries
        .iter()
        .position(|e| e.0 == ids.0 && e.1 == ids.1);
    let pos = pos.unwrap_or_else(|| {
        if bindings.entries.len() >= 4 {
            bindings.entries.clear();
        }
        let bind = ctx.render_device().create_bind_group(
            "black hole bind",
            &cache.get_bind_group_layout(&p.layout),
            &BindGroupEntries::sequential((write.source, depth.view(), uniform)),
        );
        bindings.entries.push((ids.0, ids.1, bind));
        bindings.entries.len() - 1
    });
    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();
    let span = diagnostics.time_span(ctx.command_encoder(), "black_hole");
    {
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("black_hole"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: write.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                ..default()
            });
        pass.set_pipeline(gpu);
        pass.set_bind_group(0, &bindings.entries[pos].2, &[index.index()]);
        pass.draw(0..3, 0..1);
    }
    span.end(ctx.command_encoder());
}
