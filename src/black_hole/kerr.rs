//! Independent f64 Kerr-Schild reference and inspector. Same equations/step
//! bounds as GPU f32; deliberately separate precision for validation.
use super::{
    integration::{Outcome, RayPath},
    relativity,
};
use crate::settings::LabSettings;
use bevy::{math::DVec3, prelude::*};
#[derive(Clone, Copy, Debug)]
struct State {
    x: DVec3,
    p: DVec3,
}
struct Metric {
    r: f64,
    h: f64,
    l: DVec3,
    gr: DVec3,
    gh: DVec3,
    gx: DVec3,
    gy: DVec3,
    gz: DVec3,
}
fn metric(x: DVec3, m: f64, a: f64) -> Metric {
    let a2 = a * a;
    let u = x.length_squared() - a2;
    let disc = (u * u + 4.0 * a2 * x.z * x.z).sqrt().max(1e-12);
    let r = (0.5 * (u + disc)).max(1e-12).sqrt();
    let r2 = r * r;
    let b = r2 + a2;
    let gr = DVec3::new(x.x * r, x.y * r, x.z * b / r) / disc;
    let den = r2 * r2 + a2 * x.z * x.z;
    let h = m * r * r2 / den;
    let l = DVec3::new((r * x.x + a * x.y) / b, (r * x.y - a * x.x) / b, x.z / r);
    let gh = h * (3.0 * gr / r - (4.0 * r * r2 * gr + DVec3::new(0.0, 0.0, 2.0 * a2 * x.z)) / den);
    let gx = (x.x * gr + DVec3::new(r, a, 0.0) - 2.0 * r * l.x * gr) / b;
    let gy = (x.y * gr + DVec3::new(-a, r, 0.0) - 2.0 * r * l.y * gr) / b;
    let gz = DVec3::Z / r - x.z * gr / r2;
    Metric {
        r,
        h,
        l,
        gr,
        gh,
        gx,
        gy,
        gz,
    }
}
fn initial(x: DVec3, d: DVec3, m: f64, a: f64) -> State {
    let g = metric(x, m, a);
    let f = (1.0 - 2.0 * g.h).max(1e-5);
    State {
        x,
        p: d / f.sqrt() + g.l * ((1.0 / f - 1.0 / f.sqrt()) * g.l.dot(d) - 2.0 * g.h / f),
    }
}
fn derivative(y: State, m: f64, a: f64) -> State {
    let g = metric(y.x, m, a);
    let q = -1.0 + g.l.dot(y.p);
    State {
        x: y.p - 2.0 * g.h * g.l * q,
        p: g.gh * q * q + 2.0 * g.h * q * (y.p.x * g.gx + y.p.y * g.gy + y.p.z * g.gz),
    }
}
fn add(y: State, d: State, h: f64) -> State {
    State {
        x: y.x + h * d.x,
        p: y.p + h * d.p,
    }
}
fn rk4(y: State, h: f64, m: f64, a: f64) -> State {
    let d1 = derivative(y, m, a);
    let d2 = derivative(add(y, d1, h * 0.5), m, a);
    let d3 = derivative(add(y, d2, h * 0.5), m, a);
    let d4 = derivative(add(y, d3, h), m, a);
    State {
        x: y.x + h * (d1.x + 2.0 * d2.x + 2.0 * d3.x + d4.x) / 6.0,
        p: y.p + h * (d1.p + 2.0 * d2.p + 2.0 * d3.p + d4.p) / 6.0,
    }
}
fn constraint(y: State, m: f64, a: f64) -> f64 {
    let g = metric(y.x, m, a);
    let q = -1.0 + g.l.dot(y.p);
    (y.p.length_squared() - 1.0 - 2.0 * g.h * q * q).abs() / y.p.length_squared().max(1.0)
}
fn hermite(p: DVec3, q: DVec3, v: DVec3, w: DVec3, h: f64, t: f64) -> DVec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p
        + (t3 - 2.0 * t2 + t) * h * v
        + (-2.0 * t3 + 3.0 * t2) * q
        + (t3 - t2) * h * w
}
fn lz(y: State) -> f64 {
    y.x.x * y.p.y - y.x.y * y.p.x
}
pub fn trace(origin: Vec3, direction: Vec3, s: &LabSettings) -> RayPath {
    let rot = s.disk_rotation();
    let bx = (rot * Vec3::X).as_dvec3();
    let by = (rot * Vec3::NEG_Z).as_dvec3();
    let bz = s.disk_normal().as_dvec3();
    let local = |p: DVec3| DVec3::new(bx.dot(p), by.dot(p), bz.dot(p));
    let world = |p: DVec3| (bx * p.x + by * p.y + bz * p.z).as_vec3() + s.center;
    let m = 0.5 * s.radius as f64;
    let a = s.spin as f64 * m;
    let x0 = local((origin - s.center).as_dvec3());
    let mut y = initial(x0, local(direction.normalize().as_dvec3()), m, a);
    let horizon = relativity::horizon(s.radius, s.spin) as f64;
    let escape = (x0.length() * 1.15).max(s.outer as f64 * 3.0);
    let lz0 = lz(y);
    let mut travel = 0.0;
    let mut transmission = 1.0;
    let mut path = RayPath {
        points: vec![origin],
        hits: vec![],
        outcome: Outcome::Exhausted,
        steps: 0,
        closest: metric(y.x, m, a).r as f32,
        angular_error: 0.0,
        null_error: 0.0,
    };
    for _ in 0..s.steps.min(768) {
        if travel > s.max_distance as f64 {
            break;
        }
        let g = metric(y.x, m, a);
        let vel = derivative(y, m, a).x;
        path.closest = path.closest.min(g.r as f32);
        if g.r <= 1.015 * horizon {
            path.outcome = Outcome::Captured;
            break;
        }
        if y.x.length() > escape && y.x.dot(vel) > 0.0 {
            path.outcome = Outcome::Escaped;
            break;
        }
        let mut h = s.step_scale() as f64
            * (0.16 * g.r).min(0.12 * g.r * g.r / (y.x.cross(vel).length() + 0.01));
        h = h.clamp(
            s.radius as f64 * 0.008,
            (12.0 * s.radius as f64).max(g.r * 0.15),
        );
        let refinement = (s.step_scale() as f64 / 0.85).min(1.0);
        h = h.min(0.10 * refinement * g.r / vel.length().max(1.0));
        let horizon_refinement = refinement
            * if s.quality == 2 {
                (1.0 - (s.spin as f64).powi(2)).sqrt().max(0.5)
            } else {
                1.0
            };
        if g.gr.dot(vel) < 0.0 {
            h = h.min(
                0.12 * horizon_refinement * (g.r - horizon).max(0.002 * s.radius as f64)
                    / (-g.gr.dot(vel)).max(0.01),
            );
        }
        let old = y;
        let old_v = vel;
        y = rk4(y, h, m, a);
        travel += h;
        if !y.x.is_finite() || !y.p.is_finite() {
            break;
        }
        path.points.push(world(y.x));
        path.steps += 1;
        path.null_error = path.null_error.max(constraint(y, m, a) as f32);
        path.angular_error = path
            .angular_error
            .max(((lz(y) - lz0).abs() / lz0.abs().max(1.0)) as f32);
        if s.disk && old.x.z * y.x.z < 0.0 && (y.x.z - old.x.z).abs() > 1e-7 {
            let mut lo = 0.0;
            let mut hi = 1.0;
            let next_v = derivative(y, m, a).x;
            for _ in 0..10 {
                let mid = (lo + hi) * 0.5;
                if hermite(old.x, y.x, old_v, next_v, h, mid).z * old.x.z > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let event = rk4(old, h * (lo + hi) * 0.5, m, a);
            let r = metric(event.x, m, a).r;
            if r > s.inner as f64 && r < s.outer as f64 {
                path.hits.push(world(event.x));
                transmission *= 0.04;
                if transmission < 0.002 {
                    path.outcome = Outcome::Captured;
                    break;
                }
            }
        }
    }
    path
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analytic_metric_gradients_match_finite_differences() {
        let x = DVec3::new(2.3, -1.9, 0.7);
        let g = metric(x, 0.5, 0.4);
        for (i, axis) in [DVec3::X, DVec3::Y, DVec3::Z].into_iter().enumerate() {
            let h = 1e-5;
            let plus = metric(x + axis * h, 0.5, 0.4);
            let minus = metric(x - axis * h, 0.5, 0.4);
            assert!(((plus.r - minus.r) / (2.0 * h) - g.gr[i]).abs() < 1e-8);
            assert!(((plus.h - minus.h) / (2.0 * h) - g.gh[i]).abs() < 1e-8);
            let dl = (plus.l - minus.l) / (2.0 * h);
            assert!((dl - DVec3::new(g.gx[i], g.gy[i], g.gz[i])).length() < 1e-8);
        }
    }
    #[test]
    fn static_tetrad_is_null_and_zero_spin_matches_schwarzschild() {
        for spin in [-0.98, 0.0, 0.98] {
            let x = DVec3::new(2.0, -1.0, 0.4);
            for d in [DVec3::X, DVec3::Y, DVec3::Z, -x.normalize()] {
                assert!(constraint(initial(x, d, 0.5, spin * 0.5), 0.5, spin * 0.5) < 1e-14);
            }
        }
        let x = DVec3::new(4.0, 2.0, 1.0);
        let d = DVec3::new(-1.0, 0.2, 0.1).normalize();
        let radial = x.normalize();
        let vr = d.dot(radial);
        let expected = radial * vr + (d - radial * vr) / (1.0 - 1.0 / x.length()).sqrt();
        assert!((derivative(initial(x, d, 0.5, 0.0), 0.5, 0.0).x - expected).length() < 1e-14);
    }
    #[test]
    fn zero_spin_capture_boundary_and_spinning_conservation() {
        let mut s = LabSettings {
            kerr: true,
            spin: 0.0,
            disk: false,
            quality: 2,
            steps: 768,
            ..default()
        };
        for impact in [2.4, 2.8, 4.0] {
            let origin = Vec3::Z * 160.0;
            let d = Vec3::new(impact, 0.0, -160.0).normalize();
            let k = trace(origin, d, &s);
            let reference = super::super::integration::trace(
                origin,
                d,
                &LabSettings {
                    kerr: false,
                    ..s.clone()
                },
            );
            assert_eq!(k.outcome, reference.outcome, "impact {impact}");
            assert!(k.null_error < 0.002, "{}", k.null_error);
        }
        s.spin = 0.8;
        let path = trace(Vec3::new(0.0, 1.0, 29.0), Vec3::new(3.0, -0.1, -29.0), &s);
        assert_ne!(path.outcome, Outcome::Exhausted);
        assert!(path.null_error < 0.001, "{}", path.null_error);
        assert!(path.angular_error < 0.001, "{}", path.angular_error);
    }
    #[test]
    fn axial_momentum_is_an_exact_first_integral_of_the_rhs() {
        let y = initial(
            DVec3::new(3.0, 1.1, 0.7),
            DVec3::new(-1.0, 0.3, -0.1).normalize(),
            0.5,
            0.4,
        );
        let d = derivative(y, 0.5, 0.4);
        let drift = d.x.x * y.p.y + y.x.x * d.p.y - d.x.y * y.p.x - y.x.y * d.p.x;
        assert!(drift.abs() < 1e-14);
    }
    #[test]
    fn reversing_spin_and_reflecting_equatorial_rays_preserves_capture() {
        for impact in [-4.0, -3.0, -2.0, 2.0, 3.0, 4.0] {
            let s = LabSettings {
                kerr: true,
                spin: 0.8,
                disk: false,
                quality: 2,
                steps: 768,
                ..default()
            };
            let origin = Vec3::Z * 160.0;
            let a = trace(origin, Vec3::new(impact, 0.0, -160.0), &s);
            let b = trace(
                origin,
                Vec3::new(-impact, 0.0, -160.0),
                &LabSettings { spin: -0.8, ..s },
            );
            assert_eq!(a.outcome, b.outcome, "impact {impact}");
        }
    }
    #[test]
    fn rk4_residual_converges_at_fourth_order() {
        let initial = initial(
            DVec3::new(3.0, 2.0, 1.0),
            DVec3::new(-1.0, -0.1, 0.2).normalize(),
            0.5,
            0.4,
        );
        let mut errors = vec![];
        for n in [10, 20, 40] {
            let mut y = initial;
            for _ in 0..n {
                y = rk4(y, 0.8 / n as f64, 0.5, 0.4);
            }
            errors.push(constraint(y, 0.5, 0.4));
        }
        assert!(
            errors[0] > errors[1] * 10.0 && errors[1] > errors[2] * 10.0,
            "{errors:?}"
        );
    }
}
