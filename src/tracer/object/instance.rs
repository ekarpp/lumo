use super::*;

/// Instance of an object i.e. an object to which affine transformations have
/// been applied
pub struct Instance<T> {
    /// Object to be instanced
    object: Arc<T>,
    /// Transformation from local to world
    transform: Transform,
    /// Transformation for normals from local to world.
    /// Transpose of `inv_transform` without translation.
    normal_transform: Mat3,
    /// Optional material to use for the instance
    material: Option<Material>,
}

impl<T> Instance<T> {
    /// Constructs an instance of `object` that is transformed with
    /// `transform`.
    pub fn new(object: T) -> Self {
        let object = Arc::new(object);
        let transform = Transform::default();
        let normal_transform = transform.to_normal();
        let material = None;

        Self { object, transform, normal_transform, material }
    }

    /// Clone the instance without cloning the underlying object
    pub fn clone(&self, material: Option<Material>) -> Self {
        Self {
            material,
            object: Arc::clone(&self.object),
            transform: self.transform.clone(),
            normal_transform: self.normal_transform.clone(),
        }
    }

    fn propagate_fp_err(&self, xo: Point, fp_error: Vec3) -> Vec3 {
        let e3 = fp_error.abs();
        let p3 = xo.abs();

        if e3.x == 0.0 && e3.y == 0.0 && e3.z == 0.0 {
            efloat::gamma(3) * self.transform.abs().transform_pt(p3)
        } else {
            efloat::gamma(3) * self.transform.abs().transform_pt(p3)
                + (efloat::gamma(3) + 1.0) * self.transform.abs().transform_dir(e3)
        }
    }
}

impl<T: Object> Instance<T> {
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

impl<T: Object> Object for Instance<T> {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        // inner object is in world coordinates. hence apply inverse
        // transformation to ray instead of transformation to object.
        let ray_local = r.transform::<false>(&self.transform);

        self.object.hit(&ray_local, t_min, t_max)
            .map(|mut h| {
                h.ns = (self.normal_transform.mul_vec3(h.ns)).normalize();
                h.ng = (self.normal_transform.mul_vec3(h.ng)).normalize();

                h.fp_error = self.propagate_fp_err(h.p, h.fp_error);

                if let Some(material) = self.material.as_ref() {
                    h.material = material;
                }

                h.p =  self.transform.transform_pt(h.p);
                h
            })
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        let r_local = r.transform::<false>(&self.transform);
        self.object.hit_t(&r_local, t_min, t_max)
    }

    fn bounding_box(&self) -> AaBoundingBox {
        /* Graphics Gems I, TRANSFORMING AXIS-ALIGNED BOUNDING BOXES */
        let mut ax_min = self.transform.to_translation();
        let mut ax_max = self.transform.to_translation();
        let aabb = self.object.bounding_box();

        for axis in [Axis::X, Axis::Y, Axis::Z] {
            let ri = self.transform.row(axis as usize).truncate();
            let a0 = ri * aabb.ax_min;
            let a1 = ri * aabb.ax_max;
            let mi = a0.min(a1).dot(Vec3::ONE);
            let mx = a0.max(a1).dot(Vec3::ONE);
            match axis {
                Axis::X => { ax_min.x += mi; ax_max.x += mx; }
                Axis::Y => { ax_min.y += mi; ax_max.y += mx; }
                Axis::Z => { ax_min.z += mi; ax_max.z += mx; }
            }
        }

        AaBoundingBox::new(ax_min, ax_max)
    }

    fn num_primitives(&self) -> usize { self.object.num_primitives() }
}

impl<T: Sampleable> Sampleable for Instance<T> {
    fn area(&self) -> Float {
        let scale = self.transform.to_scale();

        // only allow uniform scale
        if (scale.x - scale.y).abs() + (scale.y - scale.z).abs() > crate::EPSILON {
            unimplemented!();
            // TODO: add warning for non-uniform scaling of lights
        }

        scale.x * scale.y * self.object.area()
    }

    fn material(&self) -> &Material { self.object.material() }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let mut ho = self.object.sample_on(rand_sq);

        ho.ng = self.normal_transform.mul_vec3(ho.ng);
        ho.ns = self.normal_transform.mul_vec3(ho.ns);
        ho.p = self.transform.transform_pt(ho.p);
        ho.fp_error = self.propagate_fp_err(ho.p, ho.fp_error);

        if let Some(material) = self.material.as_ref() {
            ho.material = material;
        }

