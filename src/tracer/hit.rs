use crate::{ Point, Float, Direction, Normal, efloat, Vec2, Vec3 };
use crate::tracer::{ material::Material, ray::Ray };

/// Stores information about a hit between a ray and an object
#[derive(Clone)]
pub struct Hit<'a> {
    /// The `t` value of ray at which the hit occurred
    pub t: Float,
    /// Material of the object which got hit
    pub material: &'a Material,
    /// 3D point where object was hit
    pub p: Point,
    /// Floating point error bounds of the impact point
    pub fp_error: Vec3,
    /// Normal of the surface used for shading calculations
    pub ns: Normal,
    /// Geometric normal of the surface
    pub ng: Normal,
    /// Texture coordinates in `\[0,1\]^2`
    pub uv: Vec2,
    /// Are we on the backface?
    pub backface: bool,
}

impl<'a> Hit<'a> {
    /// # Arguments
    ///
    /// * `t` - Value of ray at which hit occurred
    /// * `material` - Material of the object which got hit
    /// * `wo` - Direction towards to point of intersection
    /// * `xi` - Point in world space at which object got hit
    /// * `fp_error` - Floating point error bounds for `xi`
    /// * `ns` - Shading normal of the object at the point of impact
    /// * `ng` - Geometric normal of the object at the point of impact
    /// * `uv` - Texture coordinates in `\[0,1\]^2`
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        t: Float,
        material: &'a Material,
        wo: Direction,
        xi: Point,
        fp_error: Vec3,
        ns: Normal,
        ng: Normal,
        uv: Vec2,
    ) -> Option<Self> {
        let backface = wo.dot(ng) > 0.0;

        Some(Self {
            t,
            material,
            backface,
            p: xi,
            fp_error,
            ns,
            ng,
            uv: Self::wrap_uv(uv),
        })
    }

    #[inline(always)]
    pub fn wrap_uv(uv: Vec2) -> Vec2 {
        let uv = uv.fract();
        Vec2::new(
            if uv.x < 0.0 { uv.x + 1.0 } else { uv.x },
            if uv.y < 0.0 { uv.y + 1.0 } else { uv.y },
        )
    }

    pub fn from_t(t: Float) -> Option<Self> {
        Self::new(
            t,
            &Material::Blank,
            Direction::Z,
            Point::Z,
            Vec3::ZERO,
            Normal::Z,
            Normal::Z,
            Vec2::ZERO,
        )
    }

    /// Robustly offset ray origin using floating point error boundaries
    pub fn ray_origin(&self, outside: bool) -> Point {
        let ne = self.ng;
        let scaled_err = self.fp_error.dot(ne.abs());

        let offset = if outside {
            ne * scaled_err
        } else {
            -ne * scaled_err
        };

        let xi = self.p + offset;

        let move_double = |v: Float, n: Float| {
            if n > 0.0 {
                efloat::next_float(v)
            } else if n < 0.0 {
                efloat::previous_float(v)
            } else {
                v
            }
        };

        Point::new(
            move_double(xi.x, offset.x),
            move_double(xi.y, offset.y),
            move_double(xi.z, offset.z),
        )
    }

    /// Generates a ray at point of impact. Would be better to use accurate
    /// error bounds instead of `EPSILON`.
    pub fn generate_ray(&self, wi: Direction) -> Ray {
        let xi = self.ray_origin(wi.dot(self.ng) >= 0.0);

        Ray::new(
            xi,
            wi
        )
    }

    /// Did we hit a medium?
    #[inline]
    pub fn is_medium(&self) -> bool {
        matches!(self.material, Material::Volumetric(..))
    }
}
