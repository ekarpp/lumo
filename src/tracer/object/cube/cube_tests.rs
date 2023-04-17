use super::*;

#[test]
fn does_intersect() {
    let cube = Cube::new(Material::Mirror);
    let r = Ray::new(10.0 * DVec3::ONE, DVec3::NEG_ONE);

    assert!(cube.hit(&r, 0.0, INFINITY).is_some());
}

#[test]
fn no_intersect_behind() {
    let cube = Cube::new(Material::Mirror);
    let r = Ray::new(10.0 * DVec3::ONE, DVec3::ONE);

    assert!(cube.hit(&r, 0.0, INFINITY).is_none());
}
