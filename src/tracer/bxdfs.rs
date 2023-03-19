use crate::DVec3;
use crate::pdfs::{Pdf, CosPdf, DeltaPdf, IsotropicPdf, MfdPdf};
use std::f64::consts::PI;
use crate::consts::{EPSILON, ETA};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::microfacet::MfDistribution;

/// Shading for microfacet. Computed as diffuse + specular, where (`D`, `F`, `G` values from the microfacet distribution):
///
/// `specular = D(wh) * F(wo, wh) * G(wo, wi) / (4.0 * (n • wo) * (n • wi))`
/// `diffuse = (1 - F(wo, wh)) * albedo / π`
pub fn brdf_microfacet(
    ro: &Ray,
    ri: &Ray,
    no: DVec3,
    color: DVec3,
    mfd: &MfDistribution,
) -> DVec3 {
    let wo = -ro.dir.normalize();
    let wi = ri.dir.normalize();
    let wh = (wi + wo).normalize();
    let met = 1.0;

    let d = mfd.d(wh, no);
    let f = mfd.f(wo, wh, color, met, ETA);
    let g = mfd.g(wo, wi, no);

    let no_dot_wi = no.dot(wi);
    let no_dot_wo = no.dot(wo);

    (DVec3::ONE - f) * color / PI + d * f * g / (4.0 * no_dot_wo * no_dot_wi)
}

/// Scattering function for diffuse material.
///
/// # Arguments
/// * `ho` - The hit from which we scatter.
/// * `ro` - Incoming ray to the hit point.
pub fn bsdf_diffuse_pdf(ho: &Hit, _ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    let no = ho.norm;
    Some( Box::new(CosPdf::new(xo, no)) )
}

/// Scattering function for microfacet surfaces
///
/// # Arguments
/// * `ho` - Hit to scatter from
/// * `ro` - Ray from viewer.
/// * `mfd` - The microfacet distribution of the surface
pub fn bsdf_microfacet_pdf(ho: &Hit, ro: &Ray, mfd: &MfDistribution)
                           -> Option<Box<dyn Pdf>> {
    let no = ho.norm;
    let xo = ho.p;
    let wo = -ro.dir;
    Some( Box::new(MfdPdf::new(xo, wo, no, *mfd)) )
}

/// TODO
pub fn bsdf_isotropic_pdf(ho: &Hit, _ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    Some( Box::new(IsotropicPdf::new(xo)) )
}

/// Scattering function for mirror material. Perfect reflection.
pub fn bsdf_mirror_pdf(ho: &Hit, _ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    let no = ho.norm;
    let wi = xo - 2.0 * xo.project_onto(no);
    Some( Box::new(DeltaPdf::new(xo, wi)) )
}

/// Scattering function for glass material.
/// Refracts according to Snell-Descartes law.
pub fn bsdf_glass_pdf(ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
    let inside = ho.object.inside(ro.origin + EPSILON*ro.dir);
    let eta_ratio = if inside { ETA } else { ETA.recip() };
    let no = if inside { -ho.norm } else { ho.norm };
    let xo = ho.p;

    /* Snell-Descartes law */
    let wo = ro.dir.normalize();
    let cos_in = no.dot(-wo).min(1.0);
    let sin_out = (1.0 - cos_in * cos_in) * eta_ratio * eta_ratio;

    /* total reflection */
    if sin_out > 1.0 {
        return bsdf_mirror_pdf(ho, ro);
    }

    let wi = eta_ratio * wo + no *
        (eta_ratio * cos_in - (1.0 - sin_out).sqrt());

    Some( Box::new(DeltaPdf::new(xo, wi)) )
}
