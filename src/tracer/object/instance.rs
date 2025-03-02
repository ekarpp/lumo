use super::*;

/// Instance of an object i.e. an object to which affine transformations have
/// been applied
pub struct Instance<T> {
    /// Object to be instanced
    object: T,
    /// Transformation from local to world
    transform: Transform,
    /// Transformation from world to local
    inv_transform: Transform,
    /// Transformation for normals from local to world.
    /// Transpose of `inv_transform` without translation.
    normal_transform: Mat3,
}

impl<T> Instance<T> {
    /// Constructs an instance of `object` that is transformed with
    /// `transform`.
    pub fn new(object: T, transform: Transform) -> Box<Self> {
        let inv_transform = transform.inverse();
        let normal_transform = inv_transform.matrix3.transpose();

        Box::new(Self {
            object,
            transform,
            inv_transform,
            normal_transform,
        })
    }

    fn propagate_fp_err(&self, h: &mut Hit) {
        let e3 = h.fp_error.abs();
        let p3 = h.p.abs();
        let m3 = self.transform.matrix3.abs();
        let t3 = self.transform.translation.abs();

        h.fp_error = if e3.x == 0.0 && e3.y == 0.0 && e3.z == 0.0 {
            efloat::gamma(3) * (t3 + m3.mul_vec3(p3))
        } else {
            efloat::gamma(3) * (t3 + m3.mul_vec3(p3))
                + (efloat::gamma(3) + 1.0) * m3.mul_vec3(e3)
        };
    }
}

impl<T: Bounded> Instance<T> {
    /// Translate `self`, such that bounding box center is at the origin
    pub fn to_origin(self) -> Box<Self> {
        let AaBoundingBox { ax_min, ax_max } = self.bounding_box();
        let ax_mid = -(ax_min + ax_max) / 2.0;

        self.translate(ax_mid.x, ax_mid.y, ax_mid.z)
    }

    /// Set `x` of bounding boxes `ax_min` to `x1`
    pub fn set_x(self, x1: Float) -> Box<Self> {
        let x_min = self.bounding_box().ax_min.x;
        self.translate(x1 - x_min, 0.0, 0.0)
    }

    /// Set `y` of bounding boxes `ax_min` to `y1`
    pub fn set_y(self, y1: Float) -> Box<Self> {
        let y_min = self.bounding_box().ax_min.y;
        self.translate(0.0, y1 - y_min, 0.0)
    }

    /// Set `z` of bounding boxes `ax_min` to `z1`
    pub fn set_z(self, z1: Float) -> Box<Self> {
        let z_min = self.bounding_box().ax_min.z;
        self.translate(0.0, 0.0, z1 - z_min)
    }
}

impl<T: Bounded> Bounded for Instance<T> {
    fn bounding_box(&self) -> AaBoundingBox {
        /* Graphics Gems I, TRANSFORMING AXIS-ALIGNED BOUNDING BOXES */
        let mut ax_min = Point::ZERO;
        let mut ax_max = Point::ZERO;
        let aabb = self.object.bounding_box();

        let a0 = self.transform.matrix3.row(0) * aabb.min(Axis::X);
        let b0 = self.transform.matrix3.row(0) * aabb.max(Axis::X);
        let a0b0 = a0.min(b0);
        ax_min.x += a0b0.x + a0b0.y + a0b0.z;
        let a0b0 = a0.max(b0);
        ax_max.x += a0b0.x + a0b0.y + a0b0.z;

        let a1 = self.transform.matrix3.row(1) * aabb.min(Axis::Y);
        let b1 = self.transform.matrix3.row(1) * aabb.max(Axis::Y);
        let a1b1 = a1.min(b1);
        ax_min.y += a1b1.x + a1b1.y + a1b1.z;
        let a1b1 = a1.max(b1);
        ax_max.y += a1b1.x + a1b1.y + a1b1.z;

        let a2 = self.transform.matrix3.row(2) * aabb.min(Axis::Z);
        let b2 = self.transform.matrix3.row(2) * aabb.max(Axis::Z);
        let a2b2 = a2.min(b2);
        ax_min.z += a2b2.x + a2b2.y + a2b2.z;
        let a2b2 = a2.max(b2);
        ax_max.z += a2b2.x + a2b2.y + a2b2.z;

        // translate
        ax_min.x += self.transform.translation.x;
        ax_min.y += self.transform.translation.y;
        ax_min.z += self.transform.translation.z;

        ax_max.x += self.transform.translation.x;
        ax_max.y += self.transform.translation.y;
        ax_max.z += self.transform.translation.z;

        AaBoundingBox::new(ax_min, ax_max)
    }
}

