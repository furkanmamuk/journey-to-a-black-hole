use crate::{
    camera::{FlyCamera, Navigation, PhysicalTarget},
    journey::Journey,
    settings::{LabSettings, RunOptions},
};
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    app::AppExit,
    core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    post_process::{auto_exposure::AutoExposure, bloom::Bloom},
    prelude::*,
    render::render_resource::Extent3d,
    render::{
        camera::{MipBias, TemporalJitter},
        view::screenshot::{Screenshot, save_to_disk},
    },
};
use std::time::Instant;
#[derive(Resource)]
pub struct CpuClock {
    start: Instant,
    pub millis: f64,
}
impl Default for CpuClock {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            millis: 0.0,
        }
    }
}
pub fn begin_cpu(mut clock: ResMut<CpuClock>) {
    clock.start = Instant::now();
}
pub fn end_cpu(mut clock: ResMut<CpuClock>) {
    clock.millis = clock.start.elapsed().as_secs_f64() * 1000.0;
}
#[derive(Component)]
pub struct Hud;
#[derive(Resource, Default)]
pub struct HudState {
    selected: usize,
    last_log: f32,
    last_ui: f32,
    captured: usize,
    pub visible: bool,
    detailed: bool,
}
pub fn setup(mut commands: Commands, options: Res<RunOptions>) {
    commands.insert_resource(HudState {
        visible: !options.hide_hud,
        ..default()
    });
    commands.spawn((
        Hud,
        GlobalZIndex(100),
        if options.hide_hud {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
        Text::new("Compiling render pipelines…"),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgb(0.77, 0.84, 0.87)),
        Node {
            position_type: PositionType::Absolute,
            left: px(18),
            top: px(16),
            max_width: px(710),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
    ));
}
/// GPU smoke sequence: exercise runtime component removal, cached resources and resizing.
pub fn validate(
    time: Res<Time>,
    options: Res<RunOptions>,
    mut settings: ResMut<LabSettings>,
    mut stage: Local<usize>,
    mut inspector: ResMut<crate::black_hole::debug_rays::RayInspector>,
    mut cameras: Query<&mut Transform, With<FlyCamera>>,
    mut journey: ResMut<Journey>,
) {
    if !options.validate {
        return;
    }
    let next = (time.elapsed_secs() / 2.0) as usize;
    if next == *stage {
        return;
    }
    *stage = next;
    match next {
        1 => settings.lensing = false,
        2 => settings.lensing = true,
        3 => settings.disk = false,
        4 => {
            settings.disk = true;
            settings.stars = false;
        }
        5 => {
            settings.stars = true;
            settings.craft = false;
        }
        6 => {
            settings.craft = true;
            settings.bloom = false;
        }
        7 => {
            settings.bloom = true;
            settings.taa = true;
        }
        8 => {
            settings.taa = false;
            settings.quantize = false;
        }
        9 => {
            settings.quantize = true;
            settings.dither = false;
        }
        10 => {
            settings.dither = true;
            settings.auto_exposure = true;
        }
        11 => {
            settings.auto_exposure = false;
            settings.stellar_light = false;
        }
        12 => {
            settings.stellar_light = true;
            settings.internal = UVec2::new(640, 360);
        }
        13 => settings.grid = UVec2::new(320, 180),
        14 => {
            settings.internal = UVec2::new(1280, 720);
            settings.grid = UVec2::new(640, 360);
            settings.taa = true;
        }
        15 => {
            settings.internal = UVec2::new(960, 540);
            settings.grid = UVec2::new(480, 270);
            settings.taa = false;
        }
        16 => {
            settings.set_quality(2);
            settings.radius = 1.3;
            settings.inner = 3.9;
        }
        17 => *settings = LabSettings::default(),
        18 => settings.disk_tilt = 42.0,
        19 => {
            settings.disk_azimuth = 73.0;
            settings.emissivity = 2.0;
        }
        20 => settings.seed = 0,
        21 => settings.sky_mode = 1,
        22 => settings.sky_mode = 2,
        23 => settings.ray_classes = true,
        24 => {
            settings.ray_classes = false;
            inspector.enabled = true;
        }
        25 => {
            inspector.side = 5;
            inspector.selected = 12;
            inspector.view = 1;
        }
        26 => {
            inspector.view = 2;
            settings.disk_tilt = -57.0;
            if let Ok(mut t) = cameras.single_mut() {
                *t = crate::camera::preset(7, &settings);
            }
        }
        27 => {
            settings.lensing = false;
            if let Ok(mut t) = cameras.single_mut() {
                *t = crate::camera::preset(4, &settings);
            }
        }
        28 => {
            settings.lensing = true;
            settings.seed = u32::MAX;
        }
        29 => {
            inspector.enabled = false;
            *settings = LabSettings::default();
            journey.elapsed = 0.60 * journey.duration;
            journey.active = true;
        }
        30 => {
            journey.active = false;
            journey.interrupted = true;
            if let Ok(mut t) = cameras.single_mut() {
                *t = crate::camera::preset(6, &settings);
            }
        }
        31 => {
            inspector.enabled = true;
            inspector.side = 3;
            inspector.selected = 4;
        }
        32 => {
            inspector.enabled = false;
            settings.kerr = true;
            settings.spin = 0.8;
            settings.isco_lock = true;
        }
        33 => settings.spin = -0.8,
        34 => {
            settings.spin = 0.0;
            settings.error_debug = true;
        }
        35 => {
            settings.spin = 0.98;
            settings.error_debug = false;
            settings.budget_debug = true;
        }
        36 => {
            settings.disk_tilt = 67.0;
            settings.disk_azimuth = 113.0;
        }
        37 => {
            settings.internal = UVec2::new(1333, 751);
            settings.grid = UVec2::new(167, 94);
            settings.taa = true;
        }
        38 => {
            settings.quantize = false;
            settings.auto_exposure = true;
        }
        39 => {
            settings.quantize = true;
            settings.internal = UVec2::new(960, 540);
            settings.grid = UVec2::new(480, 270);
            inspector.enabled = true;
        }
        40 => {
            settings.spin = -0.98;
            inspector.side = 5;
            inspector.selected = 12;
        }
        41 => {
            inspector.enabled = false;
            settings.budget_debug = false;
            settings.auto_exposure = false;
            settings.taa = false;
        }
        _ => {}
    }
    settings.sanitize();
    info!(
        "LAB_VALIDATE stage={next} internal={:?} grid={:?} taa={} auto={}",
        settings.internal, settings.grid, settings.taa, settings.auto_exposure
    );
}
pub fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut s: ResMut<LabSettings>,
    mut state: ResMut<HudState>,
    mut hud: Query<&mut Visibility, With<Hud>>,
    mut cameras: Query<&mut Transform, With<FlyCamera>>,
    mut journey: ResMut<Journey>,
) {
    if !keys.get_just_pressed().any(|key| {
        matches!(
            key,
            KeyCode::F1
                | KeyCode::F2
                | KeyCode::F3
                | KeyCode::F4
                | KeyCode::F5
                | KeyCode::F6
                | KeyCode::F7
                | KeyCode::F8
                | KeyCode::F9
                | KeyCode::F10
                | KeyCode::F11
                | KeyCode::KeyZ
                | KeyCode::KeyX
                | KeyCode::KeyC
                | KeyCode::KeyV
                | KeyCode::KeyH
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::Equal
                | KeyCode::Minus
                | KeyCode::KeyP
                | KeyCode::KeyG
                | KeyCode::KeyB
                | KeyCode::KeyT
                | KeyCode::KeyU
                | KeyCode::KeyM
        )
    }) {
        return;
    }
    let settings = &mut *s;
    for (key, value) in [
        (KeyCode::F1, &mut settings.lensing),
        (KeyCode::F2, &mut settings.disk),
        (KeyCode::F3, &mut settings.stars),
        (KeyCode::F4, &mut settings.craft),
        (KeyCode::F5, &mut settings.bloom),
        (KeyCode::F6, &mut settings.taa),
        (KeyCode::F7, &mut settings.quantize),
        (KeyCode::F8, &mut settings.dither),
        (KeyCode::F9, &mut settings.auto_exposure),
        (KeyCode::F10, &mut settings.stellar_light),
        (KeyCode::KeyZ, &mut settings.doppler),
        (KeyCode::KeyX, &mut settings.beaming),
        (KeyCode::KeyC, &mut settings.redshift),
        (KeyCode::F11, &mut settings.budget_debug),
        (KeyCode::KeyV, &mut settings.tonal),
        (KeyCode::KeyB, &mut settings.ray_classes),
        (KeyCode::KeyT, &mut settings.kerr),
        (KeyCode::KeyU, &mut settings.isco_lock),
        (KeyCode::KeyM, &mut settings.error_debug),
    ] {
        if keys.just_pressed(key) {
            *value = !*value;
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        s.sky_mode = (s.sky_mode + 1) % 3;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        state.detailed = !state.detailed;
    }
    if keys.just_pressed(KeyCode::KeyH) {
        state.visible = !state.visible;
        for mut v in &mut hud {
            *v = if state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        state.selected = (state.selected + 1) % 17;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        state.selected = (state.selected + 16) % 17;
    }
    let delta = i32::from(keys.just_pressed(KeyCode::ArrowRight))
        - i32::from(keys.just_pressed(KeyCode::ArrowLeft));
    if delta != 0 {
        let d = delta as f32;
        match state.selected {
            0 => {
                let next = (s.quality as i32 + delta).rem_euclid(3) as usize;
                s.set_quality(next);
            }
            1 => s.steps = (s.steps as i32 + delta * 32).clamp(32, 768) as u32,
            2 => {
                let sizes = [
                    UVec2::new(640, 360),
                    UVec2::new(960, 540),
                    UVec2::new(1280, 720),
                    UVec2::new(1920, 1080),
                ];
                let i = sizes.iter().position(|v| *v == s.internal).unwrap_or(1);
                s.internal = sizes[(i as i32 + delta).rem_euclid(4) as usize];
            }
            3 => {
                let sizes = [
                    UVec2::new(320, 180),
                    UVec2::new(480, 270),
                    UVec2::new(640, 360),
                    UVec2::new(960, 540),
                ];
                let i = sizes.iter().position(|v| *v == s.grid).unwrap_or(1);
                s.grid = sizes[(i as i32 + delta).rem_euclid(4) as usize];
            }
            4 => s.ev += d * 0.25,
            5 => s.radius *= 2.0_f32.powf(d * 0.1),
            6 => {
                s.isco_lock = false;
                s.inner += d * 0.25 * s.radius;
            }
            7 => s.outer += d * 0.5 * s.radius,
            8 => s.max_distance += d * 50.0,
            9 => s.levels += d * 8.0,
            10 => {
                if let Ok(mut t) = cameras.single_mut() {
                    let relative = t.translation - s.center;
                    t.translation = s.center + relative * 2.0_f32.powf(d * 0.1);
                }
                journey.active = false;
                journey.interrupted = true;
            }
            11 => s.disk_tilt += d * 5.0,
            12 => s.disk_azimuth += d * 5.0,
            13 => s.emissivity *= 2.0_f32.powf(d * 0.25),
            14 => s.seed = s.seed.wrapping_add_signed(delta),
            15 => s.spin += d * 0.05,
            16 => s.temperature *= 2.0_f32.powf(d * 0.1),
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::Equal) {
        s.ev -= 0.25;
    }
    if keys.just_pressed(KeyCode::Minus) {
        s.ev += 0.25;
    }
    s.sanitize();
}
pub fn apply(
    mut commands: Commands,
    s: Res<LabSettings>,
    target: Res<PhysicalTarget>,
    mut images: ResMut<Assets<Image>>,
    camera: Query<Entity, With<FlyCamera>>,
    mut last: Local<Option<LabSettings>>,
) {
    let Ok(entity) = camera.single() else { return };
    let old = last.as_ref();
    if old.is_none_or(|v| v.bloom != s.bloom) {
        if s.bloom {
            commands.entity(entity).insert(Bloom {
                intensity: 0.035,
                ..default()
            });
        } else {
            commands.entity(entity).remove::<Bloom>();
        }
    }
    if old.is_none_or(|v| v.taa != s.taa) {
        if s.taa {
            commands
                .entity(entity)
                .insert(TemporalAntiAliasing::default());
        } else {
            commands.entity(entity).remove::<(
                TemporalAntiAliasing,
                TemporalJitter,
                MipBias,
                DepthPrepass,
                MotionVectorPrepass,
            )>();
        }
    }
    if old.is_none_or(|v| v.auto_exposure != s.auto_exposure) {
        if s.auto_exposure {
            commands.entity(entity).insert(AutoExposure {
                range: -12.0..=16.0,
                filter: 0.05..=0.99,
                speed_brighten: 1.2,
                speed_darken: 3.0,
                ..default()
            });
        } else {
            commands.entity(entity).remove::<AutoExposure>();
        }
    }
    if old.is_none_or(|v| v.internal != s.internal) {
        if let Some(mut image) = images.get_mut(&target.0) {
            image.resize(Extent3d {
                width: s.internal.x,
                height: s.internal.y,
                depth_or_array_layers: 1,
            });
        }
        if s.taa {
            commands
                .entity(entity)
                .insert(TemporalAntiAliasing { reset: true });
        }
    }
    *last = Some(s.clone());
}
fn timing(diagnostics: &DiagnosticsStore, term: &str) -> Option<f64> {
    diagnostics
        .iter()
        .filter(|d| d.path().as_str().contains(term))
        .filter(|d| {
            d.measurement()
                .is_some_and(|m| m.time.elapsed().as_secs_f32() < 0.5)
        })
        .filter_map(|d| d.average())
        .reduce(f64::max)
}
fn format_timing(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.2} ms"))
        .unwrap_or_else(|| "unavailable".into())
}
pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    s: Res<LabSettings>,
    o: Res<RunOptions>,
    diagnostics: Res<DiagnosticsStore>,
    cpu: Res<CpuClock>,
    mut state: ResMut<HudState>,
    mut text: Query<&mut Text, With<Hud>>,
    mut exit: MessageWriter<AppExit>,
    windows: Query<&Window>,
    camera: Query<&Transform, With<FlyCamera>>,
    journey: Res<Journey>,
    nav: Res<Navigation>,
) {
    let now = time.elapsed_secs();
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.average())
        .unwrap_or(0.0);
    let frame = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.average())
        .unwrap_or(0.0);
    let bh = timing(&diagnostics, "black_hole/elapsed_gpu");
    let quant = if s.quantize {
        timing(&diagnostics, "quantization/elapsed_gpu")
    } else {
        Some(0.0)
    };
    let render = timing(&diagnostics, "lab_frame/elapsed_gpu");
    let transform = camera.single().copied().unwrap_or_default();
    let r = (transform.translation - s.center).length() / s.radius;
    if now - state.last_ui > 0.2 {
        state.last_ui = now;
        let options = [
            format!(
                "quality {} ({})",
                s.quality,
                ["cheap", "balanced", "high"][s.quality]
            ),
            format!("steps {}", s.steps),
            format!("internal {}x{}", s.internal.x, s.internal.y),
            format!("grid {}x{}", s.grid.x, s.grid.y),
            format!("exposure EV100 {:.2}", s.ev),
            format!("Schwarzschild radius {:.3}", s.radius),
            format!(
                "disk inner {:.2} rs (ISCO lock {})",
                s.inner / s.radius,
                s.isco_lock
            ),
            format!("disk outer {:.2} rs", s.outer / s.radius),
            format!("max affine distance {:.0}", s.max_distance),
            format!("tonal levels {:.0}", s.levels),
            format!("camera distance {r:.3} rs"),
            format!("disk tilt {:.0} degrees", s.disk_tilt),
            format!("disk azimuth {:.0} degrees", s.disk_azimuth),
            format!("disk emissivity {:.3} x", s.emissivity),
            format!("star seed {}", s.seed),
            format!("spin a/M {:.2}", s.spin),
            format!("temperature scale {:.0} K", s.temperature),
        ];
        let selected = &options[state.selected];
        let status = |v| if v { "ON" } else { "off" };
        for mut text in &mut text {
            let output = if s.quantize {
                "QUANTIZED OUTPUT"
            } else {
                "PHYSICAL OUTPUT"
            };
            let mode = if journey.active {
                "JOURNEY"
            } else {
                "FREE EXPLORATION"
            };
            let sky = ["stars", "celestial grid", "stars + grid"][s.sky_mode as usize];
            let mut content = format!(
                "JOURNEY TO A BLACK HOLE\n{mode} | camera {r:.3} rs | speed {:.2} units/s\n{output}\n{fps:.0} FPS | {frame:.2} ms frame | GPU {} | CPU {:.2} ms\nLens {} / Disk {} / Sky {sky} / Probe {}\nJ journey/pause | Enter restart | 1-7 views | F7 compare\nP parameters/help | I rays | G sky | B classes | H clean\n{}",
                nav.speed,
                format_timing(render),
                cpu.millis,
                status(s.lensing),
                status(s.disk),
                status(s.craft),
                nav.notice
            );
            content.push_str(&format!(
                "\n{} | spin {:.2} | disk {:.2}..{:.2} rs",
                if s.kerr {
                    "KERR-SCHILD"
                } else {
                    "SCHWARZSCHILD"
                },
                if s.kerr { s.spin } else { 0.0 },
                s.inner / s.radius,
                s.outer / s.radius
            ));
            if journey.active || journey.elapsed > 0.0 {
                content.push_str(&format!(
                    "\n{} | {:.0}% | static external cutoff",
                    journey.stage(),
                    journey.progress() * 100.0
                ));
            }
            if state.detailed {
                content.push_str(&format!("\nT Kerr/Schwarzschild | U ISCO lock | M constraint error\n\nUp/Down select; Left/Right adjust:\n> {selected}\nrs {:.3} | disk {:.2}..{:.2} rs | tilt {:.0} / azimuth {:.0} deg\n{} steps ({}) | emissivity {:.2} | seed {}\nHDR {}x{} -> grid {}x{} | EV100 {:.2}\nGPU hole {} / grid {}\nF1 lens  F2 disk  F3 sky  F4 probe\nF5 bloom {}  F6 TAA/PBR {}  F7 quant  F8 dither {}\nF9 auto exposure {}  F10 stellar {}  F11 budget {}\nZ Doppler {}  X beaming {}  C redshift {}  V tonal\nF12 save scene | L load scene | +/- exposure\nWASD / Space / Ctrl | Q/E roll | Shift / Alt\nWheel speed | Tab/RMB mouse | Esc release\nCoordinates ({:.3}, {:.3}, {:.3})",s.radius,s.inner/s.radius,s.outer/s.radius,s.disk_tilt,s.disk_azimuth,s.steps,
                    ["cheap","balanced","high"][s.quality],s.emissivity,s.seed,s.internal.x,s.internal.y,s.grid.x,s.grid.y,s.ev,
                    format_timing(bh),format_timing(quant),status(s.bloom),status(s.taa),status(s.dither),status(s.auto_exposure),status(s.stellar_light),status(s.budget_debug),
                    status(s.doppler),status(s.beaming),status(s.redshift),transform.translation.x,transform.translation.y,transform.translation.z));
            }
            if s.error_debug {
                content.push_str(
                    "\nGPU constraint: green <1e-4 / amber <1e-3 / red >=1e-3 / magenta budget",
                );
            }
            if s.ray_classes {
                content.push_str(
                    "\nGPU classes: dark capture / green escape / amber disk / magenta budget",
                );
            }
            **text = content;
        }
    }
    if now - state.last_log > 2.0 {
        state.last_log = now;
        let output = windows
            .single()
            .map(|w| UVec2::new(w.physical_width(), w.physical_height()))
            .unwrap_or_default();
        info!(
            "LAB_METRICS t={now:.1} fps={fps:.1} frame_ms={frame:.3} main_cpu_ms={:.3} bh_gpu_ms={} quant_gpu_ms={} render_gpu_ms={} quality={} steps={} internal={}x{} grid={}x{} output={}x{} camera_rs={r:.4} seed={} tilt={} journey={:.3} model={} spin={:.3}",
            cpu.millis,
            format_timing(bh),
            format_timing(quant),
            format_timing(render),
            s.quality,
            s.steps,
            s.internal.x,
            s.internal.y,
            s.grid.x,
            s.grid.y,
            output.x,
            output.y,
            s.seed,
            s.disk_tilt,
            journey.progress(),
            if s.kerr { "kerr" } else { "schwarzschild" },
            s.spin
        );
        if now < 4.1 {
            for d in diagnostics.iter() {
                info!("LAB_DIAGNOSTIC {} {:?}", d.path().as_str(), d.average());
            }
        }
    }
    if o.capture
        && state.captured < o.capture_count
        && now > o.capture_at + state.captured as f32 * o.capture_interval
    {
        state.captured += 1;
        std::fs::create_dir_all("captures").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!(
                "captures/{}-preset-{}-q{}-steps{}-{}-{}-{}.png",
                o.capture_label,
                o.preset,
                s.quality,
                s.steps,
                if s.quantize { "quant" } else { "continuous" },
                if s.budget_debug { "budget" } else { "render" },
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )));
    }
    if o.seconds.is_some_and(|seconds| now > seconds) {
        exit.write(AppExit::Success);
    }
}
