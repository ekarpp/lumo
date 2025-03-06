use super::*;
use crate::rng::Xorshift;

const NUM_SAMPLES: usize = 10_000;

#[test]
fn normalize() {
    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let v = rng.gen_vec3();
        assert!(v.normalize().is_normalized());
        let v = v *1e10 * rng.gen_float();
        assert!(v.normalize().is_normalized());
    }
}

#[test]
fn min_element() {
    let v = Vec3::new(1.23, 4.56, 7.89);
    assert!(v.min_element() == v.x);

    let v = Vec3::new(7.89, 1.23, 4.56);
    assert!(v.min_element() == v.y);

    let v = Vec3::new(4.56, 7.89, 1.23);
    assert!(v.min_element() == v.z);
}

#[test]
fn max_element() {
    let v = Vec3::new(1.23, 4.56, 7.89);
    assert!(v.max_element() == v.z);

    let v = Vec3::new(7.89, 1.23, 4.56);
    assert!(v.max_element() == v.x);

    let v = Vec3::new(4.56, 7.89, 1.23);
    assert!(v.max_element() == v.y);
}