        ho
    }

    fn sample_towards(&self, xo: Point, rand_sq: Vec2) -> Direction {
        let xo_local = self.transform.transform_pt_inv(xo);
        let dir_local = self.object.sample_towards(xo_local, rand_sq);

        self.transform.transform_dir(dir_local).normalize()
    }

    fn sample_towards_pdf(&self, ri: &Ray, xi: Point, ng: Normal) -> Float {
        let normal_transform_inverse = self.normal_transform.inv().transpose();
        let ng_local = (normal_transform_inverse.mul_vec3(ng)).normalize();
        let xi_local = self.transform.transform_pt_inv(xi);
        let ri_local = ri.transform::<true>(&self.transform);

        let wi = ri.dir;
        let wi_local = ri_local.dir;

        let xo = ri.origin;
        let xo_local = ri_local.origin;

        let pdf_local = self.object.sample_towards_pdf(&ri_local, xi_local, ng_local);

        // imagine a unit cube at the sampled point with the surface normal
        // at that point as one of the cube edges. apply the linear
        // transformation to the cube and we get a parallellepiped.
        // the base of the parallellepiped gives us the area scale at the
        // point of impact. think this is not exact with ansiotropic
        // scaling of solids.
        let height = ng.dot(self.transform.transform_dir(ng_local)).abs();
        let volume = self.transform.to_mat3().det().abs();
        let jacobian = volume / height;

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
}

/// To make applying transformations to objects easy
impl<T: Object> Instanceable<T> for T {
    fn translate(self, x: Float, y: Float, z: Float) -> Box<Instance<T>> {
        Instance::new(self)
            .translate(x, y, z)
    }

    fn scale(self, x: Float, y: Float, z: Float) -> Box<Instance<T>> {
        assert!(x * y * z != 0.0);
        Instance::new(self)
            .scale(x, y, z)
    }

    fn scale_uniform(self, s: Float) -> Box<Instance<T>> {
        self.scale(s, s, s)
    }

    fn rotate_x(self, r: Float) -> Box<Instance<T>> {
        Instance::new(self)
            .rotate_x(r)
    }

    fn rotate_y(self, r: Float) -> Box<Instance<T>> {
        Instance::new(self)
            .rotate_y(r)
    }

    fn rotate_z(self, r: Float) -> Box<Instance<T>> {
        Instance::new(self)
            .rotate_z(r)
    }
}

/// Prevent nested Instance structs
impl<T: Object> Instance<T> {
    /// Apply translation AFTER curret transformations
    pub fn translate(mut self, x: Float, y: Float, z: Float) -> Box<Instance<T>> {
        self.transform = Transform::translation(x, y, z) * self.transform;
        self.normal_transform = self.transform.to_normal();
        Box::new(self)
    }

    /// Apply scale AFTER current transformations
    pub fn scale(mut self, x: Float, y: Float, z: Float) -> Box<Instance<T>> {
        assert!(x * y * z != 0.0);
        self.transform = Transform::scale(x, y, z) * self.transform;
        self.normal_transform = self.transform.to_normal();
        Box::new(self)
    }

    /// Apply uniform scale AFTER current transformations
    pub fn scale_uniform(self, s: Float) -> Box<Instance<T>> {
        self.scale(s, s, s)
    }

    /// Apply x-rotation AFTER current transformations.
    /// Looking at positive x, rotation in clockwise direction.
    pub fn rotate_x(mut self, r: Float) -> Box<Instance<T>> {
        self.transform = Transform::rotate_x(r) * self.transform;
        self.normal_transform = self.transform.to_normal();
        Box::new(self)
    }

    /// Apply y-rotation AFTER current transformations
    pub fn rotate_y(mut self, r: Float) -> Box<Instance<T>> {
        self.transform = Transform::rotate_y(r) * self.transform;
        self.normal_transform = self.transform.to_normal();
        Box::new(self)
    }

    /// Apply z-rotation AFTER current transformations
    pub fn rotate_z(mut self, r: Float) -> Box<Instance<T>> {
        self.transform = Transform::rotate_z(r) * self.transform;
        self.normal_transform = self.transform.to_normal();
        Box::new(self)
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

        for _ in 0..NUM_SAMPLES {
            let xo = 5.0 * rng::maps::square_to_sphere(rng.gen_vec2());
            let wi = s.sample_towards(xo, rng.gen_vec2());
            let r = Ray::new(xo, wi);
            let Some(h) = s.hit(&r, 0.0, crate::INF) else { panic!() };
            let Some(h_ref) = s_ref.hit(&r, 0.0, crate::INF) else { panic!() };
            let p = s.sample_towards_pdf(&r, h.p, h.ng);
            let p_ref = s_ref.sample_towards_pdf(&r, h_ref.p, h_ref.ng);

            assert!((p - p_ref).abs() < crate::EPSILON.sqrt());
        }
    }

    #[test]
    fn area_equals_plain_object() {
        let (s, s_ref) = spheres();
        assert!((s.area() - s_ref.area()).abs() < crate::EPSILON);
    }
}
