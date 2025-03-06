use super::*;
use crate::{ Point, Direction };

#[test]
fn zero_medium_has_no_effect() {
    let m = Medium::new(Vec3::ZERO, Vec3::ZERO, 0.0);
    let mut rng = Xorshift::default();

    let r = Ray::new(Point::ZERO, Direction::Z);
    assert!(m.hit(&r, &mut rng, 0.0, crate::INF).is_none());

    let h = Hit::new(
        100.0,
        &m.material,
        -Direction::X,
        Point::X,
        Vec3::X,
        Normal::X,
        Normal::X,
        Vec2::X,
    ).unwrap();
    let lambda = ColorWavelength::sample(rng.gen_float());
    let tr = m.transmittance(&lambda, h.t);
    assert!((tr - Color::WHITE).is_black());
}

#[test]
fn medium_gets_hit() {
    let m = Medium::new(Vec3::splat(crate::EPSILON), Vec3::splat(crate::EPSILON), 0.0);
    let mut rng = Xorshift::default();

    let r = Ray::new(Point::ZERO, Direction::Z);
    assert!(m.hit(&r, &mut rng, 0.0, crate::INF).is_some());
}
