use crate::DVec3;
use std::f64::consts::PI;

/// Defines a distribution of normals for a microfacet. `f64` parameter is the
/// roughness (α) of the surface.
#[derive(Copy, Clone)]
pub enum MfDistribution {
    /// Walter et al. 2007
    Ggx(f64),
    /// Beckmann et al. 1987
    Beckmann(f64),
}

impl MfDistribution {
    /// The microfacet distribution function.
    ///
    /// # Distributions
    /// * Beckmann - exp(-tan^2(θ) / α^2) / (π * α^2 * cos^4(θ))
    /// * GGX - α^2 / (π * (cos^4(θ) * (α^2 - 1.0) + 1.0)^2)
    ///
    /// # Arguments
    /// * `wh` - The half vector of `wo` and `wi`
    /// * `no` - Surface normal at the point of impact
    pub fn d(&self, wh: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                let cos_theta2 = wh.dot(no).powi(2);
                let roughness2 = roughness * roughness;

                roughness2
                    / (PI * (cos_theta2 * (roughness2 - 1.0) + 1.0).powi(2))
            }
            Self::Beckmann(roughness) => {
                let roughness2 = roughness * roughness;
                let cos_theta2 = wh.dot(no).powi(2);
                let tan_theta2 = (1.0 - cos_theta2) / cos_theta2;

                (-tan_theta2 / roughness2).exp()
                    / (PI * roughness2 * cos_theta2.powi(2))
            }
        }
    }

    /// Shadow-masking term. Used to make sure that only microfacets that are
    /// visible from `wo` direction are considered. Uses the method described
    /// in Chapter 8.4.3 of PBR due to Heitz et al. 2013.
    ///
    /// # Arguments
    /// * `wo` - Direction of ray towards the point of impact
    /// * `wi` - Direction of ray away from the point of impact
    /// * `no` - Surface normal at the point of impact
    pub fn g(&self, wo: DVec3, wi: DVec3, no: DVec3) -> f64 {
        1.0 / (1.0 + self.lambda(wo, no) + self.lambda(wi, no))
    }

    /// Fresnel term with Schlick's approximation
    pub fn f(&self, wo: DVec3, wh: DVec3, color: DVec3, metallic: f64, eta: f64)
             -> DVec3 {
        let f0 = (eta - 1.0) / (eta + 1.0);
        let f0 = DVec3::splat(f0 * f0).lerp(color, metallic);

        let wo_dot_wh = wo.dot(wh);
        f0 + (DVec3::ONE - f0) * (1.0 - wo_dot_wh).powi(5)
    }

    /// Lambda function used in the definition of the shadow-masking term.
    /// Beckmann with polynomial approximation and GGX exactly. PBR Chapter 8.4.3
    ///
    /// # Arguments
    /// * `w` - Direction to consider
    /// * `no` - Macrosurface normal
    fn lambda(&self, w: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                let w_dot_no2 = w.dot(no).powi(2);
                let tan_w = (1.0 - w_dot_no2) / w_dot_no2;
                let roughness2 = roughness * roughness;

                ((1.0 + roughness2 * tan_w).sqrt() - 1.0) / 2.0
            }
            Self::Beckmann(roughness) => {
                let w_dot_no2 = w.dot(no).powi(2);
                let tan_w = ((1.0 - w_dot_no2) / w_dot_no2).abs();
                let a = 1.0 / (roughness * tan_w);

                if a >= 1.6 {
                    0.0
                } else {
                    (1.0 - 1.259 * a + 0.396 * a * a)
                        / (3.535 * a + 2.181 * a * a)
                }
            }
        }
    }

    /// Sampling thetas per distribution for importance sampling.
    pub fn sample_theta(&self, rand_f: f64) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                (roughness * (rand_f / (1.0 - rand_f)).sqrt()).atan()
            }
            Self::Beckmann(roughness) => {
                let roughness2 = roughness * roughness;
                (-roughness2 * (1.0 - rand_f).ln()).sqrt().atan()
            }
        }
    }
}
