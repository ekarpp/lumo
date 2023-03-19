use crate::DVec3;
use std::f64::consts::PI;

/// Configurable parameters for a microsurface
#[derive(Copy, Clone)]
pub struct MicrofacetConfig {
    /// Roughness of the surface (α) [0,1]
    pub roughness: f64,
    /// Refraction index of the material >= 1.0
    pub refraction_idx: f64,
    /// Ratio of how metallic the material is [0,1]
    pub metallicity: f64,
}

impl MicrofacetConfig {
    pub fn new(roughness: f64, refraction_idx: f64, metallicity: f64) -> Self {
        assert!(roughness <= 1.0 && roughness >= 0.0);
        assert!(refraction_idx >= 1.0);
        assert!(metallicity <= 1.0 && metallicity >= 0.0);
        Self {
            roughness,
            refraction_idx,
            metallicity,
        }
    }
}

/// Defines a distribution of normals for a microfacet. `f64` parameter is the
/// roughness (α) of the surface.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum MfDistribution {
    /// Walter et al. 2007
    Ggx(MicrofacetConfig),
    /// Beckmann et al. 1987
    Beckmann(MicrofacetConfig),
}

impl MfDistribution {
    /// Metallic material
    pub fn metallic(roughness: f64) -> Self {
        Self::Ggx(MicrofacetConfig::new(roughness, 1.5, 1.0))
    }

    /// Specular material
    pub fn specular(roughness: f64) -> Self {
        Self::Ggx(MicrofacetConfig::new(roughness, 1.5, 0.0))
    }

    /// might need tuning, send ratio that emittance is multiplied with?
    pub fn is_specular(&self) -> bool {
        self.get_config().roughness < 1.0
    }

    /// Getter, better way to do this?
    fn get_config(&self) -> &MicrofacetConfig {
        match self {
            Self::Ggx(cfg) | Self::Beckmann(cfg) => &cfg,
        }
    }

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
            Self::Ggx(cfg) => {
                let cos_theta2 = wh.dot(no).powi(2);
                let roughness2 = cfg.roughness * cfg.roughness;

                roughness2
                    / (PI * (cos_theta2 * (roughness2 - 1.0) + 1.0).powi(2))
            }
            Self::Beckmann(cfg) => {
                let roughness2 = cfg.roughness * cfg.roughness;
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
    pub fn f(&self, wo: DVec3, wh: DVec3, color: DVec3) -> DVec3 {
        let eta = self.get_config().refraction_idx;
        let metallicity = self.get_config().metallicity;

        let f0 = (eta - 1.0) / (eta + 1.0);
        let f0 = DVec3::splat(f0 * f0).lerp(color, metallicity);

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
            Self::Ggx(cfg) => {
                let w_dot_no2 = w.dot(no).powi(2);
                let tan_w = (1.0 - w_dot_no2) / w_dot_no2;
                let roughness2 = cfg.roughness * cfg.roughness;

                ((1.0 + roughness2 * tan_w).sqrt() - 1.0) / 2.0
            }
            Self::Beckmann(cfg) => {
                let w_dot_no2 = w.dot(no).powi(2);
                let tan_w = ((1.0 - w_dot_no2) / w_dot_no2).abs();
                let a = 1.0 / (cfg.roughness * tan_w);

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
            Self::Ggx(cfg) => {
                (cfg.roughness * (rand_f / (1.0 - rand_f)).sqrt()).atan()
            }
            Self::Beckmann(cfg) => {
                let roughness2 = cfg.roughness * cfg.roughness;
                (-roughness2 * (1.0 - rand_f).ln()).sqrt().atan()
            }
        }
    }
}
