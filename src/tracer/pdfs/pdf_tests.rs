use super::*;

fn mfd_pdf() -> MfdPdf {
    MfdPdf::new(
        DVec3::Z,
        DVec3::Z,
        DVec3::ONE,
        MfDistribution::specular(1.0),
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
