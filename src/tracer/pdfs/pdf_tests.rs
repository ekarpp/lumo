use super::*;
use crate::tracer::object::Sphere;
use crate::tracer::material::Material;

fn mfd_pdf() -> MfdPdf {
    MfdPdf::new(
        DVec3::Z,
        DVec3::Z,
        DVec3::ONE,
        MfDistribution::new(1.0, 1.5, 0.0, false),
    )
}

#[test]
fn reject_bad_mfd_hemisphere() {
    let pdf = mfd_pdf();
    // cos hemisphere sampling
    assert!(pdf.sample_cos_hemisphere_pdf(DVec3::Y) == 0.0);
}

#[test]
fn reject_bad_mfd_refracts() {
    let pdf = mfd_pdf();

    // bad internal reflection
    assert!(pdf.sample_ndf_refract_pdf(DVec3::Z) == 0.0);

    // bad refraction
    assert!(pdf.sample_ndf_refract_pdf(DVec3::Y) == 0.0);
}

#[test]
fn delta_value_for() {
    let pdf = DeltaPdf::new(DVec3::Z);

    let r = Ray::new(DVec3::ZERO, DVec3::Z + 1e-5 * DVec3::X);
    assert!(pdf.value_for(&r) == 1.0);

    let r = Ray::new(DVec3::ZERO, DVec3::NEG_Z);
    assert!(pdf.value_for(&r) == 0.0);
}

#[test]
fn object_pdf_returns_solid_angle() {
    let sphere = Sphere::new(DVec3::ZERO, 1.0, Material::Blank);
    let xo = 3.0 * DVec3::Z;
    let wi = DVec3::NEG_Z;
    let ri = Ray::new(xo, wi);
    let (pa, hi) = sphere.sample_towards_pdf(&ri);
    assert!(hi.is_some());

    let hi = hi.unwrap();
    let pa = pa * xo.distance_squared(hi.p) / hi.ng.dot(wi).abs();

    let pdf = ObjectPdf::new(&*sphere, xo);
    let psa = pdf.value_for(&ri);

    assert!((pa - psa).abs() < 1e-15);
}
