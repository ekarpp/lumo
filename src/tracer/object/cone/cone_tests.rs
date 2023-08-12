use super::*;

fn cone() -> Box<Cone> {
    Cone::new(1.0, 0.1, Material::Blank)
}

#[test]
fn no_self_intersect() {
    let c = cone();
    let xo = 0.1 * DVec3::Z;
    let r = Ray::new(xo, xo);

    assert!(c.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn no_intersect_behind() {
    let c = cone();
    let r = Ray::new(DVec3::Z, DVec3::Z);

    assert!(c.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn does_intersect() {
    let c = cone();
    let r = Ray::new(DVec3::Z + 0.5 * DVec3::Y, DVec3::NEG_Z);

    assert!(c.hit(&r, 0.0, INFINITY).is_some());
}
