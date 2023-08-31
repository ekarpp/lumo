use crate::{Direction, Normal, Float, Transport};
use crate::tracer::{
    color::Color, hit::Hit, ray::Ray,
    microfacet::MfDistribution,
    pdfs::{
        DeltaPdf, MfdPdf,
        Pdf, VolumetricPdf
    }
};

/// BSDF for microfacet. Works for transparent and non-transparent materials.
///
/// # Arguments
/// * `wo` - Incoming direction to the point of impact
/// * `wi` - Direction towards "light" from the point of impact
/// * `ng` - Geometric normal of the surface at the point of impact
/// * `mode` - Toggle between radiance and importance transport
/// * `albedo` - Albedo of the material at the point of impact
/// * `mfd` - Microfacet distribution of the material
pub fn bsdf_microfacet(
    wo: Direction,
    wi: Direction,
    ng: Normal,
    ns: Normal,
    mode: Transport,
    albedo: Color,
    mfd: &MfDistribution
) -> Color {
    let v = -wo;
    // abs these, for refraction it makes no difference
    // for reflection they might cause negative values when grazing ng
    let ns_dot_wi = ns.dot(wi).abs();
    let ns_dot_v = ns.dot(v).abs();

    let rfrct_idx = mfd.get_rfrct_idx();

    let ro_inside = ng.dot(v) < 0.0;
    let ri_inside = ng.dot(wi) < 0.0;
    if ro_inside == ri_inside {
        let wh = (wi + v).normalize();

        let d = mfd.d(wh, ns);
        let f = if mfd.is_transparent() && ri_inside {
            let wh_dot_v = wh.dot(v);
            let sin2_to = 1.0 - wh_dot_v * wh_dot_v;
            let sin2_ti = sin2_to * mfd.get_rfrct_idx() * mfd.get_rfrct_idx();

            if sin2_ti > 1.0 {
                // total internal reflection
                Color::WHITE
            } else {
                mfd.f(v, wh, albedo)
            }
        } else {
            mfd.f(v, wh, albedo)
        };
        let g = mfd.g(v, wi, wh, ns);

        // BRDF: specular + diffuse, where
        // specular = D(wh) * F(v, wh) * G(v, wi) / (4.0 * (no • v) * (no • wi))
        // diffuse = normalized_disney_term * albedo / π
        // normalized_disney_term = (1.0 + α^2 * (1.0 / 1.51 - 1.0))
        // * (1.0 + (F_90 - 1.0) * (1.0 - (no • v))^5)
        // * (1.0 + (F_90 - 1.0) * (1.0 - (no • wi))^5)
        // F_90 = 0.5 * α^2 + 2.0 * (no • wh)^2 * α^2

        let specular = d * f * g / (4.0 * ns_dot_v * ns_dot_wi);

        // transparent materials don't have a diffuse term
        if mfd.is_transparent() {
            specular
        } else {
            let ns_dot_wh = ns.dot(wh);
            let diffuse = (Color::WHITE - f) * albedo
                * mfd.disney_diffuse(ns_dot_v, ns_dot_wh, ns_dot_wi) / crate::PI;

            diffuse + specular
        }
    } else {
        let eta_ratio = if ro_inside {
            1.0 / rfrct_idx
        } else {
            rfrct_idx
        };
        let scale = match mode {
            Transport::Radiance => eta_ratio * eta_ratio,
            Transport::Importance => 1.0,
        };

        let wh = (wi * eta_ratio + v).normalize();
        let wh = if wh.dot(v) < 0.0 { -wh } else { wh };

        let wh_dot_wi = wh.dot(wi);
        let wh_dot_v = wh.dot(v);

        let d = mfd.d(wh, ns);
        let f = mfd.f(v, wh, albedo);
        let g = mfd.g(v, wi, wh, ns);

        // BTDF:
        // albedo * abs[(wh • wi) * (wh • v)/((no • wi) * (no • v))]
        // * D(wh) * (1 - F(v, wh)) * G(v, wi) /  (η_r * (wh • wi) + (wh • v))^2

        scale * (wh_dot_wi * wh_dot_v / (ns_dot_wi * ns_dot_v)).abs()
            * albedo * d * (Color::WHITE - f) * g
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
    albedo: Color,
    mfd: &MfDistribution,
) -> Option<Box<dyn Pdf>> {
    let ns = ho.ns;
    let ng = ho.ng;
    let v = -ro.dir;
    Some( Box::new(MfdPdf::new(v, ns, ng, albedo, *mfd)) )
}

/// Scattering function for mirror material. Perfect reflection.
pub fn brdf_mirror_pdf(ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
    let wo = ro.dir;
    let no = ho.ns;
    let wi = reflect(-wo, no);
    Some( Box::new(DeltaPdf::new(wi)) )
}

pub fn brdf_volumetric_pdf(ro: &Ray, g: Float) -> Option<Box<dyn Pdf>> {
    let v = -ro.dir;
    Some( Box::new(VolumetricPdf::new(v, g)) )
}

pub fn btdf_glass_pdf(ho: &Hit, ro: &Ray, rfrct_idx: Float) -> Option<Box<dyn Pdf>> {
    let ng = ho.ng;
    let v = -ro.dir;
    let inside = ng.dot(v) < 0.0;
    let eta_ratio = if inside { rfrct_idx } else { 1.0 / rfrct_idx };
    let ns = if inside { -ho.ns } else { ho.ns };

    let wi = refract(eta_ratio, v, ns);

    Some( Box::new(DeltaPdf::new(wi)) )
}

/// Reflect around normal
///
/// # Arguments
/// * `v` - Normalized? direction from reflection point to viewer
/// * `no` - Surface normal
pub fn reflect(v: Direction, no: Normal) -> Direction {
    2.0 * v.project_onto(no) - v
}

/// Refract direction with Snell-Descartes law.
///
/// # Arguments
/// * `eta_ratio` - Ratio of refraction indices. `from / to`
/// * `v` - Normalized direction from refraction point to viewer
/// * `no` - Surface normal, pointing to same hemisphere as `v`
pub fn refract(eta_ratio: Float, v: Direction, no: Normal) -> Direction {
    /* Snell-Descartes law */
    let cos_to = no.dot(v);
    let sin2_to = 1.0 - cos_to * cos_to;
    let sin2_ti = eta_ratio * eta_ratio * sin2_to;

    /* total internal reflection */
    if sin2_ti > 1.0 {
        reflect(v, no)
    } else {
        let cos_ti = (1.0 - sin2_ti).sqrt();

        -v * eta_ratio + (eta_ratio * cos_to - cos_ti) * no
    }
}
