use super::*;

fn plane() -> Box<Plane> {
    Plane::new(Point::ZERO, Point::Z, Material::Blank)
}

#[test]
fn no_self_intersect() {
    let r = Ray::new(Point::ZERO, Direction::Z);
    assert!(plane().hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn no_intersect_behind() {
    let r = Ray::new(Point::ONE, Direction::Z);
    assert!(plane().hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn intersects() {
    let r = Ray::new(Point::NEG_ONE, Direction::Z);
    assert!(plane().hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn no_hit_crash_parallel() {
    let p = plane();
    let r = Ray::new(
        Point::NEG_ONE,
        Point::X,
    );
    assert!(p.hit(&r, 0.0, crate::INF).is_none());
}
