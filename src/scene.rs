use crate::{materials, settings::LabSettings};
use bevy::prelude::*;
#[derive(Component)]
pub struct Probe;
#[derive(Component)]
pub struct DiskLight(pub usize);
#[derive(Component)]
pub struct StellarLight;
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    s: Res<LabSettings>,
) {
    commands.insert_resource(GlobalAmbientLight {
        brightness: 0.0,
        ..default()
    });
    let m = materials::create(&mut materials, &mut images);
    let body = meshes.add(Cylinder::new(0.72, 2.6).mesh().resolution(64));
    let unit = meshes.add(Cuboid::default());
    let strut = meshes.add(Cylinder::new(0.045, 1.0).mesh().resolution(16));
    let dish = meshes.add(Sphere::new(0.4).mesh().uv(32, 16));
    commands
        .spawn((
            Name::new("industrial probe"),
            Probe,
            Transform::from_xyz(3.8, 0.4, 18.0)
                .with_rotation(Quat::from_rotation_z(0.16) * Quat::from_rotation_y(0.6)),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(body),
                MeshMaterial3d(m.metal.clone()),
                Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ));
            for z in [-1.35, 1.35] {
                parent.spawn((
                    Mesh3d(unit.clone()),
                    MeshMaterial3d(m.ceramic.clone()),
                    Transform::from_xyz(0.0, 0.0, z).with_scale(Vec3::new(1.05, 1.05, 0.16)),
                ));
            }
            for side in [-1.0, 1.0] {
                parent.spawn((
                    Mesh3d(unit.clone()),
                    MeshMaterial3d(m.coating.clone()),
                    Transform::from_xyz(side * 1.85, 0.0, 0.0)
                        .with_scale(Vec3::new(1.55, 0.08, 2.4)),
                ));
                for z in [-1.22, 1.22] {
                    parent.spawn((
                        Mesh3d(unit.clone()),
                        MeshMaterial3d(m.frame.clone()),
                        Transform::from_xyz(side * 1.85, 0.0, z)
                            .with_scale(Vec3::new(1.65, 0.13, 0.08)),
                    ));
                }
                for i in 0..8 {
                    parent.spawn((
                        Mesh3d(unit.clone()),
                        MeshMaterial3d(m.frame.clone()),
                        Transform::from_xyz(side * (1.15 + i as f32 * 0.2), 0.052, 0.0)
                            .with_scale(Vec3::new(0.035, 0.04, 2.3)),
                    ));
                }
                for z in [-0.85, 0.85] {
                    parent.spawn((
                        Mesh3d(strut.clone()),
                        MeshMaterial3d(m.frame.clone()),
                        Transform::from_xyz(side * 0.97, -0.05, z)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::new(1.0, 1.2, 1.0)),
                    ));
                }
                parent.spawn((
                    Mesh3d(unit.clone()),
                    MeshMaterial3d(m.coating.clone()),
                    Transform::from_xyz(side * 0.55, 0.65, 0.25)
                        .with_scale(Vec3::new(0.65, 0.5, 0.95)),
                ));
                for z in [-0.9, 0.9] {
                    parent.spawn((
                        Mesh3d(meshes.add(Cylinder::new(0.16, 0.4).mesh().resolution(32))),
                        MeshMaterial3d(m.metal.clone()),
                        Transform::from_xyz(side * 0.62, -0.6, z)
                            .with_rotation(Quat::from_rotation_x(0.5)),
                    ));
                }
            }
            parent.spawn((
                Mesh3d(strut),
                MeshMaterial3d(m.frame.clone()),
                Transform::from_xyz(0.0, 1.3, -0.4).with_scale(Vec3::new(0.6, 1.3, 0.6)),
            ));
            parent.spawn((
                Mesh3d(dish),
                MeshMaterial3d(m.metal.clone()),
                Transform::from_xyz(0.0, 1.9, -0.4).with_scale(Vec3::new(1.0, 0.16, 1.0)),
            ));
            parent.spawn((
                Mesh3d(unit),
                MeshMaterial3d(m.instrument),
                Transform::from_xyz(0.0, 0.77, 0.9).with_scale(Vec3::new(0.25, 0.07, 0.22)),
            ));
            parent.spawn((
                PointLight {
                    color: Color::srgb(0.2, 0.9, 0.7),
                    intensity: 8.0,
                    range: 2.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.85, 0.9),
            ));
        });
    // Finite quadrature lights approximate disk irradiation; no vacuum fill.
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        commands.spawn((
            DiskLight(i),
            PointLight {
                color: Color::srgb(1.0, 0.79, 0.63),
                intensity: 6_000_000.0,
                range: 150.0,
                radius: 1.0,
                shadow_maps_enabled: i == 0 || i == 4,
                ..default()
            },
            Transform::from_xyz(
                angle.cos() * 5.0 * s.radius,
                0.0,
                angle.sin() * 5.0 * s.radius,
            ),
        ));
    }
    commands.spawn((
        StellarLight,
        DirectionalLight {
            illuminance: 6000.0,
            color: Color::srgb(0.75, 0.85, 1.0),
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -1.1, 0.0)),
    ));
}
pub fn sync(
    s: Res<LabSettings>,
    mut probes: Query<&mut Visibility, With<Probe>>,
    mut lights: Query<(&DiskLight, &mut PointLight, &mut Transform)>,
    mut stellar: Query<&mut DirectionalLight, With<StellarLight>>,
) {
    if !s.is_changed() {
        return;
    }
    for mut v in &mut probes {
        *v = if s.craft {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (tag, mut light, mut t) in &mut lights {
        light.intensity = if s.disk {
            6_000_000.0 * s.radius * s.radius * s.emissivity
        } else {
            0.0
        };
        let a = tag.0 as f32 * std::f32::consts::TAU / 8.0;
        let r = s.disk_world_radius((s.inner + s.outer) * 0.38);
        t.translation = s.center + s.disk_rotation() * Vec3::new(a.cos() * r, 0.0, a.sin() * r);
    }
    for mut light in &mut stellar {
        light.illuminance = if s.stellar_light { 6000.0 } else { 0.0 };
    }
}
