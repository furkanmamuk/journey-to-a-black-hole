//! Independent high-precision reference checks for the null-ray equation used in WGSL.
//! Schwarzschild rs=1, asymptotic E=1, angular momentum b.
fn scatter(b: f64) -> (bool, f64, f64) {
    let mut p = [-200.0, b];
    let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
    // Initial velocity adjusted to the radial energy constraint at finite radius.
    let radial = [p[0] / r, p[1] / r];
    let tangent = [-radial[1], radial[0]];
    let vr = -(1.0 - (1.0 - 1.0 / r) * b * b / (r * r)).sqrt();
    let mut v = [
        vr * radial[0] - b / r * tangent[0],
        vr * radial[1] - b / r * tangent[1],
    ];
    let acceleration = |p: [f64; 2]| {
        let r2 = p[0] * p[0] + p[1] * p[1];
        let f = -1.5 * b * b / r2.powf(2.5);
        [f * p[0], f * p[1]]
    };
    let mut error = 0.0_f64;
    for _ in 0..200_000 {
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        if r < 1.01 {
            return (true, 0.0, error);
        }
        if r > 220.0 && p[0] * v[0] + p[1] * v[1] > 0.0 {
            return (false, v[1].atan2(v[0]), error);
        }
        let h = (r * 0.004).min(0.25);
        let a = acceleration(p);
        for j in 0..2 {
            p[j] += v[j] * h + 0.5 * a[j] * h * h;
        }
        let next = acceleration(p);
        for j in 0..2 {
            v[j] += 0.5 * (a[j] + next[j]) * h;
        }
        let angular = p[0] * v[1] - p[1] * v[0];
        error = error.max((angular.abs() - b).abs());
    }
    panic!("reference integration failed to terminate");
}
#[test]
fn shadow_matches_schwarzschild_critical_impact_parameter() {
    let critical = 3.0_f64.sqrt() * 1.5;
    assert!(scatter(critical * 0.99).0);
    assert!(!scatter(critical * 1.01).0);
}
#[test]
fn weak_field_deflection_and_angular_momentum() {
    let (captured, angle, error) = scatter(30.0);
    assert!(!captured);
    assert!(
        (angle.abs() - 2.0 / 30.0).abs() < 0.009,
        "deflection {angle}"
    );
    assert!(error < 1e-8, "angular momentum drift {error}");
}
