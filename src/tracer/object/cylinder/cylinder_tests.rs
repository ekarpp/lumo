use super::*;

fn cylinder(r: f64) -> Box<Cylinder> {
    Cylinder::new(1.0, r, Material::Blank)
}

#[test]
fn no_self_intersect() {
    let c = cylinder(0.1);
    let xo = 0.1 * DVec3::Z;
    let r = Ray::new(xo, xo);

    assert!(c.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn no_intersect_behind() {
    let c = cylinder(0.1);
    let r = Ray::new(DVec3::Z, DVec3::Z);

    assert!(c.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn does_intersect() {
    let c = cylinder(0.1);
    let r = Ray::new(DVec3::Z + 0.5 * DVec3::Y, DVec3::NEG_Z);

    assert!(c.hit(&r, 0.0, INFINITY).is_some());
}

#[test]
fn coplanar_misses() {
    let radius = 0.1;
    let c = cylinder(radius);
    let r = Ray::new(radius * DVec3::ONE, DVec3::Y);

    assert!(c.hit(&r, 0.0, INFINITY).is_none());
}


#[test]
fn passes_through_middle() {
    let radius = 1.0;
    let c = cylinder(radius);

    let xo = DVec3::new(-0.9, -0.1, 0.0);
    let wi = DVec3::new(0.1, 0.9, 0.0);

    let r = Ray::new(xo, wi);

    assert!(c.hit(&r, 0.0, INFINITY).is_none());
}
