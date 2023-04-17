use super::*;

const NUM_RAYS: usize = 10000;

fn unit_disk() -> Box<Disk> {
    Disk::new(DVec3::ZERO, DVec3::Z, 1.0, Material::Mirror)
}

#[test]
fn does_intersect() {
    let d = unit_disk();
    let r = Ray::new(3.0 * DVec3::ONE, DVec3::NEG_ONE);

    assert!(d.hit(&r, 0.0, INFINITY).is_some());
}

#[test]
fn no_intersect_behind() {
    let d = unit_disk();
    let r = Ray::new(2.0 * DVec3::ONE, DVec3::ONE);

    assert!(d.hit(&r, 0.0, INFINITY).is_none());
}

#[test]
fn sampled_rays_hit() {
    let d = unit_disk();

    let xo = 5.0 * DVec3::ONE;

    for _ in 0..NUM_RAYS {
        let wi = d.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let (p, _) = d.sample_towards_pdf(&ri);

        assert!(p > 0.0);
    }
}
