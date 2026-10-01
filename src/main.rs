mod black_hole;
mod camera;
mod configuration;
mod debug_ui;
mod journey;
mod materials;
#[cfg(test)]
mod physics_reference;
mod profiling;
mod quantization;
mod scene;
mod settings;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    post_process::auto_exposure::AutoExposurePlugin,
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
    window::{MonitorSelection, PresentMode, WindowMode, WindowResolution},
};
fn main() {
    let (mut settings, mut options) = settings::parse_options();
    if let Some(path) = &options.config {
        match configuration::load(path) {
            Ok((s, camera, speed)) => {
                settings = s;
                options.initial_camera = Some(camera);
                options.speed = speed;
            }
            Err(e) => {
                eprintln!("Cannot load {path}: {e}");
                std::process::exit(1);
            }
        }
    }
    let local_assets = std::env::current_dir()
        .expect("working directory")
        .join("assets");
    let packaged_assets = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("assets")))
        .filter(|p| p.is_dir());
    let assets = if let Some(path) = packaged_assets {
        path
    } else if local_assets.is_dir() {
        local_assets
    } else {
        eprintln!(
            "Missing assets/. Extract the complete download, or run Cargo from the repository root."
        );
        std::process::exit(1);
    };
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Journey to a Black Hole • real-time relativistic rendering lab"
                            .into(),
                        resolution: WindowResolution::new(options.output.x, options.output.y),
                        mode: if options.fullscreen {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                        } else {
                            WindowMode::Windowed
                        },
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            RenderDiagnosticsPlugin,
            profiling::GpuFrameTimingPlugin,
            AutoExposurePlugin,
            black_hole::BlackHolePlugin,
            quantization::QuantizationPlugin,
        ))
        .insert_resource(settings)
        .insert_resource(options)
        .init_resource::<debug_ui::CpuClock>()
        .add_systems(
            Startup,
            (
                camera::setup,
                scene::setup,
                debug_ui::setup,
                black_hole::debug_rays::setup,
            )
                .chain(),
        )
        .add_systems(First, debug_ui::begin_cpu)
        .add_systems(
            Update,
            (
                debug_ui::controls,
                debug_ui::validate,
                configuration::controls,
                camera::fly,
                debug_ui::apply,
                camera::uniforms,
                camera::presentation,
                scene::sync,
                black_hole::debug_rays::update,
                debug_ui::update,
            )
                .chain(),
        )
        .add_systems(Last, debug_ui::end_cpu)
        .run();
}
