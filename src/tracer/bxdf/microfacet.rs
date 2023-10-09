use super::*;

/*
 * MICROFACET TRANSMISSION
 */

pub fn transmission_f(
    wo: Direction,
    wi: Direction,
    mfd: &MfDistribution,
    albedo: Color,
    mode: Transport,
) -> Color {
    let v = -wo;
    let cos_theta_v = v.z;
    let cos_theta_wi = wi.z;

    /* on the same hemisphere */
    if cos_theta_v * cos_theta_wi > 0.0 {
        return Color::BLACK;
    }

    let eta_ratio = if cos_theta_v < 0.0 {
        1.0 / mfd.eta()
    } else {
        mfd.eta()
    };
    let scale = match mode {
        Transport::Radiance => eta_ratio * eta_ratio,
        Transport::Importance => 1.0,
    };
    let wh = (wi * eta_ratio + v).normalize();
    let wh = if wh.z < 0.0 { -wh } else { wh };

    let wh_dot_v = wh.dot(v);
    let wh_dot_wi = wh.dot(wi);


    /* same hemisphere w.r.t. wh */
    if wh_dot_v * wh_dot_wi > 0.0 {
        return Color::BLACK;
    }

    let d = mfd.d(wh, Normal::Z);
    let f = mfd.f(v, wh, albedo);
    let g = mfd.g(v, wi, wh, Normal::Z);

    scale * albedo * d * (Color::WHITE - f) * g
        * (wh_dot_wi * wh_dot_v / (cos_theta_wi * cos_theta_v)).abs()
        / (eta_ratio * wh_dot_wi + wh_dot_v).powi(2)
}

pub fn transmission_sample(
    wo: Direction,
    mfd: &MfDistribution,
    rand_sq: Vec2
) -> Option<Direction> {
    let v = -wo;
    let wh = mfd.sample_normal(v, rand_sq).normalize();
    let inside = v.z < 0.0;
    let eta_ratio = if inside {
        mfd.eta()
    } else {
        1.0 / mfd.eta()
    };

    // here we refract v around wh
    Some( Direction::Z )
}

pub fn transmission_pdf(
    wo: Direction,
    wi: Direction,
    mfd: &MfDistribution,
    swap_dir: bool,
) -> Float {
    let v = -wo;
    let (v, wi) = if swap_dir { (wi, v) } else { (v, wi) };
    let v_inside = v.z < 0.0;
    let wi_inside = wi.z < 0.0;


    if v_inside == wi_inside {
        /* same hemisphere. check total internal reflection. */
        let wh = (v + wi).normalize();
        let wh_dot_v = wh.dot(v);
        let sin2_to = 1.0 - wh_dot_v * wh_dot_v;
        let sin2_ti = sin2_to * mfd.eta().powi(2);

        if sin2_ti > 1.0 {
            mfd.sample_normal_pdf(wh, v, Normal::Z) / (4.0 * wh_dot_v)
        } else {
            /* not total internal reflection... */
            0.0
        }
    } else {
        let eta_ratio = if v_inside {
            1.0 / mfd.eta()
        } else {
            mfd.eta()
        };

        let wh = -(v + wi * eta_ratio).normalize();
        let wh_dot_wi = wi.dot(wh);
        let wh_dot_v = wh.dot(v);

        if wh_dot_wi * wh_dot_v > 0.0 {
            /* same hemisphere w.r.t. wh */
            0.0
        } else {
            mfd.sample_normal_pdf(wh, v, Normal::Z)
                * (eta_ratio * eta_ratio * wh_dot_wi).abs()
                / (wh_dot_v + eta_ratio * wh_dot_wi).powi(2)
        }
    }
}

/*
 * MICROFACET REFLECTION
 */

pub fn reflection_f(
    wo: Direction,
    wi: Direction,
    mfd: &MfDistribution,
    albedo: Color,
) -> Color {
    let v = -wo;
    let cos_theta_v = v.z;
    let cos_theta_wi = wi.z;
    let wh = (wi + v).normalize();

    let d = mfd.d(wh, Normal::Z);
    let f = mfd.f(v, wh, albedo);
    let g = mfd.g(v, wi, wh, Normal::Z);

    d * f * g / (4.0 * cos_theta_v * cos_theta_wi)
}

pub fn reflection_sample(
    wo: Direction,
    mfd: &MfDistribution,
    rand_sq: Vec2
) -> Option<Direction> {
    let v = -wo;
    let wh = mfd.sample_normal(v, rand_sq).normalize();
    // reflect v around wh
    let wi = Direction::Z;

    if wi.z <= 0.0 {
        // bad sample, do something else?
        None
    } else {
        Some( wi )
    }
}

pub fn reflection_pdf(
    wo: Direction,
    wi: Direction,
    mfd: &MfDistribution,
) -> Float {
    let v = -wo;
    let wh = (v + wi).normalize();
    let wh_dot_v = v.dot(wh);

    mfd.sample_normal_pdf(wh, v, Normal::Z) / (4.0 * wh_dot_v)
}