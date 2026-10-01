//! Geometrized G=c=1, rs=2M. Signed spin is relative to positive disk rotation.
pub fn isco(rs: f32, chi: f32) -> f32 {
    let c = chi.clamp(-0.98, 0.98);
    let z1 = 1.0 + (1.0 - c * c).cbrt() * ((1.0 + c).cbrt() + (1.0 - c).cbrt());
    let z2 = (3.0 * c * c + z1 * z1).sqrt();
    0.5 * rs * (3.0 + z2 - c.signum() * ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).max(0.0).sqrt())
}
pub fn horizon(rs: f32, chi: f32) -> f32 {
    0.5 * rs * (1.0 + (1.0 - chi * chi).sqrt())
}
/// Full observed/emitted frequency ratio for equatorial circular emitters.
/// Backward p_t=+1; Lz is the corresponding canonical angular momentum.
#[cfg(test)]
fn frequency(rs: f64, r: f64, observer_r: f64, lz: f64, a: f64) -> f64 {
    let m = rs * 0.5;
    let om = m.sqrt() / (r.powf(1.5) + a * m.sqrt());
    let ut = (1.0 + a * m.sqrt() / r.powf(1.5))
        / (1.0 - 3.0 * m / r + 2.0 * a * m.sqrt() / r.powf(1.5)).sqrt();
    1.0 / ((1.0 - rs / observer_r).sqrt() * ut * (1.0 + om * lz))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_isco_limits() {
        assert!((isco(2.0, 0.0) - 6.0).abs() < 1e-5);
        assert!((isco(2.0, 0.8) - 2.906644).abs() < 1e-4);
        assert!((isco(2.0, -0.8) - 8.431758).abs() < 1e-4);
        assert!(isco(2.0, 0.98) > horizon(2.0, 0.98));
    }
    #[test]
    fn transfer_includes_observer_and_transverse_doppler() {
        assert!((frequency(1.0, 6.0, 1000.0, 0.0, 0.0) - (0.75f64 / 0.999).sqrt()).abs() < 1e-12);
        let approaching = frequency(1.0, 6.0, 1000.0, -3.0, 0.0);
        let receding = frequency(1.0, 6.0, 1000.0, 3.0, 0.0);
        assert!(approaching > 1.0 && receding < 1.0);
        assert!(frequency(1.0, 6.0, 1.12, 0.0, 0.0) > 2.0);
    }
}
