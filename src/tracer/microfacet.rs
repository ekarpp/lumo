use crate::{ Normal, Direction, Float, Vec2 };
use crate::math::{ complex::Complex, spherical_utils };
use crate::tracer::{ Color, ColorWavelength, hit::Hit, Texture };

/// Configurable parameters for a microsurface
pub struct MicrofacetConfig {
    /// Roughness of the surface (α) [0,1]
    pub roughness: Vec2,
    /// Refraction index of the material >= 1.0
    pub eta: Float,
    /// Absoprtion coefficient
    pub k: Float,
    /// Diffuse reflectance
    pub kd: Texture,
    /// Specular reflectance
    pub ks: Texture,
    /// Transmission filter
    pub tf: Texture,
}

impl MicrofacetConfig {
    pub fn new(
        roughness: Float,
        eta: Float,
        k: Float,
        kd: Texture,
        ks: Texture,
        tf: Texture,
    ) -> Self {
        assert!((0.0..=1.0).contains(&roughness));
        assert!(eta > 0.0);
        assert!(k >= 0.0);

        Self {
            roughness: Vec2::splat(roughness.max(1e-5)),
            eta,
            k,
            kd, ks, tf,
        }
    }
}

/// Defines a distribution of normals for a microfacet. `Float` parameter is the
/// roughness (α) of the surface.
pub enum MfDistribution {
    /// Walter et al. 2007
    Ggx(MicrofacetConfig),
    /// Beckmann et al. 1987
    Beckmann(MicrofacetConfig),
}

impl MfDistribution {
    pub fn new(
        roughness: Float,
        eta: Float,
        k: Float,
        kd: Texture,
        ks: Texture,
        tf: Texture,
    ) -> Self {
        Self::Ggx(
            MicrofacetConfig::new(
                roughness,
                eta,
                k,
                kd, ks, tf,
            )
        )
    }

    /// might need tuning, send ratio that emittance is multiplied with?
    pub fn is_specular(&self) -> bool {
        let roughness = self.roughness();
        (roughness.x + roughness.y) / 2.0 < 0.01
    }

    /// Does the material have delta scattering distribution?
    pub fn is_delta(&self) -> bool {
        let roughness = self.roughness();
        (roughness.x + roughness.y) / 2.0 < 1e-3
    }

    /// Get refraction index from config
    pub fn eta(&self) -> Float {
        self.get_config().eta
    }

    /// Get absorption coefficient from config
    pub fn k(&self) -> Float {
        self.get_config().k
    }

    /// Get roughness from config
    pub fn roughness(&self) -> Vec2 {
        self.get_config().roughness
    }

    /// Get Kd value at `h`
    pub fn kd(&self, lambda: &ColorWavelength, h: &Hit) -> Color {
        self.get_config().kd.albedo_at(lambda, h)
    }

    /// Get Ks value at `h`
    pub fn ks(&self, lambda: &ColorWavelength, h: &Hit) -> Color {
        self.get_config().ks.albedo_at(lambda, h)
    }

    /// Get Tf value at `h`
    pub fn tf(&self, lambda: &ColorWavelength, h: &Hit) -> Color {
        self.get_config().tf.albedo_at(lambda, h)
    }

    /// Getter, better way to do this?
    fn get_config(&self) -> &MicrofacetConfig {
        match self {
            Self::Ggx(cfg) | Self::Beckmann(cfg) => cfg,
        }
    }

