use super::*;

const NUM_RAYS: usize = 10000;

fn xy_rect() -> Box<Rectangle> {
    Rectangle::new(
        Mat3::from_cols(
            Point::Z,
            Point::Z + Point::X,
            Point::ONE,
        ),
        Material::Blank,
    )
}

#[test]
fn does_intersect() {
    let rect = xy_rect();
    let r = Ray::new(Point::splat(0.1), Direction::Z);

    assert!(rect.hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn no_hit_behind() {
    let rect = xy_rect();
    let r = Ray::new(Point::ZERO, Direction::NEG_Z);

    assert!(rect.hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn sampled_rays_hit() {
    let rect = xy_rect();
    let xo = 5.0 * Point::Z;

    for _ in 0..NUM_RAYS {
        let wi = rect.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let Some(hi) = rect.hit(&ri, 0.0, crate::INF) else { panic!() };
        let p = rect.sample_towards_pdf(&ri, hi.p, hi.ng);

        assert!(p > 0.0);
    }
}
