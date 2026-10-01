use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct LabSettings {
    pub kerr: bool,
    pub spin: f32,
    pub isco_lock: bool,
    pub temperature: f32,
    pub error_debug: bool,
    pub lensing: bool,
    pub disk: bool,
    pub stars: bool,
    pub craft: bool,
    pub bloom: bool,
    pub taa: bool,
    pub quantize: bool,
    pub tonal: bool,
    pub dither: bool,
    pub auto_exposure: bool,
    pub stellar_light: bool,
    pub doppler: bool,
    pub beaming: bool,
    pub redshift: bool,
    pub budget_debug: bool,
    pub quality: usize,
    pub steps: u32,
    pub radius: f32,
    pub inner: f32,
    pub outer: f32,
    pub max_distance: f32,
    pub ev: f32,
    pub levels: f32,
    pub internal: UVec2,
    pub grid: UVec2,
    pub center: Vec3,
    pub disk_tilt: f32,
    pub disk_azimuth: f32,
    pub emissivity: f32,
    pub seed: u32,
    pub sky_mode: u32,
    pub ray_classes: bool,
}
impl Default for LabSettings {
    fn default() -> Self {
        Self {
            kerr: false,
            spin: 0.8,
            isco_lock: true,
            temperature: 18000.0,
            error_debug: false,
            lensing: true,
            disk: true,
            stars: true,
            craft: true,
            bloom: true,
            taa: false,
            quantize: true,
            tonal: true,
            dither: true,
            auto_exposure: false,
            stellar_light: true,
            doppler: true,
            beaming: true,
            redshift: true,
            budget_debug: false,
            quality: 1,
            steps: 256,
            radius: 1.0,
            inner: 3.0,
            outer: 11.0,
            max_distance: 2000.0,
            ev: 9.7,
            levels: 48.0,
            internal: UVec2::new(960, 540),
            grid: UVec2::new(480, 270),
            center: Vec3::ZERO,
            disk_tilt: 0.0,
            disk_azimuth: 0.0,
            emissivity: 1.0,
            seed: 441991,
            sky_mode: 0,
            ray_classes: false,
        }
    }
}
impl LabSettings {
    pub fn set_quality(&mut self, quality: usize) {
        self.quality = quality % 3;
        self.steps = [128, 256, 768][self.quality];
    }
    pub fn sanitize(&mut self) {
        let defaults = Self::default();
        macro_rules! finite {($($field:ident),*)=>{$(if !self.$field.is_finite() {self.$field=defaults.$field;})*};}
        finite!(
            radius,
            inner,
            outer,
            max_distance,
            ev,
            levels,
            disk_tilt,
            disk_azimuth,
            emissivity,
            spin,
            temperature
        );
        if !self.center.is_finite() || self.center.abs().max_element() > 1e6 {
            self.center = Vec3::ZERO;
        }

        self.radius = self.radius.clamp(0.2, 4.0);
        self.spin = self.spin.clamp(-0.98, 0.98);
        self.temperature = self.temperature.clamp(2000.0, 100000.0);
        let isco = crate::black_hole::relativity::isco(
            self.radius,
            if self.kerr { self.spin } else { 0.0 },
        );
        self.inner = if self.isco_lock {
            isco
        } else {
            self.inner.clamp(isco, 200.0 * self.radius)
        };
        self.outer = self
            .outer
            .clamp(self.inner + 0.5 * self.radius, 500.0 * self.radius);
        self.steps = self.steps.clamp(32, 768);
        self.max_distance = self.max_distance.clamp(50.0, 20000.0);
        self.quality %= 3;
        self.sky_mode %= 3;
        self.disk_tilt = self.disk_tilt.clamp(-180.0, 180.0);
        self.disk_azimuth = self.disk_azimuth.rem_euclid(360.0);
        self.emissivity = self.emissivity.clamp(0.001, 100.0);
        self.internal = self
            .internal
            .clamp(UVec2::new(320, 180), UVec2::new(3840, 2160));
        let minimum_grid = ((self.internal + UVec2::splat(7)) / 8).max(UVec2::new(160, 90));
        self.grid = self.grid.clamp(minimum_grid, self.internal);
        self.ev = self.ev.clamp(-2.0, 20.0);
        self.levels = self.levels.clamp(8.0, 256.0);
    }
    pub fn observer_minimum(&self) -> f32 {
        let a = if self.kerr {
            self.spin * self.radius * 0.5
        } else {
            0.0
        };
        ((1.08 * self.radius).powi(2) + a * a).sqrt()
    }
    pub fn disk_world_radius(&self, r: f32) -> f32 {
        let a = if self.kerr {
            self.spin * self.radius * 0.5
        } else {
            0.0
        };
        (r * r + a * a).sqrt()
    }
    pub fn disk_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.disk_azimuth.to_radians())
            * Quat::from_rotation_x(self.disk_tilt.to_radians())
    }
    pub fn disk_normal(&self) -> Vec3 {
        self.disk_rotation() * Vec3::Y
    }
    pub fn step_scale(&self) -> f32 {
        [1.55, 0.85, 0.45][self.quality]
    }
}

