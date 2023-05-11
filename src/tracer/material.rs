use crate::Transport;
use crate::tracer::bxdfs;
use crate::tracer::hit::Hit;
use crate::tracer::microfacet::MfDistribution;
use crate::tracer::pdfs::Pdf;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use glam::DVec3;

/// Describes which material an object is made out of
pub enum Material {
    /// Glossy
    Microfacet(Texture, MfDistribution),
    /// Emits light
    Light(Texture),
    /// Perfect reflection
    Mirror,
    /// Perfect refraction with refraction index as argument
    Glass(f64),
    /// Volumetric material for mediums. `scatter_param`, `sigma_t`, `sigma_s`
    Volumetric(f64, DVec3, DVec3),
    /// Not specified. Used with objects that are built on top of other objects.
    Blank,
}

impl Material {
    /// Helper function to create a microfacet material
    pub fn microfacet(
        texture: Texture,
        roughness: f64,
        refraction_idx: f64,
        metallicity: f64,
        transparent: bool
    ) -> Self {
        let mfd = MfDistribution::new(roughness, refraction_idx, metallicity, transparent);
        Self::Microfacet(texture, mfd)
    }

    /// Metallic microfacet material
    pub fn metallic(texture: Texture, roughness: f64) -> Self {
        Self::microfacet(texture, roughness, 1.5, 1.0, false)
    }

    /// Specular microfacet material
    pub fn specular(texture: Texture, roughness: f64) -> Self {
        Self::microfacet(texture, roughness, 1.5, 0.0, false)
    }

    /// Diffuse material
    pub fn diffuse(texture: Texture) -> Self {
        Self::microfacet(texture, 1.0, 1.5, 0.0, false)
    }

    /// Transparent material
    pub fn transparent(texture: Texture, roughness: f64, refraction_idx: f64) -> Self {
        Self::microfacet(texture, roughness, refraction_idx, 0.0, true)
    }

    /// Perfect reflection
    pub fn mirror() -> Self {
        Self::Mirror
    }

    /// Perfect refraction
    pub fn glass(refraction_index: f64) -> Self {
        assert!(refraction_index >= 1.0);
        Self::Glass(refraction_index)
    }

    /// Is the material specular? I.e. reflects light
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Mirror | Self::Glass(..) => true,
            Self::Microfacet(_, mfd) => mfd.is_specular(),
            _ => false,
        }
    }

    /// Does the material scattering follow delta distribution?
    /// Dumb hack to make delta things not have shadows in path trace.
    pub fn is_delta(&self) -> bool {
        matches!(self, Self::Mirror | Self::Glass(_))
    }


    /// How much light emitted at `h`?
    pub fn emit(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Light(t) => t.albedo_at(h),
            _ => DVec3::ZERO,
        }
    }

    /// What is the color at `h`?
    pub fn bsdf_f(&self, wo: DVec3, wi: DVec3, mode: Transport, h: &Hit) -> DVec3 {
        let ns = h.ns;
        let ng = h.ng;
        match self {
            Self::Mirror | Self::Glass(..) => DVec3::ONE,
            // volumetric BSDF handled in integrator to cancel out PDF
            Self::Volumetric(_, sigma_t, sigma_s) => {
                let transmittance = (-*sigma_t * h.t).exp();
                // cancel out the transmittance pdf taken from scene transmitance
                let pdf = (transmittance * *sigma_t).dot(DVec3::ONE)
                    / transmittance.dot(DVec3::ONE);

                if pdf == 0.0 { DVec3::ONE } else { *sigma_s / pdf }
            }
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet(wo, wi, ng, ns, mode, t.albedo_at(h), mfd)
            }
            _ => DVec3::ZERO,
        }
    }

    /// Computes the shading cosine coefficient per material
    pub fn shading_cosine(&self, wi: DVec3, ns: DVec3) -> f64 {
        match self {
            Self::Microfacet(..) => ns.dot(wi).abs(),
            _ => 1.0
        }
    }

    /// How does `ro` get scattered at `ho`?
    pub fn bsdf_pdf(&self, ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
        match self {
            Self::Mirror => bxdfs::brdf_mirror_pdf(ho, ro),
            Self::Glass(ridx) => bxdfs::btdf_glass_pdf(ho, ro, *ridx),
            Self::Volumetric(g, ..) => bxdfs::brdf_volumetric_pdf(ro, *g),
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet_pdf(ho, ro, t.albedo_at(ho), mfd)
            }
            Self::Light(_) | Self::Blank => None,
        }
    }
}