impl<T: Object> Object for Instance<T> {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        // inner object is in world coordinates. hence apply inverse
        // transformation to ray instead of transformation to object.
        let ray_local = r.transform(self.inv_transform);

        self.object.hit(&ray_local, t_min, t_max)
            .map(|mut h| {
                h.ns = (self.normal_transform * h.ns).normalize();
                h.ng = (self.normal_transform * h.ng).normalize();

                self.propagate_fp_err(&mut h);

                h.p =  self.transform.transform_point3(h.p);
                h
            })
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        let r_local = r.transform(self.inv_transform);
        self.object.hit_t(&r_local, t_min, t_max)
    }
}

impl<T: Sampleable> Sampleable for Instance<T> {
    fn area(&self) -> Float {
        let scale = Vec3::new(
            self.transform.matrix3.x_axis.length(),
            self.transform.matrix3.y_axis.length(),
            self.transform.matrix3.z_axis.length(),
        );

        // only allow uniform scale
        if (scale.x - scale.y).abs() + (scale.y - scale.z).abs() > crate::EPSILON {
            unimplemented!();
        }

        scale.x * scale.y * self.object.area()
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let mut ho = self.object.sample_on(rand_sq);

        ho.ng = self.normal_transform * ho.ng;
        ho.ns = self.normal_transform * ho.ns;
        ho.p = self.transform.transform_point3(ho.p);
        self.propagate_fp_err(&mut ho);

        ho
    }

    fn sample_towards(&self, xo: Point, rand_sq: Vec2) -> Direction {
        let xo_local = self.inv_transform.transform_point3(xo);
        let dir_local = self.object.sample_towards(xo_local, rand_sq);

        self.transform.transform_vector3(dir_local).normalize()
    }

    fn sample_towards_pdf(&self, ri: &Ray, xi: Point, ng: Normal) -> Float {
        let normal_transform_inverse = self.normal_transform.inverse().transpose();
        let ng_local = (normal_transform_inverse * ng).normalize();
        let xi_local = self.inv_transform.transform_point3(xi);
        let ri_local = ri.transform(self.inv_transform);
        let xo_local = ri_local.origin;
        let pdf_local = self.object.sample_towards_pdf(&ri_local, xi_local, ng_local);

        // imagine a unit cube at the sampled point with the surface normal
        // at that point as one of the cube edges. apply the linear
        // transformation to the cube and we get a parallellepiped.
        // the base of the parallellepiped gives us the area scale at the
        // point of impact. think this is not exact with ansiotropic
        // scaling of solids.
        let height = ng.dot(self.transform.matrix3 * ng_local).abs();
        let volume = self.transform.matrix3.determinant().abs();
        let jacobian = volume / height;

        let wi_local = ri_local.dir;
        let wi = ri.dir;
        let xo = ri.origin;
        // jacoabian takes care of area, just undo and redo SA conversion
        let sa_conv = xo.distance_squared(xi) * wi_local.dot(ng_local).abs()
            / (xo_local.distance_squared(xi_local) * wi.dot(ng).abs());

        // p_y(y) = p_y(T(x)) = p_x(x) / |J_T(x)|
        pdf_local * sa_conv / jacobian
    }
}

/// Object that can be instanced
pub trait Instanceable<T> {
    /// Translate object by `xyz`
    fn translate(self, x: Float, y: Float, z: Float) -> Box<Instance<T>>;

    /// Apply scale `xyz`
    fn scale(self, x: Float, y: Float, z: Float) -> Box<Instance<T>>;

    /// Apply uniform scale `s`
    fn scale_uniform(self, s: Float) -> Box<Instance<T>>;

    /// Rotate around x-axis by `r` radians
    fn rotate_x(self, r: Float) -> Box<Instance<T>>;

    /// Rotate around y-axis by `r` radians
    fn rotate_y(self, r: Float) -> Box<Instance<T>>;

    /// Rotate around z-axis by `r` radians
    fn rotate_z(self, r: Float) -> Box<Instance<T>>;

    /// Rotate around `axis` by `r` radians
    fn rotate_axis(self, axis: Direction, r: Float) -> Box<Instance<T>>;
}

/// To make applying transformations to objects easy
impl<T: Object> Instanceable<T> for T {
    fn translate(self, x: Float, y: Float, z: Float) -> Box<Instance<T>> {
        let t = Vec3::new(x, y, z);
        Instance::new(self, Transform::from_translation(t))
    }

