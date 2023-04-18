use super::*;
use crate::tracer::texture::Texture;

const NUM_RAYS: usize = 10000;

fn get_mat() -> Material {
    Material::diffuse(Texture::Solid(DVec3::ONE))
}

#[test]
fn point_order_irrelevant() {
    let pts = vec![
        DVec3::new(-0.3, -0.3, -1.0),
        DVec3::new(0.3, -0.3, -1.0),
        DVec3::new(0.0, 0.0, -1.0),
    ];

    let r = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, -1.0));

    (0..6).for_each(|i| {
        let a = i / 2;
        let b = i % 2;

        let t = Triangle::new(
            /* permutations */
            DMat3::from_cols(pts[a], pts[(a + b + 1) % 3], pts[2 - (i % 3)]),
            None,
            None,
            get_mat(),
        );
        assert!(t.hit(&r, 0.0, INFINITY).is_some());
    });
}

#[test]
fn no_self_intersect() {
    let abc = DMat3::from_cols(
        DVec3::new(-5.0, 0.0, -3.0),
        DVec3::new(5.0, 0.0, -3.0),
        DVec3::new(5.0, 0.0, 3.0),
    );

    let t = Triangle::new(abc, None, None, get_mat());

    let r = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, -1.0));
    assert!(t.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn sampled_rays_hit() {
    let abc = DMat3::from_cols(
        DVec3::ZERO,
        DVec3::X,
        DVec3::X + DVec3::Y,
    );
    let tri = Triangle::new(abc, None, None, get_mat());

    let xo = 5.0 * DVec3::Z;

    for _ in 0..NUM_RAYS {
        let wi = tri.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let (p, _) = tri.sample_towards_pdf(&ri);

        assert!(p > 0.0);
    }
}
