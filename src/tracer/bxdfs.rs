use crate::DVec2;
use crate::pdfs::{Pdf, CosPdf};
use crate::consts::ETA;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

/// Scattering function for diffuse material.
/// # Arguments
/// * `h` - The hit from which we scatter.
/// * `r` - Incoming ray to the hit point.
pub fn bsdf_diffuse_sample(ho: &Hit, _ro: &Ray, rand_sq: DVec2)
                           -> Option<(Ray, f64)> {
    let xo = ho.p;
    let no = ho.norm;
    let pdf = CosPdf::new(no);
    let wi = pdf.generate_dir(rand_sq);
    Some( (Ray::new(xo, wi), pdf.pdf_val(wi)) )
}

/// Scattering function for mirror material. Perfect reflection.
pub fn bsdf_mirror_sample(ho: &Hit, _ro: &Ray) -> Option<(Ray, f64)> {
    let xo = ho.p;
    let no = ho.norm;
    let wi = xo - 2.0 * xo.project_onto(no);
    Some( (Ray::new(xo, wi), 1.0) )
}

/// Scattering function for glass material.
/// Refracts according to Snell-Descartes law.
pub fn bsdf_glass_sample(ho: &Hit, ro: &Ray) -> Option<(Ray, f64)> {
    let eta_ratio = if ho.object.inside(ro) { ETA } else { ETA.recip() };
    let no = if ho.object.inside(ro) { -ho.norm } else { ho.norm };
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

    Some( (Ray::new(xo, wi), 1.0) )
}