#[derive(Resource, Default)]
pub struct RunOptions {
    pub seconds: Option<f32>,
    pub benchmark: bool,
    pub capture: bool,
    pub validate: bool,
    pub hide_hud: bool,
    pub fullscreen: bool,
    pub output: UVec2,
    pub preset: usize,
    pub journey: bool,
    pub journey_seconds: f32,
    pub journey_at: f32,
    pub inspector: bool,
    pub config: Option<String>,
    pub initial_camera: Option<Transform>,
    pub speed: f32,
    pub capture_at: f32,
    pub capture_count: usize,
    pub capture_interval: f32,
    pub capture_label: String,
    pub freeze_input: bool,
}
pub fn parse_options() -> (LabSettings, RunOptions) {
    let mut s = LabSettings::default();
    let mut o = RunOptions {
        output: UVec2::new(1920, 1080),
        preset: 1,
        journey_seconds: 150.0,
        speed: 4.0,
        capture_at: 10.0,
        capture_count: 1,
        capture_interval: 0.2,
        ..default()
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let value = args.get(i + 1).map(String::as_str).unwrap_or("");
        match args[i].as_str() {
            "--kerr" => s.kerr = true,
            "--spin" => {
                s.spin = value.parse().expect("dimensionless spin");
                i += 1;
            }
            "--temperature" => {
                s.temperature = value.parse().expect("temperature kelvin");
                i += 1;
            }
            "--free-inner" => s.isco_lock = false,
            "--error-debug" => s.error_debug = true,
            "--seconds" => {
                o.seconds = Some(value.parse().expect("seconds number"));
                i += 1;
            }
            "--preset" => {
                o.preset = value.parse().expect("preset 1..7");
                i += 1;
            }
            "--quality" => {
                s.set_quality(value.parse().expect("quality 0..2"));
                i += 1;
            }
            "--steps" => {
                s.steps = value.parse().expect("steps integer");
                i += 1;
            }
            "--internal" | "--grid" | "--output" => {
                let (w, h) = value.split_once('x').expect("resolution WIDTHxHEIGHT");
                let size = UVec2::new(w.parse().unwrap(), h.parse().unwrap());
                match args[i].as_str() {
                    "--internal" => s.internal = size,
                    "--grid" => s.grid = size,
                    _ => o.output = size,
                };
                i += 1;
            }
            "--benchmark" => o.benchmark = true,
            "--journey" => o.journey = true,
            "--journey-seconds" => {
                o.journey_seconds = value.parse().expect("journey seconds");
                i += 1;
            }
            "--journey-at" => {
                o.journey_at = value.parse().expect("journey progress 0..1");
                i += 1;
            }
            "--inspect-rays" => o.inspector = true,
            "--ray-classes" => s.ray_classes = true,
            "--sky-grid" => s.sky_mode = 1,
            "--sky-both" => s.sky_mode = 2,
            "--seed" => {
                s.seed = value.parse().expect("star seed u32");
                i += 1;
            }
            "--disk-tilt" => {
                s.disk_tilt = value.parse().expect("tilt degrees");
                i += 1;
            }
            "--disk-azimuth" => {
                s.disk_azimuth = value.parse().expect("azimuth degrees");
                i += 1;
            }
            "--emissivity" => {
                s.emissivity = value.parse().expect("emission multiplier");
                i += 1;
            }
            "--config" => {
                o.config = Some(value.to_owned());
                i += 1;
            }
            "--capture" => o.capture = true,
            "--ev" => {
                s.ev = value.parse().expect("EV100");
                i += 1;
            }
            "--capture-count" => {
                o.capture_count = value.parse().expect("capture count");
                i += 1;
            }
            "--capture-interval" => {
                o.capture_interval = value.parse().expect("capture interval");
                i += 1;
            }
            "--capture-at" => {
                o.capture_at = value.parse().expect("capture time");
                i += 1;
            }
            "--capture-label" => {
                o.capture_label = value.to_owned();
                i += 1;
            }
            "--freeze-input" => o.freeze_input = true,
            "--validate" => {
                o.validate = true;
                o.seconds.get_or_insert(86.0);
            }
            "--hide-hud" => o.hide_hud = true,
            "--fullscreen" => o.fullscreen = true,
            "--budget-debug" => s.budget_debug = true,
            "--no-tonal" => s.tonal = false,
            "--no-disk" => s.disk = false,
            "--no-stars" => s.stars = false,
            "--no-craft" => s.craft = false,
            "--no-stellar-light" => s.stellar_light = false,
            "--no-quantization" => s.quantize = false,
            "--no-lensing" => s.lensing = false,
            "--no-bloom" => s.bloom = false,
            "--taa" => s.taa = true,
            "--auto-exposure" => s.auto_exposure = true,
            "--help" => {
                println!(
                    "Journey to a Black Hole: --journey --journey-seconds 150 --journey-at 0..1 --preset 1..7 --inspect-rays --ray-classes --sky-grid --sky-both --seed 441991 --disk-tilt DEGREES --disk-azimuth DEGREES --emissivity 1 --config FILE\n--internal 960x540 --grid 480x270 --output 1920x1080 --fullscreen --quality 0|1|2 --steps 256 --seconds 30 --benchmark --validate --capture --hide-hud --taa --auto-exposure --budget-debug\n--kerr --spin -0.98..0.98 --temperature K --free-inner --error-debug --ev EV100 --capture-count COUNT --capture-interval SECONDS --capture-at SECONDS --capture-label LABEL --freeze-input\n--no-quantization --no-tonal --no-lensing --no-disk --no-stars --no-craft --no-stellar-light --no-bloom"
                );
                std::process::exit(0);
            }
            a => panic!("unknown option {a}; see --help"),
        }
        i += 1;
    }
    assert!(
        [
            s.radius,
            s.inner,
            s.outer,
            s.max_distance,
            s.ev,
            s.levels,
            s.disk_tilt,
            s.disk_azimuth,
            s.emissivity,
            s.spin,
            s.temperature
        ]
        .iter()
        .all(|v| v.is_finite()),
        "physical options require finite numbers"
    );
    assert!(
        o.output.x >= 320 && o.output.y >= 180 && o.output.x <= 7680 && o.output.y <= 4320,
        "output size 320x180..7680x4320"
    );
    assert!(
        o.seconds.is_none_or(|v| v.is_finite() && v > 0.0),
        "positive finite run duration"
    );
    assert!((1..=7).contains(&o.preset), "preset 1..7");
    s.sanitize();
    assert!(
        o.capture_count <= 300 && o.capture_interval.is_finite() && o.capture_interval >= 0.05,
        "capture count <=300 and interval >=0.05s"
    );
    assert!(
        o.capture_at.is_finite() && o.capture_at >= 0.0,
        "nonnegative capture time"
    );
    assert!(
        o.capture_label.len() <= 64
            && o.capture_label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-'),
        "capture label uses ASCII letters, digits and hyphens"
    );
    assert!(
        o.journey_seconds.is_finite() && o.journey_seconds >= 5.0,
        "journey duration >= 5 seconds"
    );
    assert!(
        o.journey_at.is_finite() && (0.0..=1.0).contains(&o.journey_at),
        "journey progress 0..1"
    );
    (s, o)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disk_stays_outside_isco_and_grid_inside_target() {
        let mut s = LabSettings {
            radius: 2.0,
            inner: 1.0,
            outer: 2.0,
            grid: UVec2::splat(9000),
            ..default()
        };
        s.sanitize();
        assert_eq!(s.inner, 6.0);
        assert!(s.outer > s.inner);
        assert_eq!(s.grid, s.internal);
    }
}
