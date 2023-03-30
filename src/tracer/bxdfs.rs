use crate::tracer::hit::Hit;
use crate::tracer::microfacet::MfDistribution;
use crate::tracer::pdfs::{DeltaPdf, IsotropicPdf, MfdPdf, Pdf};
use crate::tracer::ray::Ray;
use glam::DVec3;
use std::f64::consts::PI;

/// BSDF for microfacet. Works for transparent and non-transparent materials.
pub fn bsdf_microfacet(ro: &Ray, ri: &Ray, no: DVec3, color: DVec3, mfd: &MfDistribution) -> DVec3 {
    let v = -ro.dir;
    let wi = ri.dir;
    let no_dot_wi = no.dot(wi);
    let no_dot_v = no.dot(v);

    let rfrct_idx = mfd.get_rfrct_idx();

    let ro_inside = no_dot_v < 0.0;
    let ri_inside = no_dot_wi < 0.0;
    if ro_inside == ri_inside {
        let wh = (wi + v).normalize();
        let wh_dot_v = wh.dot(v);
        let no_dot_wh = no.dot(wh);

        let d = mfd.d(wh, no);
        let f = if ro_inside && (1.0 - wh_dot_v.powi(2)) * rfrct_idx.powi(2) > 1.0 {
            DVec3::ONE
        } else {
            mfd.f(v, wh, color)
        };
        let g = mfd.g(v, wi, no);

        // BRDF: specular + diffuse, where
        // specular = D(wh) * F(v, wh) * G(v, wi) / (4.0 * (no • v) * (no • wi))
        // diffuse = normalized_disney_term * albedo / π
        // normalized_disney_term = (1.0 + α^2 * (1.0 / 1.51 - 1.0))
        // * (1.0 + (F_90 - 1.0) * (1.0 - (no • v))^5)
        // * (1.0 + (F_90 - 1.0) * (1.0 - (no • wi))^5)
        // F_90 = 0.5 * α^2 + 2.0 * (no • wh)^2 * α^2

        let specular = d * f * g / (4.0 * no_dot_v * no_dot_wi);

        // transparent materials don't have a diffuse term
        if mfd.is_transparent() {
            specular
        } else {
            let diffuse =
                (DVec3::ONE - f) * color * mfd.disney_diffuse(no_dot_v, no_dot_wh, no_dot_wi) / PI;

            diffuse + specular
        }
    } else {
        let eta_ratio = if ro_inside {
            1.0 / rfrct_idx
        } else {
            rfrct_idx
        };

        let wh = (wi * eta_ratio + v).normalize();
        let wh = if wh.dot(v) < 0.0 { -wh } else { wh };

        let wh_dot_wi = wh.dot(wi);
        let wh_dot_v = wh.dot(v);

        let d = mfd.d(wh, no);
        let f = mfd.f(v, wh, color);
        let g = mfd.g(v, wi, no);

        // BTDF:
        // albedo * abs[(wh • wi) * (wh • v)/((no • wi) * (no • v))]
        // * D(wh) * (1 - F(v, wh)) * G(v, wi) /  (η_r * (wh • wi) + (wh • v))^2

        (wh_dot_wi * wh_dot_v / (no_dot_wi * no_dot_v)).abs()
            * color * d * (DVec3::ONE - f) * g
            / (eta_ratio * wh_dot_wi + wh_dot_v).powi(2)
    }
}

/// Scattering function for microfacet surfaces
///
/// # Arguments
/// * `ho` - Hit to scatter from
/// * `ro` - Ray from viewer.
/// * `mfd` - The microfacet distribution of the surface
pub fn bsdf_microfacet_pdf(
    ho: &Hit,
    ro: &Ray,
    albedo: DVec3,
    mfd: &MfDistribution,
) -> Option<Box<dyn Pdf>> {
    let no = ho.norm;
    let xo = ho.p;
    let v = -ro.dir;
    Some( Box::new(MfdPdf::new(xo, v, no, albedo, *mfd)) )
}

/// TODO
pub fn bsdf_isotropic_pdf(ho: &Hit, _ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    Some(Box::new(IsotropicPdf::new(xo)))
}

/// Scattering function for mirror material. Perfect reflection.
pub fn brdf_mirror_pdf(ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    let wo = ro.dir;
    let no = ho.norm;
    let wi = reflect(-wo, no);
    Some(Box::new(DeltaPdf::new(xo, wi)))
}

pub fn btdf_glass_pdf(ho: &Hit, ro: &Ray, rfrct_idx: f64) -> Option<Box<dyn Pdf>> {
    let no = ho.norm;
    let v = -ro.dir;
    let inside = no.dot(v) < 0.0;
    let eta_ratio = if inside { rfrct_idx } else { 1.0 / rfrct_idx };
    let no = if inside { -ho.norm } else { ho.norm };
    let xo = ho.p;

    let wi = refract(eta_ratio, v, no);
    Some( Box::new(DeltaPdf::new(xo, wi)) )
}

/// Reflect around normal
///
/// # Arguments
/// * `v` - Normalized? direction from reflection point to viewer
/// * `no` - Surface normal
pub fn reflect(v: DVec3, no: DVec3) -> DVec3 {
    2.0 * v.project_onto(no) - v
}

/// Refract direction with Snell-Descartes law.
///
/// # Arguments
/// * `eta_ratio` - Ratio of refraction indices. `from / to`
/// * `v` - Normalized direction from refraction point to viewer
/// * `no` - Surface normal, pointing to same hemisphere as `v`
pub fn refract(eta_ratio: f64, v: DVec3, no: DVec3) -> DVec3 {
    /* Snell-Descartes law */
    let cos_to = no.dot(v);
    let sin2_to = 1.0 - cos_to * cos_to;
    let sin2_ti = eta_ratio * eta_ratio * sin2_to;

    /* total internal reflection */
    if sin2_ti > 1.0 {
        return reflect(v, no);
    }

    let cos_ti = (1.0 - sin2_ti).sqrt();

    -v * eta_ratio + (eta_ratio * cos_to - cos_ti) * no
}
