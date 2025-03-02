use super::*;

/// Sphere specified by its radius and origin
pub struct Sphere {
    /// Radius of the sphere
    pub radius: Float,
    /// Material of the sphere
    material: Material,
}

impl Sphere {
    /// # Arguments
    /// * `origin` - Origin of the sphere
    /// * `radius` - Radius of the sphere
    /// * `material` - Material of the sphere
    pub fn new(radius: Float, material: Material) -> Box<Self> {
        assert!(radius != 0.0);

        Box::new(Self {
            radius,
            material,
        })
    }
}

impl Bounded for Sphere {
    fn bounding_box(&self) -> AaBoundingBox {
        let r_vec = Point::splat(self.radius);
        AaBoundingBox::new(- r_vec, r_vec)
    }
}

impl Object for Sphere {
    /// Solve the quadratic
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        let dx = EFloat::from(wi.x);
        let dy = EFloat::from(wi.y);
        let dz = EFloat::from(wi.z);

        let ox = EFloat::from(xo.x);
        let oy = EFloat::from(xo.y);
        let oz = EFloat::from(xo.z);

        let radius2 = EFloat::from(self.radius) * EFloat::from(self.radius);

        let a = dx * dx + dy * dy + dz * dz;
        let b = EFloat::from(2.0) * (dx * ox + dy * oy + dz * oz);
        let c = ox * ox + oy * oy + oz * oz - radius2;

        let (t0, t1) = EFloat::quadratic(a, b, c)?;

        // sphere too far or behind
        if t0.high >= t_max || t1.low <= t_min {
            return None;
        }

        let t = if t0.low > t_min {
            t0
        } else {
            if t1.high >= t_max {
                return None;
            }
            t1
        };

        let xi = r.at(t.value);
        // reproject to sphere to reduce floating point error
        let xi = xi * self.radius / xi.length();
        let err = efloat::gamma(5) * xi.abs();

        let ni = xi / self.radius;

        let u = ((-ni.z).atan2(ni.x) + crate::PI) / (2.0 * crate::PI);
        let v = (-ni.y).acos() / crate::PI;
        let uv = Vec2::new(u, v);

        Hit::new(t.value, &self.material, r.dir, xi, err, ni, ni, uv)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        let xo = r.origin;
        let wi = r.dir;

        let a = wi.dot(wi);
        let b = 2.0 * wi.dot(xo);
        let c = xo.dot(xo) - self.radius * self.radius;

        let Some((t0, t1)) = util::quadratic(a, b, c) else { return crate::INF; };

        if t0 >= t_max || t1 <= t_min { return crate::INF; }

        if t0 > t_min {
            t0
        } else if t1 >= t_max {
            crate::INF
        } else {
            t1
        }
    }
}

impl Sampleable for Sphere {
    fn area(&self) -> Float {
        4.0 * crate::PI * self.radius * self.radius
    }

    /// Sample on unit sphere and scale
    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let rand_sph = rng::maps::square_to_sphere(rand_sq);

        let xo = self.radius * rand_sph;
        // reproject to sphere to reduce floating point error
        let xo = xo * self.radius / xo.length();
        let err = xo.abs() * efloat::gamma(5);
        let ng = xo / self.radius;

        Hit::new(
            0.0,
            &self.material,
            -ng,
            xo,
            err,
            ng,
            ng,
            Vec2::ZERO,
        ).unwrap()
    }

    /// Visible area from `xo` forms a cone. Sample a random point on the
    /// spherical cap that the visible area forms. Return a ray with direction
    /// towards the sampled point.
    fn sample_towards(&self, xo: Point, rand_sq: Vec2) -> Direction {
        let dist_origin2 = xo.length_squared();
        let radius2 = self.radius * self.radius;

        let xi = if dist_origin2 < radius2 {
            // if inside sphere, just sample on the surface
            let xi = self.sample_on(rand_sq).p;
            xi
        } else {
            /* uvw-orthonormal basis,
             * where w is the direction from xo to origin of this sphere. */
            let uvw = Onb::new(xo);

            let dist_origin = dist_origin2.sqrt();

            // theta_max = maximum angle of the visible cone to sphere

            let sin2_theta_max = radius2 / dist_origin2;
            let cos_theta_max = (1.0 - sin2_theta_max).max(0.0).sqrt();

            let cos_theta = (1.0 - rand_sq.x) + rand_sq.x * cos_theta_max;
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let phi = 2.0 * crate::PI * rand_sq.y;

            // we have a point on the disk base of the cone.
            // consider disk origin to be at the sphere origin, say `xs`.
            // we compute normal at the point on the sphere where the direction
            // `xs - xo` from `xo` intersects the sphere. then add the normal
            // scaled to radius to the origin of the sphere to get the point
            // on the spherical cap.

            let dist_sampled = dist_origin * cos_theta
                - (radius2 - dist_origin2 * sin_theta * sin_theta).max(0.0).sqrt();

            // alpha = angle between `origin - xo` and normal at sampled point
            let cos_alpha = (dist_origin2 + radius2 - dist_sampled * dist_sampled)
                / (2.0 * dist_origin * self.radius);
            let sin_alpha = (1.0 - cos_alpha * cos_alpha).max(0.0).sqrt();

            let ng_local = Normal::new(
                phi.cos() * sin_alpha,
                phi.sin() * sin_alpha,
                cos_alpha,
            );

            let ng = uvw.to_world(ng_local);

            ng * self.radius
        };

        (xi - xo).normalize()
    }

    /// PDF (w.r.t SA) for sampling area of the sphere
    /// that is visible from `xo` (a spherical cap formed by a cone)
    fn sample_towards_pdf(&self, ri: &Ray, xi: Point, ng: Normal) -> Float {
        let xo = ri.origin;

        let radius2 = self.radius * self.radius;
        let dist_origin2 = xo.length_squared();

        let area = if dist_origin2 < radius2 {
            4.0 * crate::PI * radius2
        } else {
            /* this computes the area of the spherical cap of the visible
             * area. slightly faster way is to directly compute the
             * solid angle of the visible cone. the area is then with
             * respect to the disk at the base of the spherical cap though
             */

            let dist_origin = dist_origin2.sqrt();

            let sin2_theta_max = radius2 / dist_origin2;
            let cos_theta_max = (1.0 - sin2_theta_max).max(0.0).sqrt();

            let dist_tangent = cos_theta_max * dist_origin;

            let cos_alpha_max =
                (dist_origin2 + radius2 - dist_tangent * dist_tangent)
                / (2.0 * dist_origin * self.radius);

            2.0 * crate::PI * (1.0 - cos_alpha_max) * radius2
        };

        let p_area = 1.0 / area;

        let wi = ri.dir;

        p_area * xo.distance_squared(xi) / ng.dot(wi).abs()
    }
}

#[cfg(test)]
mod sphere_tests {
    use super::*;
    test_util::test_sampleable!(Sphere::new(1.0, Material::Blank));
}