    fn scale(self, x: Float, y: Float, z: Float) -> Box<Instance<T>> {
        assert!(x * y * z != 0.0);
        let s = Vec3::new(x, y, z);
        Instance::new(self, Transform::from_scale(s))
    }

    fn scale_uniform(self, s: Float) -> Box<Instance<T>> {
        self.scale(s, s, s)
    }

    fn rotate_x(self, r: Float) -> Box<Instance<T>> {
        Instance::new(self, Transform::from_rotation_x(r))
    }

    fn rotate_y(self, r: Float) -> Box<Instance<T>> {
        Instance::new(self, Transform::from_rotation_y(r))
    }

    fn rotate_z(self, r: Float) -> Box<Instance<T>> {
        Instance::new(self, Transform::from_rotation_z(r))
    }

    fn rotate_axis(self, axis: Direction, r: Float) -> Box<Instance<T>> {
        Instance::new(self, Transform::from_axis_angle(axis, r))
    }
}

/// Prevent nested Instance structs
impl<T: Object> Instance<T> {
    /// Apply translation AFTER curret transformations
    pub fn translate(self, x: Float, y: Float, z: Float) -> Box<Self> {
        let t = Vec3::new(x, y, z);
        Self::new(self.object, Transform::from_translation(t) * self.transform)
    }

    /// Apply scale AFTER current transformations
    pub fn scale(self, x: Float, y: Float, z: Float) -> Box<Self> {
        assert!(x * y * z != 0.0);
        let s = Vec3::new(x, y, z);
        Self::new(self.object, Transform::from_scale(s) * self.transform)
    }

    /// Apply uniform scale AFTER current transformations
    pub fn scale_uniform(self, s: Float) -> Box<Self> {
        self.scale(s, s, s)
    }

    /// Apply x-rotation AFTER current transformations.
    /// Looking at positive x, rotation in clockwise direction.
    pub fn rotate_x(self, r: Float) -> Box<Self> {
        Self::new(self.object, Transform::from_rotation_x(r) * self.transform)
    }

    /// Apply y-rotation AFTER current transformations
    pub fn rotate_y(self, r: Float) -> Box<Self> {
        Self::new(self.object, Transform::from_rotation_y(r) * self.transform)
    }

    /// Apply z-rotation AFTER current transformations
    pub fn rotate_z(self, r: Float) -> Box<Self> {
        Self::new(self.object, Transform::from_rotation_z(r) * self.transform)
    }

    /// Apply axis rotation AFTER current transformations
    pub fn rotate_axis(self, axis: Direction, r: Float) -> Box<Instance<T>> {
        Self::new(self.object, Transform::from_axis_angle(axis, r) * self.transform)
    }
}

#[cfg(test)]
mod instance_tests {
    use super::*;
    fn spheres() -> (Box<Instance<Sphere>>, Box<Sphere>) {
        let s_ref = Sphere::new(1.0, Material::Blank);
        let s = Sphere::new(0.1, Material::Blank)
            .rotate_x(crate::PI)
            .scale_uniform(10.0)
            .rotate_y(crate::PI)
            .rotate_z(crate::PI)
            .translate(-1.23, 4.56, -7.89)
            .set_y(-1.0)
            .set_x(-1.0)
            .set_z(-1.0);

        (s, s_ref)
    }

    test_util::test_sampleable!(spheres().0);

    #[test]
    fn sampling_equals_plain_object() {
        let (s, s_ref) = spheres();
        let mut rng = Xorshift::default();

        let xo = 5.0 * rng::maps::square_to_sphere(rng.gen_vec2());
        for _ in 0..NUM_SAMPLES {
            let wi = s.sample_towards(xo, rng.gen_vec2());
            let r = Ray::new(xo, wi);
            let Some(h) = s.hit(&r, 0.0, crate::INF) else { panic!() };
            let Some(h_ref) = s_ref.hit(&r, 0.0, crate::INF) else { panic!() };
            let p = s.sample_towards_pdf(&r, h.p, h.ng);
            let p_ref = s_ref.sample_towards_pdf(&r, h_ref.p, h_ref.ng);
            assert!((p - p_ref).abs() < 1e-5);
        }
    }

    #[test]
    fn area_equals_plain_object() {
        let (s, s_ref) = spheres();
        assert!((s.area() - s_ref.area()).abs() < crate::EPSILON);
    }
}
