use super::*;

fn plane() -> Box<Plane> {
    Plane::new(
        DVec3::ZERO,
        DVec3::ONE,
        Material::Glass
    )
}

#[test]
fn no_self_intersect() {
    let r = Ray::new(
        DVec3::ZERO,
        DVec3::ONE,
    );
    assert!(plane().hit(&r).is_none());
}

#[test]
fn no_intersect_behind() {
    let r = Ray::new(
        DVec3::ONE,
        DVec3::ONE,
    );
    assert!(plane().hit(&r).is_none());
}

#[test]
fn intersects() {
    let r = Ray::new(
        -DVec3::ONE,
        DVec3::ONE,
    );
    assert!(plane().hit(&r).is_some());
}

#[test]
fn no_hit_crash_parallel() {
    let p = plane();
    let r = Ray::new(
        -DVec3::ONE,
        // pray it works
        p.norm.cross(DVec3::new(1.23, 4.56, 7.89)),
    );
    assert!(p.hit(&r).is_none());
}
