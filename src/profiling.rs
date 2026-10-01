//! Timestamp the queued GPU frame, including shadow rendering outside camera passes.
use bevy::{
    prelude::*,
    render::{
        RenderApp,
        diagnostic::{DiagnosticsRecorder, RecordDiagnostics, resolve_encoder},
        render_resource::CommandEncoderDescriptor,
        renderer::{PendingCommandBuffers, RenderDevice, RenderGraph, RenderGraphSystems},
    },
};

pub struct GpuFrameTimingPlugin;
impl Plugin for GpuFrameTimingPlugin {
    fn build(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render.add_systems(
            RenderGraph,
            (
                begin_frame
                    .after(RenderGraphSystems::Begin)
                    .before(RenderGraphSystems::Render),
                end_frame
                    .after(RenderGraphSystems::Render)
                    .before(resolve_encoder)
                    .before(RenderGraphSystems::Submit),
            ),
        );
    }
}

// Exclusive systems keep the recorder's begin/end thread-local span on the same
// schedule thread. Two tiny ordered command buffers bracket all rendering; no
// CPU wait, device poll, mapped-buffer access or hot-path fence is introduced.
fn begin_frame(world: &mut World) {
    let Some(recorder) = world.get_resource::<DiagnosticsRecorder>() else {
        return;
    };
    let mut encoder =
        world
            .resource::<RenderDevice>()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("lab GPU frame begin"),
            });
    recorder.begin_time_span(&mut encoder, "lab_frame".into());
    world
        .resource_mut::<PendingCommandBuffers>()
        // Finished buffers preserve order. Deferred encoders are appended after
        // all finished render buffers by PendingCommandBuffers::take().
        .push([encoder.finish()]);
}

fn end_frame(world: &mut World) {
    let Some(recorder) = world.get_resource::<DiagnosticsRecorder>() else {
        return;
    };
    let mut encoder =
        world
            .resource::<RenderDevice>()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("lab GPU frame end"),
            });
    recorder.end_time_span(&mut encoder);
    world
        .resource_mut::<PendingCommandBuffers>()
        .push([encoder.finish()]);
}
