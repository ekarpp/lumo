use super::*;

fn xy_rect() -> Box<Rectangle> {
    Rectangle::new(
        DMat3::from_cols(
            DVec3::Z,
            DVec3::Z + DVec3::X,
            DVec3::ONE
        ),
        Material::Mirror,
    )
}

#[test]
fn does_intersect() {
    let rect = xy_rect();
    let r = Ray::new(DVec3::ZERO, DVec3::Z);

    assert!(rect.hit(&r, 0.0, INFINITY).is_some());
}

#[test]
fn no_hit_behind() {
    let rect = xy_rect();
    let r = Ray::new(DVec3::ZERO, DVec3::NEG_Z);

    assert!(rect.hit(&r, 0.0, INFINITY).is_none());
}
