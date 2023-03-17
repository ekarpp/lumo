use super::*;
use crate::tracer::texture::Texture;

fn get_mat() -> Material {
    Material::Diffuse(Texture::Solid(DVec3::ONE))
}

#[test]
fn point_order_irrelevant() {
    let pts = vec![
        DVec3::new(-0.3, -0.3, -1.0),
        DVec3::new(0.3, -0.3, -1.0),
        DVec3::new(0.0, 0.0, -1.0),
    ];

    let r = Ray::new(
        DVec3::ZERO,
        DVec3::new(0.0, 0.0, -1.0),
    );

    (0..6).for_each(|i| {
        let a = i / 2;
        let b = i % 2;

        let t = Triangle::new(
            /* permutations */
            pts[a],
            pts[(a + b + 1)%3],
            pts[2 - (i % 3)],
            DVec3::ONE,
            get_mat()
        );
        assert!(t.hit(&r, 0.0, INFINITY).is_some());
    });
}

#[test]
fn no_self_intersect() {
    let t = Triangle::new(
        DVec3::ZERO,
        DVec3::ONE,
        DVec3::new(1.0, 0.0, 1.0),
        DVec3::ONE,
        get_mat(),
    );
    let r = Ray::new(
        DVec3::ZERO,
        DVec3::new(0.0, 0.0, -1.0),
    );
    assert!(t.hit(&r, 0.0, INFINITY).is_none());
}
