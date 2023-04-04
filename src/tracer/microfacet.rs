use glam::{DVec2, DVec3};
use std::f64::consts::PI;
use crate::tracer::onb::Onb;

/// Configurable parameters for a microsurface
#[derive(Copy, Clone)]
pub struct MicrofacetConfig {
    /// Roughness of the surface (α) [0,1]
    pub roughness: f64,
    /// Refraction index of the material >= 1.0
    pub refraction_idx: f64,
    /// Ratio of how metallic the material is [0,1]
    pub metallicity: f64,
    /// Transparency of the material
    pub transparent: bool,
}

impl MicrofacetConfig {
    pub fn new(roughness: f64, refraction_idx: f64, metallicity: f64, transparent: bool) -> Self {
        assert!((0.0..=1.0).contains(&roughness));
        assert!((0.0..=1.0).contains(&metallicity));
        assert!(refraction_idx >= 1.0);

        Self {
            roughness: roughness.max(1e-5),
            refraction_idx,
            metallicity,
            transparent,
        }
    }
}

/// Defines a distribution of normals for a microfacet. `f64` parameter is the
/// roughness (α) of the surface.
#[derive(Copy, Clone)]
pub enum MfDistribution {
    /// Walter et al. 2007
    Ggx(MicrofacetConfig),
    /// Beckmann et al. 1987
    Beckmann(MicrofacetConfig),
}

impl MfDistribution {
    /// Metallic material
    pub fn metallic(roughness: f64) -> Self {
        Self::Ggx(MicrofacetConfig::new(roughness, 1.5, 1.0, false))
    }

    /// Specular material
    pub fn specular(roughness: f64) -> Self {
        Self::Ggx(MicrofacetConfig::new(roughness, 1.5, 0.0, false))
    }

    /// Diffuse material
    pub fn diffuse() -> Self {
        Self::Ggx(MicrofacetConfig::new(1.0, 1.5, 0.0, false))
    }

    /// Transparent material f.ex. glass
    pub fn transparent(refraction_idx: f64, roughness: f64) -> Self {
        Self::Ggx(MicrofacetConfig::new(roughness, refraction_idx, 0.0, true))
    }

    /// might need tuning, send ratio that emittance is multiplied with?
    pub fn specularity(&self) -> f64 {
        if self.is_transparent() {
            1.0
        } else {
            1.0 - self.get_config().roughness.powi(2)
        }
    }

    /// Is the material transparent?
    pub fn is_transparent(&self) -> bool {
        self.get_config().transparent
    }

    /// Gets the refraction index
    pub fn get_rfrct_idx(&self) -> f64 {
        self.get_config().refraction_idx
    }

    pub fn get_roughness(&self) -> f64 {
        self.get_config().roughness
    }

    /// Getter, better way to do this?
    fn get_config(&self) -> &MicrofacetConfig {
        match self {
            Self::Ggx(cfg) | Self::Beckmann(cfg) => cfg,
        }
    }

