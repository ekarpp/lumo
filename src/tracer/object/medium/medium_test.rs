use super::*;

#[test]
fn zero_medium_has_no_effect() {
    let m = Medium::new(Vec3::ZERO, Vec3::ZERO, 0.0);

    let r = Ray::new(Point::ZERO, Direction::Z);
    assert!(m.hit(&r, 0.0, crate::INF).is_none());

    let h = Hit::new(
        100.0,
        &m.material,
        Direction::NEG_X,
        Point::X,
        Vec3::X,
        Normal::X,
        Normal::X,
        Vec2::X,
    ).unwrap();
    let tr = m.transmittance(&ColorWavelength::sample(rand_utils::rand_float()), h.t);
    assert!((tr - Color::WHITE).is_black());
}

#[test]
fn medium_gets_hit() {
    let m = Medium::new(Vec3::splat(crate::EPSILON), Vec3::splat(crate::EPSILON), 0.0);

    let r = Ray::new(Point::ZERO, Direction::Z);
    assert!(m.hit(&r, 0.0, crate::INF).is_some());
}
