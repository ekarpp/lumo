use super::*;
use crate::rand_utils;

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

    for _ in 0..NUM_SAMPLES {
        let wo = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        let wi = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!(same_hemisphere(wo, wi) != same_hemisphere(wo, -wi));
        assert!(same_hemisphere(wo, wi) == same_hemisphere(-wo, -wi));
    }
}

#[test]
fn cos_phi_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!((cos_phi(w) - phi(w).cos()) < crate::EPSILON);
    }
}

#[test]
fn sin_phi_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!((sin_phi(w) - phi(w).sin()) < crate::EPSILON);
    }
}

#[test]
fn cos_theta_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!((cos_theta(w) - theta(w).cos()) < crate::EPSILON);
    }
}

#[test]
fn cos2_theta_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!((cos2_theta(w) - theta(w).cos().powi(2)) < crate::EPSILON);
    }
}

#[test]
fn sin_theta_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!((sin_theta(w) - theta(w).sin()) < crate::EPSILON);
    }
}

#[test]
fn sin2_theta_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_sphere(rand_utils::unit_square()).normalize();
        assert!((sin2_theta(w) - theta(w).sin().powi(2)) < crate::EPSILON);
    }
}

#[test]
fn tan_theta_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_cos_hemisphere(rand_utils::unit_square()).normalize();
        assert!(
            (tan_theta(w) - theta(w).tan()) < 1e-5
                || (tan_theta(w).is_infinite() && theta(w).tan().is_infinite())
        );
    }
}

#[test]
fn tan_theta_infinite_test() {
    assert!(tan_theta(Direction::X).is_infinite());
    assert!(tan_theta(Direction::Y).is_infinite());
    assert!(tan_theta(Direction::new(1.0, 1.0, Float::MIN).normalize()).is_infinite());
}

#[test]
fn tan2_theta_test() {
    for _ in 0..NUM_SAMPLES {
        let w = rand_utils::square_to_cos_hemisphere(rand_utils::unit_square()).normalize();
        assert!(
            (tan2_theta(w) - theta(w).tan().powi(2)) < 1e-5
                || (tan2_theta(w).is_infinite() && theta(w).tan().powi(2).is_infinite())
        );
    }
}

#[test]
fn tan2_theta_infinite_test() {
    assert!(tan2_theta(Direction::X).is_infinite());
    assert!(tan2_theta(Direction::Y).is_infinite());
    assert!(tan2_theta(Direction::new(1.0, 1.0, Float::MIN).normalize()).is_infinite());
}
