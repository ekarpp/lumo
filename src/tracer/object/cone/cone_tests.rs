use super::*;

fn cone() -> Box<Cone> {
    Cone::new(1.0, 0.1, Material::Blank)
}

#[test]
fn no_self_intersect() {
    let c = cone();
    let xo = 0.1 * Point::Z;
    let r = Ray::new(xo, xo);

    assert!(c.hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn no_intersect_behind() {
    let c = cone();
    let r = Ray::new(Point::Z, Point::Z);

    assert!(c.hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn does_intersect() {
    let c = cone();
    let r = Ray::new(Point::Z + 0.5 * Point::Y, Point::NEG_Z);

    assert!(c.hit(&r, 0.0, crate::INF).is_some());
}
