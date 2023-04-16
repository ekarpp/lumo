use super::*;

const NUM_RAYS: usize = 10000;

fn unit_sphere() -> Box<Sphere> {
    Sphere::new(DVec3::ZERO, 1.0, Material::Blank)
}

#[test]
fn sampled_rays_hit() {
    let s = unit_sphere();

    let xo = 5.0 * DVec3::Z;

    for _ in 0..NUM_RAYS {
        let wi = s.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let (p, _) = s.sample_towards_pdf(&ri);

        assert!(p > 0.0);
    }
}

#[test]
fn no_self_intersect() {
    let s = unit_sphere();
    let r = Ray::new(DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 1.0, 0.0));
    assert!(s.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn no_intersect_behind() {
    let s = unit_sphere();

    let r = Ray::new(DVec3::new(1.5, 0.0, 0.0), DVec3::new(1.0, 0.0, 0.0));
    assert!(s.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn does_intersect() {
    let s = unit_sphere();
    let v = DVec3::new(12.3, 45.6, 78.9);
    let r = Ray::new(v, -v);
    assert!(s.hit(&r, 0.0, INFINITY).is_some());
}
