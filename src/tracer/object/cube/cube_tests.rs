use super::*;

const NUM_RAYS: usize = 10000;

#[test]
fn sampled_rays_hit() {
    let c = Cube::new(Material::mirror());
    let xo = 3.0 * rand_utils::square_to_sphere(rand_utils::unit_square());

    for _ in 0..NUM_RAYS {
        let wi = c.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let Some(hi) = c.hit(&ri, 0.0, crate::INF) else { panic!() };

        let p = c.sample_towards_pdf(&ri, hi.p, hi.ng);
        assert!(p > 0.0);
    }
}

#[test]
fn does_intersect() {
    let cube = Cube::new(Material::mirror());
    let r = Ray::new(10.0 * Point::ONE, Direction::NEG_ONE);

    assert!(cube.hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn no_intersect_behind() {
    let cube = Cube::new(Material::mirror());
    let r = Ray::new(10.0 * Point::ONE, Direction::ONE);

    assert!(cube.hit(&r, 0.0, crate::INF).is_none());
}