    /// Disney diffuse (Burley 2012) with renormalization to conserve energy
    /// as done in Frostbite (Lagarde et al. 2014)
    pub fn disney_diffuse(
        &self,
        cos_theta_wo: Float,
        cos_theta_wi: Float,
        cos_theta_wh: Float
    ) -> Float {
        let roughness2 = self.roughness().x.powi(2);
        let energy_bias = 0.5 * roughness2;
        let fd90 = energy_bias + 2.0 * cos_theta_wh.powi(2) * roughness2;

        let view_scatter = self.f_schlick(1.0, fd90, cos_theta_wo);
        let light_scatter = self.f_schlick(1.0, fd90, cos_theta_wi);

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
    /// * `wh` - Microsurface normal in shading space
    pub fn d(&self, wh: Normal) -> Float {
        match self {
            Self::Ggx(cfg) => {
                let tan2_theta = spherical_utils::tan2_theta(wh);

                if tan2_theta.is_infinite() {
                    0.0
                } else {
                    let cos4_theta = spherical_utils::cos2_theta(wh).powi(2);
                    if cos4_theta < crate::EPSILON.powi(2) {
                        return 0.0;
                    }
                    let cos_phi = spherical_utils::cos_phi(wh);
                    let sin_phi = spherical_utils::sin_phi(wh);

                    let alpha2 = cfg.roughness.x * cfg.roughness.y;
                    let e = tan2_theta * (
                        (cos_phi / cfg.roughness.x).powi(2)
                            + (sin_phi / cfg.roughness.y).powi(2)
                    );

                    1.0 / (crate::PI * alpha2 * cos4_theta * (1.0 + e).powi(2))

                }
            }
            Self::Beckmann(cfg) => {
                let tan2_theta = spherical_utils::tan2_theta(wh);

                if tan2_theta.is_infinite() {
                    0.0
                } else {
                    let roughness2 = cfg.roughness.x * cfg.roughness.x;
                    let cos4_theta = spherical_utils::cos2_theta(wh).powi(2);

                    (-tan2_theta / roughness2).exp()
                        / (crate::PI * roughness2 * cos4_theta)
                }
            }
        }
    }

    /// Schlicks approximation for Fresnel term
    pub fn f_schlick(&self, f0: Float, f90: Float, cos_theta: Float) -> Float {
        f0 + (f90 - f0) * (1.0 - cos_theta).powi(5)
    }

    /// Fresnel term with the full equations
    /// # Arguments
    /// * `wo`      - Direction to viewer in shading space
    /// * `wh`     - Microsurface normal in shading space
    pub fn f(&self, wo: Direction, wh: Normal) -> Float {
        if self.k() == 0.0 {
            self.fr_real(wo, wh)
        } else {
            self.fr_complex(wo, wh)
        }
    }

    fn fr_complex(&self, wo: Direction, wh: Normal) -> Float {
        // this is a complex number: n + ik
        let eta = Complex::new(self.eta(), self.k());
        let cos_o = wo.dot(wh).clamp(0.0, 1.0);
        let sin2_o = 1.0 - cos_o * cos_o;

        let sin2_i: Complex = sin2_o / (eta * eta);
        let cos_i: Complex = (1.0 - sin2_i).sqrt();

        let r_par: Complex = (eta * cos_o - cos_i) / (eta * cos_o + cos_i);
        let r_per: Complex = (cos_o - eta * cos_i) / (cos_o + eta * cos_i);

        (r_par.norm_sqr() + r_per.norm_sqr()) / 2.0
    }

    fn fr_real(&self, wo: Direction, wh: Normal) -> Float {
        let cos_o = wo.dot(wh);
        let inside = cos_o < 0.0;
        let eta = if inside { 1.0 / self.eta() } else { self.eta() };

        let cos_o = cos_o.abs();
        let sin2_o = 1.0 - cos_o * cos_o;
        let sin2_i = sin2_o / (eta * eta);

        // total internal reflection
        if sin2_i >= 1.0 {
            return 1.0;
        }

        let cos_i = (1.0 - sin2_i).max(0.0).sqrt();

        let r_par = (eta * cos_o - cos_i) / (eta * cos_o + cos_i);
        let r_per = (cos_o - eta * cos_i) / (cos_o + eta * cos_i);

        (r_par * r_par + r_per * r_per) / 2.0
    }

    fn chi_pass(&self, wo: Direction, wh: Normal) -> bool {
        // signum to fix refraction
        let cos_theta_wh = spherical_utils::cos_theta(wh);
        let cos_theta_wo = spherical_utils::cos_theta(wo);
        let chi = cos_theta_wh.signum() * wo.dot(wh) * cos_theta_wo;
        chi > crate::EPSILON
    }

    /// Shadow-masking term. Used to make sure that only microfacets that are
    /// visible from `v` direction are considered. Uses the method described
    /// in Chapter 8.4.3 of PBR due to Heitz et al. 2013.
    ///
    /// # Arguments
    /// * `v`  - View direction in shading space
    /// * `wi` - Direction of ray away from the point of impact in shading space
    /// * `wh` - Microsurface normal in shading space
    pub fn g(&self, wo: Direction, wi: Direction, wh: Normal) -> Float {
        if !self.chi_pass(wo, wh) {
            0.0
        } else {
            1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
        }
    }

    pub fn g1(&self, wo: Direction, wh: Normal) -> Float {
        if !self.chi_pass(wo, wh) {
            0.0
        } else {
            1.0 / (1.0 + self.lambda(wo))
        }
    }

    /// Lambda function used in the definition of the shadow-masking term.
    /// Beckmann with polynomial approximation and GGX exactly. PBR Chapter 8.4.3
    ///
    /// # Arguments
    /// * `w` - Direction to consider in shading space
    fn lambda(&self, w: Direction) -> Float {
        match self {
            Self::Ggx(cfg) => {
                let tan2_theta = spherical_utils::tan2_theta(w);

                if tan2_theta.is_infinite() {
                    0.0
                } else {
                    let cos_phi = spherical_utils::cos_phi(w);
                    let sin_phi = spherical_utils::sin_phi(w);

                    let alpha2 = (cfg.roughness.x * cos_phi).powi(2)
                        + (cfg.roughness.y * sin_phi).powi(2);

                    ((1.0 + alpha2 * tan2_theta).max(0.0).sqrt() - 1.0) / 2.0
                }
            }
            Self::Beckmann(cfg) => {
                let tan2_theta = spherical_utils::tan2_theta(w);
                if tan2_theta.is_infinite() {
                    0.0
                } else {
                    let a = 1.0 / (cfg.roughness.x * tan2_theta);

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

    /// Probability that `wh` got sampled. `wh` and `wo` in shading space.
    pub fn sample_normal_pdf(
        &self,
        wh: Normal,
        wo: Direction,
    ) -> Float {
        let pdf = match self {
            Self::Beckmann(..) => {
                let cos_theta_wh = spherical_utils::cos_theta(wh);
                self.d(wh) * cos_theta_wh
            }
            Self::Ggx(..) => {
                let wh_dot_wo = wh.dot(wo);
                let cos_theta_wo = spherical_utils::cos_theta(wo);

                self.g1(wo, wh) * self.d(wh) * wh_dot_wo.abs() / cos_theta_wo.abs()
            }
        };

        pdf.max(0.0)
    }

    /// Sampling microfacet normals per distribution for importance sampling.
    /// `wo` in shading space.
    pub fn sample_normal(&self, wo: Direction, rand_sq: Vec2) -> Normal {
        match self {
            Self::Ggx(cfg) => {
                // Heitz 2018 or
                // https://schuttejoe.github.io/post/ggximportancesamplingpart2/

                let roughness = cfg.roughness;
                // Map the GGX ellipsoid to a hemisphere
                let wo_stretch = Normal::new(
                    wo.x * roughness.x,
                    wo.y * roughness.y,
                    wo.z
                ).normalize();
                // orient the sampled normal to same hemisphere as geometric normal
                let wo_stretch = if wo_stretch.z < 0.0 { -wo_stretch } else { wo_stretch };

                // ONB basis of the hemisphere configuration
                // don't use Onb class, as it has too strict requirements for orthonormality
                // first vector should be perpendicular to Z
                let u = if 1.0 - wo_stretch.z < crate::EPSILON {
                    Normal::X
                } else {
                    wo_stretch.cross(Normal::Z).normalize()
                };
                let v = u.cross(wo_stretch);

                // first a point on the unit disk
                let r = rand_sq.x.sqrt();
                let theta = 2.0 * crate::PI * rand_sq.y;
                let x = r * theta.cos();
                // then map it to the projection disk
                let h = (1.0 - x * x).max(0.0).sqrt();
                let lerp = (1.0 + wo_stretch.z) / 2.0;
                let y = (1.0 - lerp) * h + lerp * r * theta.sin();

                // compute normal in hemisphere configuration
                let wm = Normal::new(
                    x,
                    y,
                    (1.0 - x*x - y*y).max(0.0).sqrt(),
                );

                // move back to ellipsoid
                let wm = wm.x * u + wm.y * v + wm.z * wo_stretch;
                Normal::new(
                    roughness.x * wm.x,
                    roughness.y * wm.y,
                    wm.z.max(crate::EPSILON)
                ).normalize()
            }
            Self::Beckmann(cfg) => {
                let roughness2 = cfg.roughness.x * cfg.roughness.x;
                let theta = (-roughness2 * (1.0 - rand_sq.y).ln()).sqrt().atan();
                let phi = 2.0 * crate::PI * rand_sq.x;

                // already normalized?
                Normal::new(
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ).normalize()
            }
        }
    }
}
