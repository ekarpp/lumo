use super::*;

#[test]
fn does_intersect() {
    let cube = Cube::new(Material::Mirror);
    let r = Ray::new(10.0 * Point::ONE, Direction::NEG_ONE);

    assert!(cube.hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn no_intersect_behind() {
    let cube = Cube::new(Material::Mirror);
    let r = Ray::new(10.0 * Point::ONE, Direction::ONE);

    assert!(cube.hit(&r, 0.0, crate::INF).is_none());
}
