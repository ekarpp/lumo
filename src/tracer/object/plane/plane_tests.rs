use super::*;

fn plane() -> Box<Plane> {
    Plane::new(DVec3::ZERO, DVec3::Z, Material::Blank)
}

#[test]
fn no_self_intersect() {
    let r = Ray::new(DVec3::ZERO, DVec3::Z);
    assert!(plane().hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn no_intersect_behind() {
    let r = Ray::new(DVec3::ONE, DVec3::Z);
    assert!(plane().hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn intersects() {
    let r = Ray::new(DVec3::NEG_ONE, DVec3::Z);
    assert!(plane().hit(&r, 0.0, INFINITY).is_some());
}

#[test]
fn no_hit_crash_parallel() {
    let p = plane();
    let r = Ray::new(
        DVec3::NEG_ONE,
        DVec3::X,
    );
    assert!(p.hit(&r, 0.0, INFINITY).is_none());
}
