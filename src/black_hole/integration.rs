//! Small CPU mirror of black_hole.wgsl, for inspection, not production pixels.
//! Keep tetrad, Verlet step, termination and disk tests in sync with WGSL.
use crate::settings::LabSettings;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Captured,
    Escaped,
    Exhausted,
}
#[derive(Debug)]
pub struct RayPath {
    pub points: Vec<Vec3>,
    pub hits: Vec<Vec3>,
    pub outcome: Outcome,
    pub steps: usize,
    pub closest: f32,
    pub angular_error: f32,
    pub null_error: f32,
}
impl RayPath {
    pub fn class_name(&self) -> &'static str {
        if !self.hits.is_empty() {
            "disk hit"
        } else {
            match self.outcome {
                Outcome::Captured => "captured",
                Outcome::Escaped => "escaped",
                Outcome::Exhausted => "exhausted",
            }
        }
    }
}
fn acceleration(p: Vec3, l2: f32, rs: f32) -> Vec3 {
    let r2 = p.length_squared().max(0.01);
    -1.5 * rs * l2 * p / (r2 * r2 * r2.sqrt())
}
pub fn trace(origin: Vec3, direction: Vec3, s: &LabSettings) -> RayPath {
    if s.kerr && s.lensing {
        return super::kerr::trace(origin, direction, s);
    }
    let mut p = origin - s.center;
    let mut v = direction.normalize();
    let mut result = RayPath {
        points: vec![origin],
        hits: Vec::new(),
        outcome: Outcome::Exhausted,
        steps: 0,
        closest: p.length(),
        angular_error: 0.0,
        null_error: 0.0,
    };
    if !s.lensing {
        let b = p.dot(v);
        let c = p.length_squared() - s.radius * s.radius;
        let discriminant = b * b - c;
        let capture = if discriminant >= 0.0 && b < 0.0 {
            -b - discriminant.sqrt()
        } else {
            f32::INFINITY
        };
        let n = s.disk_normal();
        let denominator = v.dot(n);
        if s.disk && denominator.abs() > 0.000001 {
            let t = -p.dot(n) / denominator;
            let hit = p + v * t;
            let r = hit.length();
            if t > 0.0 && t < capture && r > s.inner && r < s.outer {
                result.hits.push(hit + s.center);
                result.points.push(hit + s.center);
            }
        }
        let end = if capture.is_finite() {
            capture
        } else {
            p.length().max(s.outer * 3.0) * 2.3
        };
        result.closest = (p + v * (-b).clamp(0.0, end)).length();
        result.points.push(p + v * end + s.center);
        result.outcome = if capture.is_finite() {
            Outcome::Captured
        } else {
            Outcome::Escaped
        };
        return result;
    }
    let r0 = p.length();
    let radial = p / r0.max(0.0001);
    let vr = v.dot(radial);
    v = radial * vr + (v - radial * vr) / (1.0 - s.radius / r0).max(0.0001).sqrt();
    let angular = p.cross(v);
    let l2 = angular.length_squared();
    let escape_radius = (r0 * 1.15).max(s.outer * 3.0);
    let mut travel = 0.0;
    let mut transmission = 1.0;
    for _ in 0..s.steps.min(768) {
        if travel > s.max_distance {
            break;
        }
        let r = p.length();
        result.closest = result.closest.min(r);
        if r <= s.radius * 1.015 {
            result.outcome = Outcome::Captured;
            break;
        }
        if r > escape_radius && p.dot(v) > 0.0 {
            result.outcome = Outcome::Escaped;
            break;
        }
        let mut h = s.step_scale() * (r * 0.16).min(r * r / (l2.sqrt() + 0.01) * 0.12);
        h = h.clamp(s.radius * 0.018, (s.radius * 12.0).max(r * 0.15));
        h = h.min(r * 0.12 / v.length().max(1.0));
        let old = p;
        let old_v = v;
        let acc = acceleration(p, l2, s.radius);
        p += v * h + 0.5 * acc * h * h;
        v += 0.5 * (acc + acceleration(p, l2, s.radius)) * h;
        travel += h;
        result.steps += 1;
        result.points.push(p + s.center);
        result.angular_error = result
            .angular_error
            .max((p.cross(v) - angular).length() / angular.length().max(1.0));
        let r = p.length();
        result.null_error = result
            .null_error
            .max(((p.dot(v) / r).powi(2) + (1.0 - s.radius / r) * l2 / (r * r) - 1.0).abs());
        let old_plane = old.dot(s.disk_normal());
        let new_plane = p.dot(s.disk_normal());
        if s.disk && old_plane * new_plane <= 0.0 && (new_plane - old_plane).abs() > 0.000001 {
            let mut lo = 0.0;
            let mut hi = 1.0;
            for _ in 0..10 {
                let mid = (lo + hi) * 0.5;
                if hermite(old, p, old_v, v, h, mid).dot(s.disk_normal()) * old_plane > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let hit = hermite(old, p, old_v, v, h, (lo + hi) * 0.5);
            let r = hit.length();
            if r > s.inner && r < s.outer {
                result.hits.push(hit + s.center);
                transmission *= 0.04;
                if transmission < 0.002 {
                    result.outcome = Outcome::Captured;
                    break;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn straight_rays_hit_rotated_disk_and_do_not_skip_capture() {
        let mut s = LabSettings {
            lensing: false,
            disk_tilt: 57.0,
            ..default()
        };
        let hit = s.disk_rotation() * Vec3::new(5.0, 0.0, 0.0);
        let n = s.disk_normal();
        let path = trace(hit + n * 20.0, -n, &s);
        assert_eq!(path.hits.len(), 1);
        assert!(path.hits[0].distance(hit) < 0.0001);
        s.disk = false;
        assert_eq!(
            trace(Vec3::Z * 160.0, Vec3::NEG_Z, &s).outcome,
            Outcome::Captured
        );
        assert_eq!(
            trace(Vec3::new(0.0, 0.0, 160.0), Vec3::X, &s).outcome,
            Outcome::Escaped
        );
    }
    #[test]
    fn bounded_model_handles_far_close_and_changed_mass() {
        let s = LabSettings {
            disk: false,
            ..default()
        };
        assert_eq!(
            trace(Vec3::Z * 160.0, Vec3::NEG_Z, &s).outcome,
            Outcome::Captured
        );
        assert_eq!(
            trace(Vec3::Z * 160.0, Vec3::X, &s).outcome,
            Outcome::Escaped
        );
        assert_eq!(
            trace(Vec3::Z * 1.12, Vec3::NEG_Z, &s).outcome,
            Outcome::Captured
        );
        let direction = Vec3::new(3.0, 0.0, -160.0).normalize();
        assert_eq!(
            trace(Vec3::Z * 160.0, direction, &s).outcome,
            Outcome::Escaped
        );
        let massive = LabSettings {
            radius: 2.0,
            ..s.clone()
        };
        assert_eq!(
            trace(Vec3::Z * 160.0, direction, &massive).outcome,
            Outcome::Captured
        );
    }
}

fn hermite(p: Vec3, q: Vec3, v: Vec3, w: Vec3, h: f32, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p
        + (t3 - 2.0 * t2 + t) * h * v
        + (-2.0 * t3 + 3.0 * t2) * q
        + (t3 - t2) * h * w
}
