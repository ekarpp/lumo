use crate::tracer::hit::Hit;
use crate::tracer::microfacet::MfDistribution;
use crate::tracer::pdfs::{DeltaPdf, MfdPdf, Pdf};
use crate::tracer::ray::Ray;
use glam::DVec3;
use std::f64::consts::PI;

/// BSDF for microfacet. Works for transparent and non-transparent materials.
///
/// # Arguments
/// * `ro` - Ray incoming to the point of impact
/// * `ri` - Ray towards "light" from the point of impact
/// * `ng` - Geometric normal of the surface at the point of impact
/// * `albedo` - Albedo of the material at the point of impact
/// * `mfd` - Microfacet distribution of the material
pub fn bsdf_microfacet(
    ro: &Ray,
    ri: &Ray,
    ng: DVec3,
    albedo: DVec3,
    mfd: &MfDistribution
) -> DVec3 {
    let v = -ro.dir;
    let wi = ri.dir;
    let ng_dot_wi = ng.dot(wi);
    let ng_dot_v = ng.dot(v);

    let rfrct_idx = mfd.get_rfrct_idx();

    let ro_inside = ng.dot(v) < 0.0;
    let ri_inside = ng.dot(wi) < 0.0;
    if ro_inside == ri_inside {
        let wh = (wi + v).normalize();
        let ng_dot_wh = ng.dot(wh);
        let wh_dot_v = wh.dot(v);

        let d = mfd.d(wh, ng);
        let sin2_to = 1.0 - wh_dot_v * wh_dot_v;
        let sin2_ti = sin2_to * mfd.get_rfrct_idx() * mfd.get_rfrct_idx();
        let f = if ri_inside && sin2_ti > 1.0 {
            DVec3::ONE
        } else {
            mfd.f(v, wh, albedo)
        };
        let g = mfd.g(v, wi, ng);

        // BRDF: specular + diffuse, where
        // specular = D(wh) * F(v, wh) * G(v, wi) / (4.0 * (no • v) * (no • wi))
        // diffuse = normalized_disney_term * albedo / π
        // normalized_disney_term = (1.0 + α^2 * (1.0 / 1.51 - 1.0))
        // * (1.0 + (F_90 - 1.0) * (1.0 - (no • v))^5)
        // * (1.0 + (F_90 - 1.0) * (1.0 - (no • wi))^5)
        // F_90 = 0.5 * α^2 + 2.0 * (no • wh)^2 * α^2

        let specular = d * f * g / (4.0 * ng_dot_v * ng_dot_wi);

        // transparent materials don't have a diffuse term
        if mfd.is_transparent() {
            specular
        } else {
            let diffuse = (DVec3::ONE - f) * albedo
                * mfd.disney_diffuse(ng_dot_v, ng_dot_wh, ng_dot_wi) / PI;

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

        let d = mfd.d(wh, ng);
        let f = mfd.f(v, wh, albedo);
        let g = mfd.g(v, wi, ng);

        // BTDF:
        // albedo * abs[(wh • wi) * (wh • v)/((no • wi) * (no • v))]
        // * D(wh) * (1 - F(v, wh)) * G(v, wi) /  (η_r * (wh • wi) + (wh • v))^2

        (wh_dot_wi * wh_dot_v / (ng_dot_wi * ng_dot_v)).abs()
            * albedo * d * (DVec3::ONE - f) * g
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
    let ng = ho.ng;
    let xo = ho.p;
    let v = -ro.dir;
    Some( Box::new(MfdPdf::new(xo, v, ng, albedo, *mfd)) )
}

/// Scattering function for mirror material. Perfect reflection.
pub fn brdf_mirror_pdf(ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
    let xo = ho.p;
    let wo = ro.dir;
    let no = ho.ng;
    let wi = reflect(-wo, no);
    Some(Box::new(DeltaPdf::new(xo, wi)))
}

pub fn btdf_glass_pdf(ho: &Hit, ro: &Ray, rfrct_idx: f64) -> Option<Box<dyn Pdf>> {
    let ng = ho.ng;
    let v = -ro.dir;
    let inside = ng.dot(v) < 0.0;
    let eta_ratio = if inside { rfrct_idx } else { 1.0 / rfrct_idx };
    let ng = if inside { -ng } else { ng };
    let xo = ho.p;

    let wi = match refract(eta_ratio, v, ng) {
        None => reflect(v, ng),
        Some(wi) => wi,
    };

    Some( Box::new(DeltaPdf::new(xo, wi)) )
}

/// Reflect around normal
///
/// # Arguments
/// * `v` - Normalized? direction from reflection point to viewer
/// * `no` - Surface normal
pub fn reflect(v: DVec3, ng: DVec3) -> DVec3 {
    2.0 * v.project_onto(ng) - v
}

/// Refract direction with Snell-Descartes law.
///
/// # Arguments
/// * `eta_ratio` - Ratio of refraction indices. `from / to`
/// * `v` - Normalized direction from refraction point to viewer
/// * `ng` - Surface geometric normal, pointing to same hemisphere as `v`
pub fn refract(eta_ratio: f64, v: DVec3, ng: DVec3) -> Option<DVec3> {
    /* Snell-Descartes law */
    let cos_to = ng.dot(v);
    let sin2_to = 1.0 - cos_to * cos_to;
    let sin2_ti = eta_ratio * eta_ratio * sin2_to;

    /* total internal reflection */
    if sin2_ti > 1.0 {
        return Some( reflect(v, ng) );
    }

    let cos_ti = (1.0 - sin2_ti).sqrt();

    Some( -v * eta_ratio + (eta_ratio * cos_to - cos_ti) * ng )
}
