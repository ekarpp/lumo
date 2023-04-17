use super::*;

const NUM_SAMPLES: usize = 10000;

#[test]
fn sampling_equals_plain_object() {
    let ref_sphere = Sphere::new(DVec3::Z, 1.0, Material::Blank);
    let sphere = Sphere::new(DVec3::ZERO, 0.1, Material::Blank)
        .rotate_x(PI)
        .scale(10.0, 10.0, 10.0)
        .rotate_y(PI)
        .rotate_z(PI)
        .translate(0.0, 0.0, 1.0);

    let xo = DVec3::NEG_Z;

    for _ in 0..NUM_SAMPLES {
        let wi = sphere.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let (ref_p, _) = ref_sphere.sample_towards_pdf(&ri);
        let (p, _) = sphere.sample_towards_pdf(&ri);

        assert!((ref_p - p).abs() < 1e-10);
    }
}

#[test]
fn sampled_direction_hits() {
    let sphere = Sphere::new(DVec3::ZERO, 0.1, Material::Blank)
        .translate(0.123, 0.456, 0.789)
        .scale(2.0, 2.0, 2.0)
        .rotate_x(PI);


    let xo = DVec3::NEG_Z;

    for _ in 0..NUM_SAMPLES {
        let wi = sphere.sample_towards(xo, rand_utils::unit_square());
        let ri = Ray::new(xo, wi);
        let (p, _) = sphere.sample_towards_pdf(&ri);

        assert!(p > 0.0);
    }
}
