use super::*;

#[test]
fn zero_medium_has_no_effect() {
    let m = Medium::new(DVec3::ZERO, DVec3::ZERO, 0.0);

    let r = Ray::new(DVec3::ZERO, DVec3::Z);
    assert!(m.hit(&r, 0.0, INFINITY).is_none());

    let h = Hit::new(
        100.0,
        &m.material,
        false,
        DVec3::X,
        DVec3::X,
        DVec3::X,
        DVec3::X,
        DVec2::X,
    ).unwrap();
    assert!(m.transmittance(&h) == DVec3::ONE);
}

#[test]
fn medium_gets_hit() {
    let m = Medium::new(DVec3::splat(EPSILON), DVec3::splat(EPSILON), 0.0);

    let r = Ray::new(DVec3::ZERO, DVec3::Z);
    assert!(m.hit(&r, 0.0, INFINITY).is_some());
}
