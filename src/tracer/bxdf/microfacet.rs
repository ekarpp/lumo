use super::*;

/* util functions */
mod util {
    use super::*;

    pub fn reflect(wo: Direction, no: Normal) -> Option<Direction> {
        let wi = 2.0 * wo.project_onto(no) - wo;
        if !spherical_utils::same_hemisphere(wi, wo) {
            // bad sample, do something else?
            None
        } else {
            Some( wi )
        }
    }

    pub fn refract(eta: Float, wo: Direction, no: Normal) -> Option<Direction> {
        /* Snell-Descartes law */
        // possibly flip the orientation if we are "inside"
        let (cos_to, eta_ratio, n) = if no.dot(wo) < 0.0 {
            (-no.dot(wo), 1.0 / eta, -no)
        } else {
            (no.dot(wo), eta, no)
        };
        let sin2_to = 1.0 - cos_to * cos_to;
        let sin2_ti = sin2_to / eta_ratio.powi(2);

        if sin2_ti >= 1.0 {
            /* total internal reflection */
            // handled in dielectric_sample
            unreachable!()
        } else {
            let cos_ti = (1.0 - sin2_ti).max(0.0).sqrt();
            let wi = -wo / eta_ratio + (cos_to / eta_ratio - cos_ti) * n;

            if spherical_utils::same_hemisphere(wi, wo) {
                None
            } else {
                Some( wi )
            }
        }
    }

    pub fn reflect_coeff(
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        mfd: &MfDistribution,
    ) -> Color {
        let cos_theta_wo = spherical_utils::cos_theta(wo);
        let cos_theta_wi = spherical_utils::cos_theta(wi);
        let wh = (wi + wo).normalize();

        let d = mfd.d(wh);
        let f = mfd.f(wo, wh, lambda);
        let g = mfd.g(wo, wi, wh);

        d * f * g / (4.0 * cos_theta_wo.abs() * cos_theta_wi.abs())
    }
}

/*
 * MICROFACET CONDUCTOR
 */
pub mod conductor {
    use super::*;

    pub fn f(
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        uv: Vec2,
        mfd: &MfDistribution,
    ) -> Color {
        let ks = mfd.ks(lambda, uv);
        if mfd.is_delta() {
            let f = mfd.f(wo, Normal::Z, lambda);
            ks * f / spherical_utils::cos_theta(wi).abs()
        } else {
            ks * util::reflect_coeff(wo, wi, lambda, mfd)
        }
    }

    pub fn sample(
        wo: Direction,
        mfd: &MfDistribution,
        rand_sq: Vec2
    ) -> Option<Direction> {
        if mfd.is_delta() {
            // 2.0 * wo.project(Z) - wo = 2.0 * (0, 0, wo.z) - v = (-wo.x, -wo.y, wo.z)
            Some( Direction::new(-wo.x, -wo.y, wo.z) )
        } else {
            let wh = mfd.sample_normal(wo, rand_sq);
            util::reflect(wo, wh)
        }
    }

    pub fn pdf(
        wo: Direction,
        wi: Direction,
        mfd: &MfDistribution,
    ) -> Float {
        // check if in same hemisphere or perpendicular to normal
        if !spherical_utils::same_hemisphere(wi, wo) {
            return 0.0;
        }

        let wh = (wo + wi).normalize();
        let wh = if spherical_utils::cos_theta(wh) < 0.0 { -wh } else { wh };

        if mfd.is_delta() {
            if 1.0 - spherical_utils::cos_theta(wh) < crate::EPSILON { 1.0 } else { 0.0 }
        } else {
            let wh_dot_wo = wo.dot(wh);
            mfd.sample_normal_pdf(wh, wo) / (4.0 * wh_dot_wo.abs())
        }
    }
}

/*
 * MICROFACET DIFFUSE
 * Disney diffuse (Burley 2012) with renormalization to conserve energy
 * as done in Frostbite (Lagarde et al. 2014)
 */
