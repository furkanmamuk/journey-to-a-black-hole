//! Dependency-free, versioned, human-readable reproducible scene state.
//! Parsing is transactional: malformed files never partly change a running scene.
use crate::{
    camera::{FlyCamera, Navigation},
    journey::Journey,
    settings::{LabSettings, RunOptions},
};
use bevy::prelude::*;
use std::{collections::HashSet, fmt::Write, path::Path};

pub fn encode(s: &LabSettings, camera: &Transform, speed: f32) -> String {
    let mut text =
        String::from("# Journey to a Black Hole — external relativistic scene\nversion = 1\n");
    macro_rules! field {
        ($key:literal,$value:expr) => {
            writeln!(text, "{} = {}", $key, $value).unwrap()
        };
    }
    field!("rs", s.radius);
    field!("spin", s.spin);
    field!("temperature", s.temperature);
    field!("disk_inner", s.inner);
    field!("disk_outer", s.outer);
    field!("disk_tilt", s.disk_tilt);
    field!("disk_azimuth", s.disk_azimuth);
    field!("emissivity", s.emissivity);
    field!("star_seed", s.seed);
    field!("sky_mode", s.sky_mode);
    field!("quality", s.quality);
    field!("steps", s.steps);
    field!("max_distance", s.max_distance);
    field!("ev100", s.ev);
    field!("tonal_levels", s.levels);
    for (name, v) in [
        ("lensing", s.lensing),
        ("kerr", s.kerr),
        ("isco_lock", s.isco_lock),
        ("error_debug", s.error_debug),
        ("disk", s.disk),
        ("stars", s.stars),
        ("probe", s.craft),
        ("bloom", s.bloom),
        ("taa", s.taa),
        ("quantize", s.quantize),
        ("tonal", s.tonal),
        ("dither", s.dither),
        ("auto_exposure", s.auto_exposure),
        ("stellar_light", s.stellar_light),
        ("doppler", s.doppler),
        ("beaming", s.beaming),
        ("redshift", s.redshift),
        ("budget_debug", s.budget_debug),
        ("ray_classes", s.ray_classes),
    ] {
        writeln!(text, "{name} = {v}").unwrap();
    }
    writeln!(
        text,
        "center = {} {} {}",
        s.center.x, s.center.y, s.center.z
    )
    .unwrap();
    writeln!(
        text,
        "internal = {} {}\ngrid = {} {}",
        s.internal.x, s.internal.y, s.grid.x, s.grid.y
    )
    .unwrap();
    let p = camera.translation;
    let q = camera.rotation;
    writeln!(
        text,
        "camera_position = {} {} {}\ncamera_rotation = {} {} {} {}\nmovement_speed = {speed}",
        p.x, p.y, p.z, q.x, q.y, q.z, q.w
    )
    .unwrap();
    text
}
fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("invalid value '{value}'"))
}
fn floats<const N: usize>(value: &str) -> Result<[f32; N], String> {
    let parts = value
        .split_ascii_whitespace()
        .map(number::<f32>)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != N || parts.iter().any(|v| !v.is_finite()) {
        return Err(format!("expected {N} finite numbers"));
    }
    Ok(parts.try_into().unwrap())
}
pub fn decode(text: &str) -> Result<(LabSettings, Transform, f32), String> {
    let mut s = LabSettings::default();
    let mut camera = crate::camera::preset(1, &s);
    let mut speed = 4.0;
    let mut seen = HashSet::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {}: expected key = value", i + 1))?;
        let key = key.trim();
        let value = value.trim();
        if !seen.insert(key) {
            return Err(format!("line {}: duplicate {key}", i + 1));
        }
        let parsed = (|| -> Result<(), String> {
            macro_rules! scalar {
                ($field:ident) => {{
                    let v = floats::<1>(value)?[0];
                    s.$field = v;
                }};
            }
            match key {
                "version" => {
                    if value != "1" {
                        return Err("unsupported configuration version".into());
                    }
                }
                "rs" => scalar!(radius),
                "spin" => scalar!(spin),
                "temperature" => scalar!(temperature),
                "kerr" => s.kerr = number(value)?,
                "isco_lock" => s.isco_lock = number(value)?,
                "error_debug" => s.error_debug = number(value)?,
                "disk_inner" => scalar!(inner),
                "disk_outer" => scalar!(outer),
                "disk_tilt" => scalar!(disk_tilt),
                "disk_azimuth" => scalar!(disk_azimuth),
                "emissivity" => scalar!(emissivity),
                "max_distance" => scalar!(max_distance),
                "ev100" => scalar!(ev),
                "tonal_levels" => scalar!(levels),
                "star_seed" => s.seed = number(value)?,
                "sky_mode" => s.sky_mode = number(value)?,
                "quality" => s.quality = number(value)?,
                "steps" => s.steps = number(value)?,
                "center" => s.center = Vec3::from_array(floats(value)?),
                "camera_position" => camera.translation = Vec3::from_array(floats(value)?),
                "camera_rotation" => {
                    let q = Quat::from_array(floats(value)?);
                    if !q.length().is_finite() || q.length() < 0.001 {
                        return Err("zero camera quaternion".into());
                    }
                    camera.rotation = q.normalize();
                }
                "movement_speed" => speed = floats::<1>(value)?[0],
                "internal" | "grid" => {
                    let parts = value
                        .split_ascii_whitespace()
                        .map(number::<u32>)
                        .collect::<Result<Vec<_>, _>>()?;
                    if parts.len() != 2 {
                        return Err("resolution needs two integers".into());
                    }
                    let size = UVec2::new(parts[0], parts[1]);
                    if key == "internal" {
                        s.internal = size;
                    } else {
                        s.grid = size;
                    }
                }
                "lensing" => s.lensing = number(value)?,
                "disk" => s.disk = number(value)?,
                "stars" => s.stars = number(value)?,
                "probe" => s.craft = number(value)?,
                "bloom" => s.bloom = number(value)?,
                "taa" => s.taa = number(value)?,
                "quantize" => s.quantize = number(value)?,
                "tonal" => s.tonal = number(value)?,
                "dither" => s.dither = number(value)?,
                "auto_exposure" => s.auto_exposure = number(value)?,
                "stellar_light" => s.stellar_light = number(value)?,
                "doppler" => s.doppler = number(value)?,
                "beaming" => s.beaming = number(value)?,
                "redshift" => s.redshift = number(value)?,
                "budget_debug" => s.budget_debug = number(value)?,
                "ray_classes" => s.ray_classes = number(value)?,
                _ => return Err(format!("unknown key {key}")),
            }
            Ok(())
        })();
        parsed.map_err(|e| format!("line {} ({key}): {e}", i + 1))?;
    }
    if !seen.contains("version") {
        return Err("missing version = 1".into());
    }
    if !seen.contains("isco_lock") {
        s.isco_lock = false;
    }
    if camera.translation.abs().max_element() > 1e6 {
        return Err("camera coordinates exceed 1e6 scene units".into());
    }
    s.sanitize();
    speed = speed.clamp(0.02, 2000.0);
    if !seen.contains("camera_position") {
        camera.translation = crate::camera::preset(1, &s).translation;
    }
    Ok((s, camera, speed))
}
pub fn load(path: impl AsRef<Path>) -> Result<(LabSettings, Transform, f32), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    decode(&text)
}
pub fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    o: Res<RunOptions>,
    mut s: ResMut<LabSettings>,
    mut cameras: Query<
        (
            &mut Transform,
            Option<&mut bevy::anti_alias::taa::TemporalAntiAliasing>,
        ),
        With<FlyCamera>,
    >,
    mut nav: ResMut<Navigation>,
    mut journey: ResMut<Journey>,
) {
    let saved_path = "journey.cfg";
    if keys.just_pressed(KeyCode::F12) {
        if let Ok((camera, _)) = cameras.single() {
            nav.notice = match std::fs::write(saved_path, encode(&s, camera, nav.speed)) {
                Ok(_) => format!("Saved reproducible scene to {saved_path}"),
                Err(e) => format!("Save failed: {e}"),
            };
        }
    }
    if keys.just_pressed(KeyCode::KeyL) {
        let path = if Path::new(saved_path).is_file() {
            saved_path
        } else {
            o.config.as_deref().unwrap_or(saved_path)
        };
        match load(path) {
            Ok((settings, camera, speed)) => {
                *s = settings;
                if let Ok((mut t, taa)) = cameras.single_mut() {
                    *t = camera;
                    if let Some(mut taa) = taa {
                        taa.reset = true;
                    }
                }
                nav.speed = speed;
                journey.active = false;
                journey.interrupted = true;
                nav.notice = format!("Loaded {path}");
            }
            Err(e) => nav.notice = format!("Load failed: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_round_trip_keeps_physics_camera_seed_and_quality() {
        let s = LabSettings {
            seed: u32::MAX,
            disk_tilt: 37.0,
            radius: 1.2,
            emissivity: 2.4,
            kerr: true,
            spin: -0.6,
            isco_lock: true,
            temperature: 23000.0,
            error_debug: true,
            sky_mode: 2,
            ..default()
        };
        let camera = Transform::from_xyz(14.0, -3.0, 29.0).looking_at(Vec3::ZERO, Vec3::Y);
        let (restored, c, speed) = decode(&encode(&s, &camera, 7.5)).unwrap();
        assert_eq!(restored.seed, s.seed);
        assert!(restored.kerr && restored.isco_lock && restored.error_debug);
        assert_eq!(restored.spin, s.spin);
        assert_eq!(restored.temperature, s.temperature);
        assert_eq!(restored.disk_normal(), s.disk_normal());
        assert_eq!(restored.emissivity, s.emissivity);
        assert_eq!(c.translation, camera.translation);
        assert!(c.rotation.dot(camera.rotation).abs() > 0.99999);
        assert_eq!(speed, 7.5);
    }
    #[test]
    fn invalid_or_ambiguous_files_are_rejected() {
        for text in [
            "version = 9",
            "version = 1\nrs = NaN",
            "version = 1\nstar_seed = 1\nstar_seed = 2",
            "version = 1\nmagic_ring_size = 4",
            "version = 1\ncamera_rotation = 0 0 0 0",
        ] {
            assert!(decode(text).is_err());
        }
    }
}
