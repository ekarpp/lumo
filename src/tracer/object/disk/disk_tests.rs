use super::*;

const NUM_RAYS: usize = 10000;

fn unit_disk() -> Box<Disk> {
    Disk::new(Point::ZERO, Point::Z, 1.0, Material::mirror())
}

#[test]
fn does_intersect() {
    let d = unit_disk();
    let r = Ray::new(3.0 * Point::ONE, Point::NEG_ONE);

    assert!(d.hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn no_intersect_behind() {
    let d = unit_disk();
    let r = Ray::new(2.0 * Point::ONE, Point::ONE);

    assert!(d.hit(&r, 0.0, crate::INF).is_none());
}

#[test]
fn sampled_rays_hit() {
    let d = unit_disk();

    let xo = 5.0 * Point::ONE;

    for _ in 0..NUM_RAYS {
        let wi = d.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let Some(hi) = d.hit(&ri, 0.0, crate::INF) else { panic!() };
        let p = d.sample_towards_pdf(&ri, hi.p, hi.ng);

        assert!(p > 0.0);
    }
}
