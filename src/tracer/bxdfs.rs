use crate::pdfs::{Pdf, CosPdf, DeltaPdf};
use crate::consts::{EPSILON, ETA};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

/// Scattering function for diffuse material.
/// # Arguments
/// * `h` - The hit from which we scatter.
/// * `r` - Incoming ray to the hit point.
pub fn bsdf_diffuse_sample(ho: &Hit, _ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    let no = ho.norm;
    Some( Box::new(CosPdf::new(xo, no)) )
}

/// Scattering function for mirror material. Perfect reflection.
pub fn bsdf_mirror_sample(ho: &Hit, _ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    let no = ho.norm;
    let wi = xo - 2.0 * xo.project_onto(no);
    Some( Box::new(DeltaPdf::new(xo, wi)) )
}

/// Scattering function for glass material.
/// Refracts according to Snell-Descartes law.
pub fn bsdf_glass_sample(ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
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
        return bsdf_mirror_sample(ho, ro);
    }

    let wi = eta_ratio * wo + no *
        (eta_ratio * cos_in - (1.0 - sin_out).sqrt());

    Some( Box::new(DeltaPdf::new(xo, wi)) )
}
