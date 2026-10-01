//! A guided observer path. No stage, progress or preset name reaches the renderer.
use crate::settings::{LabSettings, RunOptions};
use bevy::prelude::*;

#[derive(Resource)]
pub struct Journey {
    pub active: bool,
    pub elapsed: f32,
    pub duration: f32,
    pub interrupted: bool,
}
impl Journey {
    pub fn from_options(o: &RunOptions) -> Self {
        Self {
            active: o.journey,
            elapsed: o.journey_at * o.journey_seconds,
            duration: o.journey_seconds,
            interrupted: false,
        }
    }
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
    pub fn stage(&self) -> &'static str {
        match self.progress() {
            p if p < 0.06 => "Distant observation",
            p if p < 0.28 => "Continuous approach",
            p if p < 0.44 => "Lateral orbit",
            p if p < 0.60 => "Inclination change",
            p if p < 0.84 => "Close orbital pass",
            p if p < 1.0 => "Terminal external approach",
            _ => "Static external-observer cutoff",
        }
    }
}

/// Interpolate ordinary spherical camera coordinates, with fixed FOV.
/// Smooth easing makes each segment continuous with zero velocity at joins.
pub fn pose(progress: f32, s: &LabSettings) -> Transform {
    let anchors: [(f32, f32, f32, f32); 7] = [
        (0.0, 160.0, 0.0, 12.0),
        (0.06, 160.0, 0.0, 12.0),
        (0.28, 45.0, 0.0, 8.0),
        (0.44, 28.0, 65.0, 8.0),
        (0.60, 20.0, 110.0, 62.0),
        (0.84, 6.0, 245.0, 18.0),
        (1.0, 1.12, 280.0, 22.0),
    ];
    let p = progress.clamp(0.0, 1.0);
    let i = anchors.windows(2).position(|w| p <= w[1].0).unwrap_or(5);
    let a = anchors[i];
    let b = anchors[i + 1];
    let u = ((p - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
    let u = u * u * (3.0 - 2.0 * u);
    // Log radius keeps the terminal approach slow and geometrically meaningful.
    let cutoff = (s.observer_minimum() / s.radius * 1.01).max(1.12);
    let ar = a.1;
    let br = if i == 5 { cutoff } else { b.1 };
    let radius = (ar.ln() + (br.ln() - ar.ln()) * u).exp() * s.radius;
    let azimuth = (a.2 + (b.2 - a.2) * u).to_radians();
    let elevation = (a.3 + (b.3 - a.3) * u).to_radians();
    let position = s.center
        + radius
            * Vec3::new(
                azimuth.sin() * elevation.cos(),
                elevation.sin(),
                azimuth.cos() * elevation.cos(),
            );
    Transform::from_translation(position).looking_at(s.center, Vec3::Y)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journey_moves_camera_continuously_and_stops_outside_horizon() {
        let s = LabSettings::default();
        assert!((pose(0.0, &s).translation.length() - 160.0).abs() < 0.001);
        assert!((pose(1.0, &s).translation.length() - 1.12).abs() < 0.001);
        for boundary in [0.06, 0.28, 0.44, 0.60, 0.84] {
            assert!(
                pose(boundary - 0.00001, &s)
                    .translation
                    .distance(pose(boundary + 0.00001, &s).translation)
                    < 0.01
            );
        }
        for i in 0..1001 {
            assert!(pose(i as f32 / 1000.0, &s).translation.length() >= 1.1199);
        }
    }
}
