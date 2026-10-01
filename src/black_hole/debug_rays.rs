//! Separate 3D inspector, deliberately isolated from the astronomical render.
//! Geometry uses radial log coordinates to show both horizon and distant observer.
use super::integration::{self, Outcome};
use crate::{
    camera::FlyCamera,
    debug_ui::HudState,
    settings::{LabSettings, RunOptions},
};
use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, visibility::RenderLayers},
    core_pipeline::tonemapping::Tonemapping,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{PrimitiveTopology, TextureFormat},
};

#[derive(Resource)]
pub struct RayInspector {
    pub enabled: bool,
    pub side: usize,
    pub selected: usize,
    pub view: usize,
    pub extent: f32,
    pub summary: String,
    last_update: f32,
}
#[derive(Component)]
pub struct InspectorRoot;
#[derive(Component)]
pub struct InspectorCamera;
#[derive(Component)]
pub(crate) struct InspectorLabel;
#[derive(Resource)]
pub(crate) struct InspectorMesh(Handle<Mesh>);

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    o: Res<RunOptions>,
) {
    commands.insert_resource(RayInspector {
        enabled: o.inspector,
        side: 3,
        selected: 4,
        view: 0,
        extent: 0.6,
        summary: String::new(),
        last_update: -1.0,
    });
    let mut image = Image::new_target_texture(
        640,
        640,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    image.sampler = ImageSampler::linear();
    let image = images.add(image);
    commands.spawn((
        InspectorCamera,
        Camera3d::default(),
        Camera {
            order: -2,
            is_active: o.inspector,
            clear_color: Color::srgb(0.007, 0.012, 0.02).into(),
            ..default()
        },
        RenderTarget::Image(image.clone().into()),
        RenderLayers::layer(1),
        Msaa::Off,
        Tonemapping::None,
        Transform::from_xyz(7.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let mesh = meshes.add(line_mesh(
        vec![[0.0, 0.0, 0.0], [0.0, 0.01, 0.0]],
        vec![[1.0; 4]; 2],
    ));
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        RenderLayers::layer(1),
        Transform::default(),
    ));
    commands.insert_resource(InspectorMesh(mesh));
    commands
        .spawn((
            InspectorRoot,
            GlobalZIndex(110),
            Node {
                position_type: PositionType::Absolute,
                right: px(20),
                top: px(20),
                width: px(440),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.025, 0.04, 0.94)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(image),
                Node {
                    width: percent(100),
                    aspect_ratio: Some(1.0),
                    ..default()
                },
            ));
            parent.spawn((
                InspectorLabel,
                Text::new("Ray inspector"),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.9, 1.0)),
            ));
        });
}
fn line_mesh(positions: Vec<[f32; 3]>, colors: Vec<[f32; 4]>) -> Mesh {
    Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
}
fn compressed(p: Vec3, s: &LabSettings) -> Vec3 {
    let q = (p - s.center) / s.radius;
    let r = q.length();
    q.normalize_or_zero() * r.ln_1p()
}
struct Lines {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
}
impl Lines {
    fn line(&mut self, a: Vec3, b: Vec3, c: [f32; 4], s: &LabSettings) {
        self.positions
            .extend([compressed(a, s).to_array(), compressed(b, s).to_array()]);
        self.colors.extend([c, c]);
    }
    fn circle(&mut self, r: f32, rotation: Quat, c: [f32; 4], s: &LabSettings) {
        for i in 0..128 {
            let a = i as f32 * std::f32::consts::TAU / 128.0;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 128.0;
            self.line(
                s.center + rotation * Vec3::new(a.cos() * r, 0.0, a.sin() * r),
                s.center + rotation * Vec3::new(b.cos() * r, 0.0, b.sin() * r),
                c,
                s,
            );
        }
    }
    fn cross(&mut self, p: Vec3, size: f32, color: [f32; 4], s: &LabSettings) {
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            self.line(p - axis * size, p + axis * size, color, s);
        }
    }
}
pub fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    s: Res<LabSettings>,
    hud: Res<HudState>,
    mut inspector: ResMut<RayInspector>,
    fly: Query<&Transform, With<FlyCamera>>,
    mut roots: Query<&mut Visibility, With<InspectorRoot>>,
    mut cameras: Query<(&mut Camera, &mut Transform), (With<InspectorCamera>, Without<FlyCamera>)>,
    mesh: Res<InspectorMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut labels: Query<&mut Text, With<InspectorLabel>>,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        inspector.enabled = !inspector.enabled;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        inspector.side = if inspector.side == 3 { 5 } else { 3 };
        inspector.selected = inspector.side * inspector.side / 2;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        inspector.view = (inspector.view + 1) % 3;
    }
    if keys.just_pressed(KeyCode::Period) {
        inspector.selected = (inspector.selected + 1) % (inspector.side * inspector.side);
    }
    if keys.just_pressed(KeyCode::Comma) {
        inspector.selected = (inspector.selected + inspector.side * inspector.side - 1)
            % (inspector.side * inspector.side);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        inspector.extent = (inspector.extent * 0.7).max(0.001);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        inspector.extent = (inspector.extent / 0.7).min(0.95);
    }
    let visible = inspector.enabled && hud.visible;
    for mut root in &mut roots {
        *root = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut camera, mut transform) in &mut cameras {
        camera.is_active = visible;
        let a = time.elapsed_secs() * 0.07;
        let position = match inspector.view {
            1 => Vec3::new(0.01, 11.5, 0.0),
            2 => Vec3::new(11.5, 0.4, 0.0),
            _ => Vec3::new(a.sin() * 9.0, 6.0, a.cos() * 9.0),
        };
        *transform = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y);
    }
    if !visible || time.elapsed_secs() - inspector.last_update < 0.1 {
        return;
    }
    inspector.last_update = time.elapsed_secs();
    let Ok(camera) = fly.single() else {
        return;
    };
    let mut lines = Lines {
        positions: Vec::with_capacity(30000),
        colors: Vec::with_capacity(30000),
    };
    if s.kerr {
        let h = super::relativity::horizon(s.radius, s.spin);
        let equator = s.disk_world_radius(h);
        for plane in 0..3 {
            for i in 0..128 {
                let point = |i: usize| {
                    let a = i as f32 * std::f32::consts::TAU / 128.0;
                    let p = match plane {
                        0 => Vec3::new(equator * a.cos(), 0.0, equator * a.sin()),
                        1 => Vec3::new(equator * a.cos(), h * a.sin(), 0.0),
                        _ => Vec3::new(0.0, h * a.sin(), equator * a.cos()),
                    };
                    s.center + s.disk_rotation() * p
                };
                lines.line(point(i), point(i + 1), [0.75, 0.79, 0.84, 1.0], &s);
            }
        }
        for sign in [-1.0, 1.0] {
            let r = s.radius * (1.0 + ((2.0 / 3.0) * (-sign * s.spin).acos()).cos());
            lines.circle(
                s.disk_world_radius(r),
                s.disk_rotation(),
                [0.13, 0.4, 0.95, 1.0],
                &s,
            );
        }
    } else {
        for rotation in [
            Quat::IDENTITY,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        ] {
            lines.circle(s.radius, rotation, [0.75, 0.79, 0.84, 1.0], &s);
            lines.circle(s.radius * 1.5, rotation, [0.13, 0.4, 0.95, 1.0], &s);
        }
    }
    for r in [s.disk_world_radius(s.inner), s.disk_world_radius(s.outer)] {
        lines.circle(r, s.disk_rotation(), [0.9, 0.47, 0.08, 1.0], &s);
    }
    lines.cross(
        camera.translation,
        s.radius * 0.25,
        [1.0, 1.0, 1.0, 1.0],
        &s,
    );
    let mut counts = [0usize; 4];
    let mut detail = String::new();
    let tan = (30.0_f32.to_radians()).tan();
    for y in 0..inspector.side {
        for x in 0..inspector.side {
            let index = y * inspector.side + x;
            let uv = Vec2::new(x as f32, y as f32) / (inspector.side - 1) as f32 * 2.0 - Vec2::ONE;
            let uv = uv * inspector.extent;
            let direction = camera.rotation
                * Vec3::new(
                    uv.x * tan * s.internal.x as f32 / s.internal.y as f32,
                    uv.y * tan,
                    -1.0,
                )
                .normalize();
            let path = integration::trace(camera.translation, direction, &s);
            let class = if !path.hits.is_empty() {
                2
            } else {
                match path.outcome {
                    Outcome::Captured => 0,
                    Outcome::Escaped => 1,
                    Outcome::Exhausted => 3,
                }
            };
            counts[class] += 1;
            let color = if index == inspector.selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [
                    [0.95, 0.12, 0.18, 1.0],
                    [0.13, 0.8, 0.42, 1.0],
                    [1.0, 0.57, 0.07, 1.0],
                    [1.0, 0.05, 0.85, 1.0],
                ][class]
            };
            for p in path.points.windows(2) {
                lines.line(p[0], p[1], color, &s);
            }
            for hit in &path.hits {
                lines.cross(*hit, s.radius * 0.10, [1.0, 0.65, 0.1, 1.0], &s);
            }
            if index == inspector.selected {
                detail = format!(
                    "Selected ray {index}: {} | {} steps\nclosest {:.3} rs | conserved momentum drift {:.1e} | null error {:.1e}\n{} disk intersection(s)",
                    path.class_name(),
                    path.steps,
                    path.closest / s.radius,
                    path.angular_error,
                    path.null_error,
                    path.hits.len()
                );
            }
        }
    }
    if let Some(mut target) = meshes.get_mut(&mesh.0) {
        *target = line_mesh(lines.positions, lines.colors);
    }
    inspector.summary = format!(
        "CPU RAY INSPECTOR (Kerr f64 / Schwarzschild f32) / {} samples\nRadial log scale: |display| = ln(1 + r/rs)\nRed capture {}  Green escape {}\nAmber disk {}  Magenta budget {}\nWhite horizon / Blue orbit references\nKerr: equatorial pro/retro orbits, not full photon shell\n{detail}\nI close | K 9/25 rays | O orbit/top/side\n, / . select | [ / ] screen-ray spread {:.3}",
        inspector.side * inspector.side,
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        inspector.extent
    );
    for mut label in &mut labels {
        **label = inspector.summary.clone();
    }
}
