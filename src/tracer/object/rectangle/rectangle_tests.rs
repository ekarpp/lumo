use super::*;

const NUM_RAYS: usize = 10000;

fn xy_rect() -> Box<Rectangle> {
    Rectangle::new(
        DMat3::from_cols(
            DVec3::Z,
            DVec3::Z + DVec3::X,
            DVec3::ONE
        ),
        Material::Blank,
    )
}

#[test]
fn does_intersect() {
    let rect = xy_rect();
    let r = Ray::new(DVec3::splat(0.1), DVec3::Z);

    assert!(rect.hit(&r, 0.0, INFINITY).is_some());
}

#[test]
fn no_hit_behind() {
    let rect = xy_rect();
    let r = Ray::new(DVec3::ZERO, DVec3::NEG_Z);

    assert!(rect.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn sampled_rays_hit() {
    let rect = xy_rect();
    let xo = 5.0 * DVec3::Z;

    for _ in 0..NUM_RAYS {
        let wi = rect.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let (p, _) = rect.sample_towards_pdf(&ri);

        assert!(p > 0.0);
    }
}
