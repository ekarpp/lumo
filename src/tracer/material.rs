use crate::{Normal, Direction, Transport, Float, Vec3};
use crate::tracer::bxdfs;
use crate::tracer::color::Color;
use crate::tracer::hit::Hit;
use crate::tracer::microfacet::MfDistribution;
use crate::tracer::pdfs::{Pdf, CosPdf};
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;

/// Describes which material an object is made out of
pub enum Material {
    /// Glossy
    Microfacet(Texture, MfDistribution),
    /// Lambertian diffuse material. Mostly for debugging
    Lambertian(Texture),
    /// Emits light
    Light(Texture),
    /// Perfect reflection
    Mirror,
    /// Perfect refraction with refraction index as argument
    Glass(Float),
    /// Volumetric material for mediums. `scatter_param`, `sigma_t`, `sigma_s`
    Volumetric(Float, Vec3, Color),
    /// Not specified. Used with objects that are built on top of other objects.
    Blank,
}

impl Material {
    /// Helper function to create a microfacet material
    pub fn microfacet(
        texture: Texture,
        roughness: Float,
        refraction_idx: Float,
        metallicity: Float,
        transparent: bool
    ) -> Self {
        let mfd = MfDistribution::new(roughness, refraction_idx, metallicity, transparent);
        Self::Microfacet(texture, mfd)
    }

    /// Metallic microfacet material
    pub fn metallic(texture: Texture, roughness: Float) -> Self {
        Self::microfacet(texture, roughness, 1.5, 1.0, false)
    }

    /// Specular microfacet material
    pub fn specular(texture: Texture, roughness: Float) -> Self {
        Self::microfacet(texture, roughness, 1.5, 0.0, false)
    }

    /// Diffuse material
    pub fn diffuse(texture: Texture) -> Self {
        Self::microfacet(texture, 1.0, 1.5, 0.0, false)
    }

    /// Transparent material
    pub fn transparent(texture: Texture, roughness: Float, refraction_idx: Float) -> Self {
        Self::microfacet(texture, roughness, refraction_idx, 0.0, true)
    }

    /// Perfect reflection
    pub fn mirror() -> Self {
        Self::Mirror
    }

    /// Perfect refraction
    pub fn glass(refraction_index: Float) -> Self {
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
        match self {
            Self::Lambertian(_) => false,
            Self::Microfacet(_, mfd) => mfd.is_delta(),
            _ => true,
        }
    }


    /// How much light emitted at `h`?
    pub fn emit(&self, h: &Hit) -> Color {
        match self {
            Self::Light(t) => if h.backface {
                Color::BLACK
            } else {
                t.albedo_at(h)
            },
            _ => Color::BLACK
        }
    }

    /// What is the color at `h`?
    pub fn bsdf_f(
        &self,
        wo: Direction,
        wi: Direction,
        mode: Transport,
        h: &Hit
    ) -> Color {
        let ns = h.ns;
        let ng = h.ng;
        match self {
            Self::Mirror => Color::WHITE,
            Self::Glass(eta) => {
                match mode {
                    Transport::Importance => Color::WHITE,
                    Transport::Radiance => {
                        let inside = wi.dot(ng) > 0.0;
                        if inside {
                            Color::splat(1.0 / (eta * eta))
                        } else {
                            Color::splat(eta * eta)
                        }
                    }
                }
            }
            // volumetric BSDF handled in integrator to cancel out PDF
            Self::Volumetric(_, sigma_t, sigma_s) => {
                let transmittance = (-*sigma_t * h.t).exp();
                // cancel out the transmittance pdf taken from scene transmitance
                let pdf = (transmittance * *sigma_t).dot(Vec3::ONE)
                    / transmittance.dot(Vec3::ONE);

                if pdf == 0.0 { Color::WHITE } else { *sigma_s / pdf }
            }
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet(wo, wi, ng, ns, mode, t.albedo_at(h), mfd)
            }
            Self::Lambertian(t) => t.albedo_at(h) / crate::PI,
            _ => Color::BLACK,
        }
    }

    /// Computes the shading cosine coefficient per material
    pub fn shading_cosine(&self, wi: Direction, ns: Normal) -> Float {
        match self {
            Self::Microfacet(..) | Self::Lambertian(_) => ns.dot(wi).abs(),
            _ => 1.0
        }
    }

    /// How does `ro` get scattered at `ho`?
    pub fn bsdf_pdf(&self, ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
        match self {
            Self::Mirror => bxdfs::brdf_mirror_pdf(ho, ro),
            Self::Glass(ridx) => bxdfs::btdf_glass_pdf(ho, ro, *ridx),
            Self::Volumetric(g, ..) => bxdfs::brdf_volumetric_pdf(ro, *g),
            Self::Lambertian(_) => Some(Box::new(CosPdf::new(ho.ns))),
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet_pdf(ho, ro, t.albedo_at(ho), mfd)
            }
            Self::Light(_) | Self::Blank => None,
        }
    }
}
