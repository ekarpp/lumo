use crate::EPSILON;
use crate::tracer::material::Material;
use crate::tracer::ray::Ray;
use glam::{DVec2, DVec3};

/// Stores information about a hit between a ray and an object
pub struct Hit<'a> {
    /// The `t` value of ray at which the hit occurred
    pub t: f64,
    /// Material of the object which got hit
    pub material: &'a Material,
    /// 3D point where object was hit
    pub p: DVec3,
    /// Normal of the surface used for shading calculations
    pub ns: DVec3,
    /// Geometric normal of the surface used for scattering calculations
    pub ng: DVec3,
    /// Texture coordinates in `\[0,1\]^2`
    pub uv: DVec2,
}

impl<'a> Hit<'a> {
    /// # Arguments
    ///
    /// * `t` - Value of ray at which hit occurred
    /// * `material` - Material of the object which got hit
    /// * `xi` - Point in world space at which object got hit
    /// * `ns` - Shading normal of the object at the point of impact
    /// * `ng` - Geometric normal of the object at the point of impact
    /// * `uv` - Texture coordinates in `\[0,1\]^2`
    pub fn new(
        t: f64,
        material: &'a Material,
        xi: DVec3,
        ns: DVec3,
        ng: DVec3,
        uv: DVec2,
    ) -> Option<Self> {
        Some(Self {
            t,
            material,
            p: xi,
            ns,
            ng,
            uv,
        })
    }

    /// Generates a ray at point of impact. Would be better to use accurate
    /// error bounds instead of `EPSILON`.
    pub fn generate_ray(&self, wi: DVec3) -> Ray {
        let norm = if wi.dot(self.ng) >= 0.0 { self.ng } else { -self.ng };
        let xi = self.p + norm * EPSILON;

        Ray::new(
            xi,
            wi
        )
    }

    /// Did we hit a medium?
    pub fn is_medium(&self) -> bool {
        matches!(self.material, Material::Volumetric(..))
    }
}
