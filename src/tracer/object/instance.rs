use super::*;

#[cfg(test)]
mod instance_tests;

/// Instance of an object i.e. an object to which affine transformations have
/// been applied
pub struct Instance<T> {
    /// Object to be instanced
    object: T,
    /// Transformation from local to world
    transform: DAffine3,
    /// Transformation from world to local
    inv_transform: DAffine3,
    /// Transformation for normals from local to world.
    /// Transpose of `inv_transform` without translation.
    normal_transform: DMat3,
}

impl<T> Instance<T> {
    /// Constructs an instance of `object` that is transformed with
    /// `transform`.
    pub fn new(object: T, transform: DAffine3) -> Box<Self> {
        let inv_transform = transform.inverse();
        let normal_transform = inv_transform.matrix3.transpose();

        Box::new(Self {
            object,
            transform,
            inv_transform,
            normal_transform,
        })
    }
}

impl<T: Bounded> Instance<T> {
    /// Translate `self`, such that bounding box center is at the origin
    pub fn to_origin(self) -> Box<Self> {
        let AaBoundingBox { ax_min, ax_max } = self.bounding_box();
        let ax_mid = -(ax_min + ax_max) / 2.0;

        self.translate(ax_mid.x, ax_mid.y, ax_mid.z)
    }
}

impl<T: Bounded> Bounded for Instance<T> {
    fn bounding_box(&self) -> AaBoundingBox {
        /* Graphics Gems I, TRANSFORMING AXIS-ALIGNED BOUNDING BOXES */
        let mut ax_min = DVec3::ZERO;
        let mut ax_max = DVec3::ZERO;
        let aabb = self.object.bounding_box();

        let a0 = self.transform.matrix3.row(0) * aabb.min(Axis::X);
        let b0 = self.transform.matrix3.row(0) * aabb.max(Axis::X);
        ax_min.x += a0.min(b0).dot(DVec3::ONE);
        ax_max.x += a0.max(b0).dot(DVec3::ONE);

        let a1 = self.transform.matrix3.row(1) * aabb.min(Axis::Y);
        let b1 = self.transform.matrix3.row(1) * aabb.max(Axis::Y);
        ax_min.y += a1.min(b1).dot(DVec3::ONE);
        ax_max.y += a1.max(b1).dot(DVec3::ONE);

        let a2 = self.transform.matrix3.row(2) * aabb.min(Axis::Z);
        let b2 = self.transform.matrix3.row(2) * aabb.max(Axis::Z);
        ax_min.z += a2.min(b2).dot(DVec3::ONE);
        ax_max.z += a2.max(b2).dot(DVec3::ONE);

        // translate
        ax_min += self.transform.translation;
        ax_max += self.transform.translation;

        AaBoundingBox::new(ax_min, ax_max)
    }
}

impl<T: Object> Object for Instance<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        // inner object is in world coordinates. hence apply inverse
        // transformation to ray instead of transformation to object.
        let ray_local = r.transform(self.inv_transform);

        self.object.hit(&ray_local, t_min, t_max).map(|mut h| {
            h.ns = (self.normal_transform * h.ns).normalize();
            h.ng = (self.normal_transform * h.ng).normalize();
            h.p = r.at(h.t);
            h.object = self;
            h
        })
    }

    fn material(&self) -> &Material {
        self.object.material()
    }
}

impl<T: Sampleable> Sampleable for Instance<T> {
    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let sample_local = self.object.sample_on(rand_sq);

