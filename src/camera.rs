use crate::{
    black_hole::uniforms::BlackHoleUniform,
    journey::Journey,
    quantization::QuantizeUniform,
    settings::{LabSettings, RunOptions},
};
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{Exposure, Hdr, RenderTarget},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    image::ImageSampler,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::render_resource::{TextureFormat, TextureUsages},
    render::view::ColorGrading,
    window::{CursorGrabMode, CursorOptions},
};
#[derive(Component)]
pub struct FlyCamera;
#[derive(Component)]
pub struct PhysicalDisplay;
#[derive(Resource)]
pub struct PhysicalTarget(pub Handle<Image>);
#[derive(Resource)]
pub struct Navigation {
    pub speed: f32,
    pub notice: String,
}
pub fn preset(index: usize, s: &LabSettings) -> Transform {
    let (position, target) = match index {
        1 => (Vec3::new(0.0, 33.2659, 156.5036), Vec3::ZERO),
        2 => (Vec3::new(0.0, 0.35, 29.0), Vec3::ZERO),
        3 => (Vec3::new(0.0, 18.0, 8.0), Vec3::ZERO),
        4 => (Vec3::new(7.0, 2.1, 3.0), Vec3::ZERO),
        5 => (Vec3::new(0.0, 0.025, 34.0), Vec3::ZERO),
        7 => (Vec3::new(0.0, 0.42, 1.04), Vec3::ZERO),
        _ => {
            return Transform::from_xyz(6.3, 3.1, 27.0)
                .looking_at(s.center + Vec3::new(0.0, 0.0, 3.0), Vec3::Y);
        }
    };
    let mut position = position * s.radius;
    if position.length() < s.observer_minimum() {
        position = position.normalize() * s.observer_minimum() * 1.01;
    }
    Transform::from_translation(s.center + position).looking_at(s.center + target, Vec3::Y)
}
pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    s: Res<LabSettings>,
    o: Res<RunOptions>,
) {
    let mut image = Image::new_target_texture(
        s.internal.x,
        s.internal.y,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    image.sampler = ImageSampler::nearest();
    let target = images.add(image);
    commands.spawn((
        Name::new("physical HDR camera"),
        Camera3d {
            depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING)
                .into(),
            ..default()
        },
        Camera {
            order: -1,
            clear_color: Color::BLACK.into(),
            ..default()
        },
        RenderTarget::Image(target.clone().into()),
        Hdr,
        Msaa::Off,
        Tonemapping::AcesFitted,
        DebandDither::Disabled,
        Exposure { ev100: s.ev },
        ColorGrading::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            near: 0.02,
            far: 2000.0,
            ..default()
        }),
        o.initial_camera.unwrap_or_else(|| preset(o.preset, &s)),
        FlyCamera,
        BlackHoleUniform::default(),
        QuantizeUniform::default(),
    ));
    let display = commands
        .spawn((
            Name::new("presentation camera"),
            Camera2d,
            IsDefaultUiCamera,
            Camera {
                clear_color: Color::BLACK.into(),
                ..default()
            },
        ))
        .id();
    commands.spawn((
        PhysicalDisplay,
        ImageNode::new(target.clone()),
        Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            ..default()
        },
        UiTargetCamera(display),
    ));
    commands.insert_resource(PhysicalTarget(target));
    commands.insert_resource(Navigation {
        speed: o.speed,
        notice: "External static observer / T compares Schwarzschild and Kerr".into(),
    });
    commands.insert_resource(Journey::from_options(&o));
}
/// Keep presentation cells aligned to physical display pixels when the OS resizes a window.
pub fn presentation(
    s: Res<LabSettings>,
    windows: Query<&Window>,
    mut displays: Query<&mut Node, With<PhysicalDisplay>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let window_size = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    // Keep the framing identical when quantization is bypassed for comparison.
    let cells = s.grid.as_vec2();
    let fit = (window_size / cells).min_element();
    let scale = if fit >= 1.0 { fit.floor() } else { fit };
    let size = cells * scale;
    let dpi = window.scale_factor();
    let offset = ((window_size - size) * 0.5).floor();
    for mut node in &mut displays {
        node.width = px(size.x / dpi);
        node.height = px(size.y / dpi);
        node.left = px(offset.x / dpi);
        node.top = px(offset.y / dpi);
    }
}
pub fn fly(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut windows: Query<(&Window, &mut CursorOptions)>,
    mut cameras: Query<(&mut Transform, Option<&mut TemporalAntiAliasing>), With<FlyCamera>>,
    s: Res<LabSettings>,
    o: Res<RunOptions>,
    mut previous_benchmark_phase: Local<Option<usize>>,
    mut journey: ResMut<Journey>,
    mut nav: ResMut<Navigation>,
) {
    let Ok((window, mut cursor)) = windows.single_mut() else {
        return;
    };
    if o.freeze_input && !o.journey && !o.benchmark {
        for (mut t, _) in &mut cameras {
            let p = t.translation - s.center;
            if p.length() < s.observer_minimum() {
                t.translation = s.center + p.normalize_or(Vec3::Z) * s.observer_minimum();
            }
        }
        return;
    }
    if keys.just_pressed(KeyCode::Tab) || buttons.just_pressed(MouseButton::Right) {
        cursor.grab_mode = if cursor.grab_mode == CursorGrabMode::None {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = cursor.grab_mode == CursorGrabMode::None;
    }
    if keys.just_pressed(KeyCode::Escape) || !window.focused {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
    if window.focused && scroll.delta.y != 0.0 {
        nav.speed = (nav.speed * 2.0_f32.powf(scroll.delta.y * 0.25)).clamp(0.02, 2000.0);
    }
    let guide_cut = keys.just_pressed(KeyCode::Enter)
        || (keys.just_pressed(KeyCode::KeyJ)
            && !journey.active
            && (journey.interrupted || journey.progress() >= 1.0));
    if keys.just_pressed(KeyCode::Enter) {
        journey.elapsed = 0.0;
        journey.active = true;
        journey.interrupted = false;
    } else if keys.just_pressed(KeyCode::KeyJ) {
        if journey.interrupted || journey.progress() >= 1.0 {
            journey.elapsed = 0.0;
        }
        journey.active = !journey.active;
        journey.interrupted = false;
    }
    for (mut t, taa) in &mut cameras {
        let mut cut = guide_cut;
        for (key, index) in [
            (KeyCode::Digit1, 1),
            (KeyCode::Digit2, 2),
            (KeyCode::Digit3, 3),
            (KeyCode::Digit4, 4),
            (KeyCode::Digit5, 5),
            (KeyCode::Digit6, 6),
            (KeyCode::Digit7, 7),
        ] {
            if keys.just_pressed(key) {
                *t = preset(index, &s);
                cut = true;
                journey.active = false;
                journey.interrupted = true;
            }
        }
        let dt = time.delta_secs().min(0.05);
        let manual = window.focused
            && ([
                KeyCode::KeyW,
                KeyCode::KeyA,
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::Space,
                KeyCode::ControlLeft,
                KeyCode::KeyQ,
                KeyCode::KeyE,
            ]
            .iter()
            .any(|k| keys.pressed(*k))
                || (cursor.grab_mode != CursorGrabMode::None
                    && mouse.delta.length_squared() > 0.0));
        if manual {
            journey.active = false;
            journey.interrupted = true;
        }
        if journey.active {
            journey.elapsed = (journey.elapsed + time.delta_secs()).min(journey.duration);
            *t = crate::journey::pose(journey.progress(), &s);
            if journey.progress() >= 1.0 {
                journey.active = false;
                nav.notice = format!(
                    "Journey ended at {:.3} rs: static external observer cutoff. Free flight remains available.",
                    (t.translation - s.center).length() / s.radius
                );
            }
        }
        if cursor.grab_mode != CursorGrabMode::None && window.focused {
            t.rotation = t.rotation
                * Quat::from_rotation_y(-mouse.delta.x * 0.002)
                * Quat::from_rotation_x(-mouse.delta.y * 0.002);
        }
        let axis = |positive, negative| {
            f32::from(keys.pressed(positive)) - f32::from(keys.pressed(negative))
        };
        let motion = Vec3::new(
            axis(KeyCode::KeyD, KeyCode::KeyA),
            axis(KeyCode::Space, KeyCode::ControlLeft),
            axis(KeyCode::KeyS, KeyCode::KeyW),
        );
        let speed = if keys.pressed(KeyCode::ShiftLeft) {
            nav.speed * 4.0
        } else if keys.pressed(KeyCode::AltLeft) {
            nav.speed * 0.125
        } else {
            nav.speed
        };
        let rotation = t.rotation;
        if window.focused {
            t.translation += rotation * motion.normalize_or_zero() * speed * dt;
            t.rotation *= Quat::from_rotation_z(axis(KeyCode::KeyQ, KeyCode::KeyE) * dt);
        }
        if o.benchmark {
            let phase = (time.elapsed_secs() / 8.0) as usize;
            cut = *previous_benchmark_phase != Some(phase);
            *previous_benchmark_phase = Some(phase);
            *t = preset(phase % 7 + 1, &s);
            t.translation.x += (time.elapsed_secs() * 0.35).sin() * 1.2;
        }
        if cut {
            if let Some(mut taa) = taa {
                taa.reset = true;
            }
        }
        let relative = t.translation - s.center;
        let r = relative.length();
        if r < s.observer_minimum() {
            t.translation = s.center + relative.normalize_or(Vec3::Z) * s.observer_minimum();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn navigation_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<LabSettings>()
            .init_resource::<RunOptions>()
            .insert_resource(Journey {
                active: true,
                elapsed: 60.0,
                duration: 150.0,
                interrupted: false,
            })
            .insert_resource(Navigation {
                speed: 4.0,
                notice: String::new(),
            })
            .add_systems(Update, fly);
        app.world_mut().spawn((
            Window {
                focused: true,
                ..default()
            },
            CursorOptions::default(),
        ));
        let pose = crate::journey::pose(0.4, app.world().resource::<LabSettings>());
        app.world_mut().spawn((FlyCamera, pose));
        app
    }
    #[test]
    fn movement_takes_over_guidance_without_a_camera_cut() {
        let mut app = navigation_app();
        let before = crate::journey::pose(0.4, app.world().resource::<LabSettings>());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();
        let journey = app.world().resource::<Journey>();
        assert!(!journey.active && journey.interrupted);
        let mut query = app
            .world_mut()
            .query_filtered::<&Transform, With<FlyCamera>>();
        let after = query.single(app.world()).unwrap();
        // First TimePlugin tick has zero delta: input must still relinquish guidance.
        assert!(after.translation.distance(before.translation) < 0.0001);
        assert!(after.rotation.dot(before.rotation).abs() > 0.99999);
    }
    #[test]
    fn presets_change_only_camera_and_release_guidance() {
        let mut app = navigation_app();
        app.world_mut().resource_mut::<LabSettings>().disk_tilt = 42.0;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit3);
        app.update();
        assert!(!app.world().resource::<Journey>().active);
        assert_eq!(app.world().resource::<LabSettings>().disk_tilt, 42.0);
        let expected = preset(3, app.world().resource::<LabSettings>());
        let mut query = app
            .world_mut()
            .query_filtered::<&Transform, With<FlyCamera>>();
        assert_eq!(
            query.single(app.world()).unwrap().translation,
            expected.translation
        );
    }
}
pub fn uniforms(
    s: Res<LabSettings>,
    mut cameras: Query<
        (
            &Transform,
            &mut BlackHoleUniform,
            &mut QuantizeUniform,
            &mut Exposure,
            &mut ColorGrading,
        ),
        With<FlyCamera>,
    >,
) {
    for (t, mut u, mut q, mut exposure, mut grading) in &mut cameras {
        // Meter the same baseline in auto mode; apply manual stops after metering
        // so histogram adaptation cannot silently cancel the user's adjustment.
        exposure.ev100 = if s.auto_exposure {
            Exposure::EV100_BLENDER
        } else {
            s.ev
        };
        grading.global.exposure = if s.auto_exposure {
            Exposure::EV100_BLENDER - s.ev
        } else {
            0.0
        };
        let tan = (30.0_f32.to_radians()).tan();
        *u = BlackHoleUniform {
            camera: t.translation.extend(0.0),
            right: (t.rotation * Vec3::X).extend(tan * s.internal.x as f32 / s.internal.y as f32),
            up: (t.rotation * Vec3::Y).extend(tan),
            forward: (t.rotation * Vec3::NEG_Z).extend(0.0),
            geometry: Vec4::new(s.radius, s.inner, s.outer, s.max_distance),
            integration: Vec4::new(s.steps as f32, s.step_scale(), exposure.exposure(), 0.0),
            features: Vec4::new(
                s.lensing as u8 as f32,
                s.disk as u8 as f32,
                s.stars as u8 as f32,
                s.doppler as u8 as f32,
            ),
            relativity: Vec4::new(
                s.beaming as u8 as f32,
                s.redshift as u8 as f32,
                s.budget_debug as u8 as f32,
                0.0,
            ),
            disk: s.disk_normal().extend(s.emissivity),
            center: s.center.extend(0.0),
            sky: UVec4::new(s.seed, s.sky_mode, s.ray_classes as u32, 0),
            model: Vec4::new(
                s.spin,
                s.kerr as u8 as f32,
                s.temperature,
                s.error_debug as u8 as f32,
            ),
            axes_x: (s.disk_rotation() * Vec3::X).extend(0.0),
            axes_y: (s.disk_rotation() * Vec3::NEG_Z).extend(0.0),
            axes_z: s.disk_normal().extend(0.0),
        };
        *q = QuantizeUniform {
            parameters: Vec4::new(
                s.grid.x as f32,
                s.grid.y as f32,
                s.levels,
                if s.dither { 0.65 } else { 0.0 },
            ),
            flags: Vec4::new(
                s.quantize as u8 as f32,
                (s.quantize && s.tonal) as u8 as f32,
                0.0,
                0.0,
            ),
        };
    }
}