    /// Disney diffuse (Burley 2012) with renormalization to conserve energy
    /// as done in Frostbite (Lagarde et al. 2014)
    pub fn disney_diffuse(&self, no_dot_v: f64, no_dot_wh: f64, no_dot_wi: f64) -> f64 {
        let roughness2 = self.get_config().roughness.powi(2);
        let energy_bias = 0.5 * roughness2;
        let fd90 = energy_bias + 2.0 * no_dot_wh.powi(2) * roughness2;

        let view_scatter = 1.0 + (fd90 - 1.0) * (1.0 - no_dot_v).powi(5);
        let light_scatter = 1.0 + (fd90 - 1.0) * (1.0 - no_dot_wi).powi(5);

        let energy_factor = 1.0 + roughness2 * (1.0 / 1.51 - 1.0);

        view_scatter * light_scatter * energy_factor
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
                let cos2_theta = wh.dot(no).powi(2);
                if cos2_theta == 0.0 {
                    0.0
                } else {
                    let roughness2 = cfg.roughness * cfg.roughness;

                    roughness2
                        / (PI * (cos2_theta * (roughness2 - 1.0) + 1.0).powi(2))
                }
            }
            Self::Beckmann(cfg) => {
                let roughness2 = cfg.roughness * cfg.roughness;
                let cos2_theta = wh.dot(no).powi(2);
                if cos2_theta == 0.0 {
                    0.0
                } else {
                    let tan2_theta = (1.0 - cos_theta2) / cos_theta2;

                    (-tan_theta2 / roughness2).exp()
                        / (PI * roughness2 * cos_theta2.powi(2))
                }
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

    pub fn g1(&self, v: DVec3, no: DVec3) -> f64 {
        1.0 / (1.0 + self.lambda(v, no))
    }

    /// Fresnel term with Schlick's approximation
    pub fn f(&self, wo: DVec3, wh: DVec3, color: DVec3) -> DVec3 {
        let eta = self.get_config().refraction_idx;
        let metallicity = self.get_config().metallicity;

        let f0 = (eta - 1.0) / (eta + 1.0);
        let f0 = DVec3::splat(f0 * f0).lerp(color, metallicity);

        let wo_dot_wh = wo.dot(wh).abs();
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
                let cos2_theta = w.dot(no).powi(2);
                if cos2_theta == 0.0 {
                    0.0
                } else {
                    let tan2_theta = (1.0 - cos2_theta) / cos2_theta;
                    let roughness2 = cfg.roughness * cfg.roughness;

                    ((1.0 + roughness2 * tan_w).sqrt() - 1.0) / 2.0
                }
            }
            Self::Beckmann(cfg) => {
                let cos2_theta = w.dot(no).powi(2);
                if cos2_theta == 0.0 {
                    0.0
                } else {
                    let tan2_theta = ((1.0 - w_dot_no2) / w_dot_no2).abs();
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
    }

    /// Probability to do importance sampling from NDF. Estimate based on
    /// the Fresnel term.
    pub fn probability_ndf_sample(&self, albedo: DVec3) -> f64 {
        let cfg = self.get_config();

        let f0 = (cfg.refraction_idx - 1.0) / (cfg.refraction_idx + 1.0);
        let f0 = f0 * f0;
        let albedo_mean = (albedo.x + albedo.y + albedo.z) / 3.0;

        (1.0 - cfg.metallicity) * f0 + cfg.metallicity * albedo_mean
    }

    /// Probability that `wh` got sampled
    pub fn sample_normal_pdf(&self, wh: DVec3, v: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Beckmann(..) => {
                let wh_dot_no = wh.dot(no);
                self.d(wh, no) * wh_dot_no
            }
            Self::Ggx(..) => {
                let wh_dot_v = wh.dot(v);
                let no_dot_v = no.dot(v);

                self.g1(v, no) * self.d(wh, no) * wh_dot_v / no_dot_v
            }
        }
    }

    /// Sampling microfacet normals per distribution for importance sampling.
    /// `v` in shading space.
    pub fn sample_normal(&self, v: DVec3, rand_sq: DVec2) -> DVec3 {
        match self {
            Self::Ggx(cfg) => {
                // Heitz 2018 or
                // https://schuttejoe.github.io/post/ggximportancesamplingpart2/

                let roughness = cfg.roughness;
                // Map the GGX ellipsoid to a hemisphere
                let v_stretch = DVec3::new(
                    v.x * roughness,
                    v.y * roughness,
                    v.z
                ).normalize();

                // ONB basis of the hemisphere configuration
                let hemi_basis = Onb::new(v_stretch);

                // compute a point on the disk
                let a = 1.0 / (1.0 + v_stretch.z);
                let r = rand_sq.x.sqrt();
                let phi = if rand_sq.y < a {
                    PI * rand_sq.y / a
                } else {
                    PI + PI * (rand_sq.y - a) / (1.0 - a)
                };

                let x = r * phi.cos();
                let y = if rand_sq.y < a {
                    r * phi.sin()
                } else {
                    r * phi.sin() * v_stretch.z
                };

                // compute normal in hemisphere configuration
                let wm = DVec3::new(
                    x,
                    y,
                    (1.0 - x*x - y*y).max(0.0).sqrt(),
                );
                let wm = hemi_basis.to_world(wm);

                // move back to ellipsoid
                DVec3::new(
                    roughness * wm.x,
                    roughness * wm.y,
                    wm.z.max(0.0)
                ).normalize()
            }
            Self::Beckmann(cfg) => {
                let roughness2 = cfg.roughness * cfg.roughness;
                let theta = (-roughness2 * (1.0 - rand_sq.y).ln()).sqrt().atan();
                let phi = 2.0 * PI * rand_sq.x;

                DVec3::new(
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                )
            }
        }
    }
}