pub mod diffuse {
    use super::*;

    pub fn f(
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        uv: Vec2,
        mfd: &MfDistribution,
    ) -> Color {
        let wh = (wo + wi).normalize();

        let cos_wo = spherical_utils::cos_theta(wo);
        let cos_wi = spherical_utils::cos_theta(wi);
        let cos_wh = spherical_utils::cos_theta(wh);

        let d = mfd.d(wh);
        let f = mfd.f(wo, wh, lambda);
        let g = mfd.g(wo, wi, wh);

        let fr = d * f * g / (4.0 * cos_wo.abs() * cos_wi.abs());
        let fd = mfd.disney_diffuse(cos_wo, cos_wi, cos_wh);


        let ks = mfd.ks(lambda, uv);
        let kd = mfd.kd(lambda, uv);

        fr * ks + kd * (Color::WHITE - f) * fd / crate::PI
    }

    #[allow(dead_code)]
    pub fn sample(
        wo: Direction,
        mfd: &MfDistribution,
        rand_u: Float,
        rand_sq: Vec2,
    ) -> Option<Direction> {
        let pr = mfd.f_schlick(0.04, 1.0, spherical_utils::cos_theta(wo));
        let ps = 1.0 - pr;

        if rand_u < pr / (pr + ps) {
            let wh = if mfd.is_delta() {
                Normal::Z
            } else {
                mfd.sample_normal(wo, rand_sq)
            };
            util::reflect(wo, wh)
        } else {
            scatter::lambertian::sample(rand_sq)
        }
    }

    #[allow(dead_code)]
    pub fn pdf(
        wo: Direction,
        wi: Direction,
        mfd: &MfDistribution,
    ) -> Float {
        if !spherical_utils::same_hemisphere(wi, wo) {
            return 0.0;
        }

        let wh = (wo + wi).normalize();
        let pr = mfd.f_schlick(0.04, 1.0, spherical_utils::cos_theta(wo));
        let ps = 1.0 - pr;

        let wh_dot_wo = wo.dot(wh);
        let p_ref = if mfd.is_delta() {
            if 1.0 - spherical_utils::cos_theta(wh) < crate::EPSILON { 1.0 } else { 0.0 }
        } else {
            mfd.sample_normal_pdf(wh, wo) / (4.0 * wh_dot_wo.abs())
        };

        let p_sct = scatter::lambertian::pdf(wo, wi);

        pr * p_ref + ps * p_sct
    }
}

/*
 * MICROFACET DIELECTRIC
 */
pub mod dielectric {
    use super::*;

    pub fn f(
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        reflection: bool,
        uv: Vec2,
        mfd: &MfDistribution,
        mode: Transport,
    ) -> Color {
        let cos_theta_wo = spherical_utils::cos_theta(wo);
        let cos_theta_wi = spherical_utils::cos_theta(wi);
        let wo_inside = cos_theta_wo < 0.0;
        let wl = lambda.leading_sample();
        let eta = mfd.eta_at(wl);

        let eta_ratio = if reflection {
            1.0
        } else if wo_inside {
            1.0 / eta
        } else {
            eta
        };

        let wh = if eta == 1.0 || mfd.is_delta() {
            Normal::Z
        } else {
            (wi * eta_ratio + wo).normalize()
        };

        if reflection {
            let ks = mfd.ks(lambda, uv);
            if eta == 1.0 || mfd.is_delta() {
                let f = mfd.f(wo, wh, lambda);
                ks * f / cos_theta_wi.abs()
            } else {
                ks * util::reflect_coeff(wo, wi, lambda, mfd)
            }
        } else {
            let f = mfd.f(wo, wh, lambda);
            let wh = if spherical_utils::cos_theta(wh) < 0.0 { -wh } else { wh };

            // scale coefficient if transporting radiance
            let scale = match mode {
                Transport::Radiance => eta_ratio * eta_ratio,
                Transport::Importance => 1.0,
            };

            let tf = mfd.tf(lambda, uv);

            if eta == 1.0 || mfd.is_delta() {
                tf * (Color::WHITE - f) / (scale * cos_theta_wi.abs())
            } else {
                let d = mfd.d(wh);
                let g = mfd.g(wo, wi, wh);

                let wh_dot_wo = wh.dot(wo);
                let wh_dot_wi = wh.dot(wi);

                tf * d * (Color::WHITE - f) * g / scale
                    * (wh_dot_wi * wh_dot_wo / (cos_theta_wi * cos_theta_wo)).abs()
                    / (eta_ratio * wh_dot_wi + wh_dot_wo).powi(2)
            }
        }
    }

