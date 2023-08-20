use super::*;

fn cylinder(r: Float) -> Box<Cylinder> {
    Cylinder::new(1.0, r, Material::Blank)
}

#[test]
fn no_self_intersect() {
    let c = cylinder(0.1);
    let xo = 0.1 * Point::Z;
    let r = Ray::new(xo, xo);

    assert!(c.hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn no_intersect_behind() {
    let c = cylinder(0.1);
    let r = Ray::new(Point::Z, Point::Z);

    assert!(c.hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn does_intersect() {
    let c = cylinder(0.1);
    let r = Ray::new(Point::Z + 0.5 * Point::Y, Point::NEG_Z);

    assert!(c.hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn coplanar_misses() {
    let radius = 0.1;
    let c = cylinder(radius);
    let r = Ray::new(radius * Point::ONE, Point::Y);

    assert!(c.hit(&r, 0.0, crate::INF).is_none());
}


#[test]
fn passes_through_middle() {
    let radius = 1.0;
    let c = cylinder(radius);

    let xo = Point::new(-0.9, -0.1, 0.0);
    let wi = Point::new(0.1, 0.9, 0.0);

    let r = Ray::new(xo, wi);

    assert!(c.hit(&r, 0.0, crate::INF).is_none());
}