        self.transform.transform_point3(sample_local)
    }

    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> DVec3 {
        let xo_local = self.inv_transform.transform_point3(xo);
        let dir_local = self.object.sample_towards(xo_local, rand_sq);

        self.transform.transform_vector3(dir_local)
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> (f64, Option<Hit>) {
        let ri_local = ri.transform(self.inv_transform);
        let (pdf_local, hi_local) = self.object.sample_towards_pdf(&ri_local);
        if let Some(mut hi) = hi_local {
            let ng_local = hi.ng;

            let xi = ri.at(hi.t);
            let ng = (self.normal_transform * ng_local).normalize();

            // object pdf just needs these in world coordinates
            hi.p = xi;
            hi.ng = ng;

            // imagine a unit cube at the sampled point with the surface normal
            // at that point as one of the cube edges. apply the linear
            // transformation to the cube and we get a parallellepiped.
            // the base of the parallellepiped gives us the area scale at the
            // point of impact. think this is not exact with ansiotropic
            // scaling of solids. how to do for solid angle?
            let height = ng.dot(self.transform.matrix3 * ng_local);
            let volume = self.transform.matrix3.determinant();
            let jacobian = volume / height;

            // p_y(y) = p_y(T(x)) = p_x(x) / |J_T(x)|
            (pdf_local / jacobian, Some(hi))
        } else {
            (0.0, None)
        }
    }
}

/// Object that can be instanced
pub trait Instanceable<T> {
    /// Translate object by `xyz`
    fn translate(self, x: f64, y: f64, z: f64) -> Box<Instance<T>>;

    /// Apply scale `xyz`
    fn scale(self, x: f64, y: f64, z: f64) -> Box<Instance<T>>;

    /// Rotate around x-axis by `r` radians
    fn rotate_x(self, r: f64) -> Box<Instance<T>>;

    /// Rotate around y-axis by `r` radians
    fn rotate_y(self, r: f64) -> Box<Instance<T>>;

    /// Rotate around z-axis by `r` radians
    fn rotate_z(self, r: f64) -> Box<Instance<T>>;
}

/// To make applying transformations to objects easy
impl<T: Object> Instanceable<T> for T {
    fn translate(self, x: f64, y: f64, z: f64) -> Box<Instance<T>> {
        let t = DVec3::new(x, y, z);
        Instance::new(self, DAffine3::from_translation(t))
    }

    fn scale(self, x: f64, y: f64, z: f64) -> Box<Instance<T>> {
        assert!(x * y * z != 0.0);
        let s = DVec3::new(x, y, z);
        Instance::new(self, DAffine3::from_scale(s))
    }

    fn rotate_x(self, r: f64) -> Box<Instance<T>> {
        Instance::new(self, DAffine3::from_rotation_x(r))
    }

    fn rotate_y(self, r: f64) -> Box<Instance<T>> {
        Instance::new(self, DAffine3::from_rotation_y(r))
    }

    fn rotate_z(self, r: f64) -> Box<Instance<T>> {
        Instance::new(self, DAffine3::from_rotation_z(r))
    }
}

/// Prevent nested Instance structs
impl<T: Object> Instance<T> {
    /// Apply translation AFTER curret transformations
    pub fn translate(self, x: f64, y: f64, z: f64) -> Box<Self> {
        let t = DVec3::new(x, y, z);
        Self::new(self.object, DAffine3::from_translation(t) * self.transform)
    }

    /// Apply scale AFTER current transformations
    pub fn scale(self, x: f64, y: f64, z: f64) -> Box<Self> {
        assert!(x * y * z != 0.0);
        let s = DVec3::new(x, y, z);
        Self::new(self.object, DAffine3::from_scale(s) * self.transform)
    }

    /// Apply x-rotation AFTER current transformations.
    /// Looking at positive x, rotation in clockwise direction.
    pub fn rotate_x(self, r: f64) -> Box<Self> {
        Self::new(self.object, DAffine3::from_rotation_x(r) * self.transform)
    }

    /// Apply y-rotation AFTER current transformations
    pub fn rotate_y(self, r: f64) -> Box<Self> {
        Self::new(self.object, DAffine3::from_rotation_y(r) * self.transform)
    }

    /// Apply z-rotation AFTER current transformations
    pub fn rotate_z(self, r: f64) -> Box<Self> {
        Self::new(self.object, DAffine3::from_rotation_z(r) * self.transform)
    }
}
