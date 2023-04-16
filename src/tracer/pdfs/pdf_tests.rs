use super::*;

#[test]
fn reject_bad_mfd_samples() {
    let pdf = MfdPdf::new(
        DVec3::Z,
        DVec3::Z,
        DVec3::ONE,
        MfDistribution::diffuse(),
    );

    // cos hemisphere sampling
    assert!(pdf.sample_cos_hemisphere_pdf(DVec3::Y) == 0.0);

    // bad internal reflection
    assert!(pdf.sample_ndf_refract_pdf(DVec3::Z) == 0.0);

    // bad refraction
    assert!(pdf.sample_ndf_refract_pdf(DVec3::Y) == 0.0);
}