    pub fn sample(
        wo: Direction,
        mfd: &MfDistribution,
        lambda: &mut ColorWavelength,
        rand_u: Float,
        rand_sq: Vec2,
    ) -> Option<Direction> {
        let wl = if mfd.constant_eta() {
            lambda.leading_sample()
        } else {
            lambda.terminate()
        };
        let eta = mfd.eta_at(wl);
        let wh = if eta == 1.0 || mfd.is_delta() {
            Normal::Z
        } else {
            mfd.sample_normal(wo, rand_sq)
        };

        // importance sample reflection/transmission with fresnel
        let pr = mfd.f_at(wo, wh, wl);
        let pt = 1.0 - pr;

        if rand_u < pr / (pr + pt) {
            util::reflect(wo, wh)
        } else {
            util::refract(eta, wo, wh)
        }
    }

    pub fn pdf(
        wo: Direction,
        wi: Direction,
        reflection: bool,
        lambda: &ColorWavelength,
        mfd: &MfDistribution,
    ) -> Float {
        let cos_theta_wo = spherical_utils::cos_theta(wo);
        let cos_theta_wi = spherical_utils::cos_theta(wi);
        let wo_inside = cos_theta_wo < 0.0;
        let wl = lambda.leading_sample();
        let eta = mfd.eta_at(wl);

        let eta_ratio = if reflection {
            1.0
        } else if wo_inside {
            1.0 / eta
        } else {
            eta
        };

        let wh = if eta == 1.0 {
            // PBRT just returns 0.0 for delta and index matched dielectrics (in f too)
            Normal::Z
        } else {
            (wo + wi * eta_ratio).normalize()
        };
        // orient MS normal to same side as geometric normal
        let wh = if spherical_utils::cos_theta(wh) < 0.0 { -wh } else { wh };
        let wh_dot_wo = wo.dot(wh);
        let wh_dot_wi = wi.dot(wh);

        if wh_dot_wo == 0.0 || wh_dot_wi == 0.0 {
            return 0.0;
        }

        // discard backfacing wh
        if wh_dot_wo * cos_theta_wo < 0.0 || wh_dot_wi * cos_theta_wi < 0.0 {
            return 0.0;
        }

        let pr = mfd.f_at(wo, wh, wl);
        let pt = 1.0 - pr;

        if reflection && (eta == 1.0 || mfd.is_delta()) {
            // reflection with delta
            if 1.0 - spherical_utils::cos_theta(wh) < crate::EPSILON {
                pr / (pr + pt)
            } else {
                0.0
            }
        } else if reflection {
            // reflection with rough surface
            mfd.sample_normal_pdf(wh, wo) / (4.0 * wh_dot_wo.abs())
                * pr / (pr + pt)
        } else if eta == 1.0 || mfd.is_delta() {
            // transmission with delta
            if 1.0 - spherical_utils::cos_theta(wh) < crate::EPSILON {
                pt / (pr + pt)
            } else {
                0.0
            }
        } else {
            // transmission with rough surface
            mfd.sample_normal_pdf(wh, wo)
                * wh_dot_wi.abs() / (wh_dot_wi + wh_dot_wo / eta_ratio).powi(2)
                * pt / (pr + pt)
        }
    }
}
