use super::*;
use crate::{ rng::{self, Xorshift} };

const NUM_SAMPLES: usize = 100_000;

#[test]
fn same_hemisphere_test() {
    assert!(same_hemisphere(
        Direction::Z,
        Direction::X + 0.1 * Direction::Z
    ));
    assert!(!same_hemisphere(
        Direction::Z,
        Direction::NEG_Z
    ));
    assert!(same_hemisphere(
        Direction::Y + 0.1 * Direction::Z,
        Direction::NEG_Y + 0.1 * Direction::Z
    ));

    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let wo = rng::maps::square_to_sphere(rng.gen_vec2());
        let wi = rng::maps::square_to_sphere(rng.gen_vec2());
        assert!(same_hemisphere(wo, wi) != same_hemisphere(wo, -wi));
        assert!(same_hemisphere(wo, wi) == same_hemisphere(-wo, -wi));
    }
}

macro_rules! test_func {
    ( $( $name:ident, $ref:expr, $call:expr),* ) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn is_correct() {
                    let mut rng = Xorshift::default();

                    for _ in 0..NUM_SAMPLES {
                        let w = rng::maps::square_to_sphere(rng.gen_vec2());

                        // tan does not like Z = 1 or Z = 0, relax them a bit
                        let rlx = w.z.abs().min(1.0 - w.z.abs());
                        let threshold = crate::EPSILON / rlx.sqrt();

                        assert!(threshold < 1e-5);
                        assert!(($ref(w) - $call(w)).abs() < threshold);
                    }
                }
            }
        )*
    }
}

test_func!{
    cos_phi, cos_phi, |w| phi(w).cos(),
    sin_phi, sin_phi, |w| phi(w).sin(),

    cos_theta, cos_theta, |w| theta(w).cos(),
    sin_theta, sin_theta, |w| theta(w).sin(),

    cos2_theta, cos2_theta, |w| theta(w).cos().powi(2),
    sin2_theta, sin2_theta, |w| theta(w).sin().powi(2),

    tan_theta,  |_| 1.0, |w| tan_theta(w)  / theta(w).tan(),
    tan2_theta, |_| 1.0, |w| tan2_theta(w) / theta(w).tan().powi(2)
}

#[test]
fn tan_theta_infinite_test() {
    assert!(tan_theta(Direction::X).is_infinite());
    assert!(tan_theta(Direction::Y).is_infinite());
    assert!(tan_theta(Direction::new(1.0, 1.0, Float::MIN).normalize()).is_infinite());
}

#[test]
fn tan2_theta_infinite_test() {
    assert!(tan2_theta(Direction::X).is_infinite());
    assert!(tan2_theta(Direction::Y).is_infinite());
    assert!(tan2_theta(Direction::new(1.0, 1.0, Float::MIN).normalize()).is_infinite());
}
