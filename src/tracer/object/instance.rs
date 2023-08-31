use super::*;

#[cfg(test)]
mod instance_tests;

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
        let normal_transform = inv_transform.matrix3.transpose().into();

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

        self.object.hit(&ray_local, t_min, t_max).map(|mut h| {
            h.ns = (self.normal_transform * h.ns).normalize();
            h.ng = (self.normal_transform * h.ng).normalize();

            let err = efloat::gamma(3) * Vec3::new(
                (Vec3::from(self.transform.matrix3.row(0)) * h.p)
                    .abs().dot(Vec3::ONE) + self.transform.translation.x.abs(),
                (Vec3::from(self.transform.matrix3.row(1)) * h.p)
                    .abs().dot(Vec3::ONE) + self.transform.translation.y.abs(),
                (Vec3::from(self.transform.matrix3.row(2)) * h.p)
                    .abs().dot(Vec3::ONE) + self.transform.translation.z.abs(),
            );

            h.p = self.transform.transform_point3(h.p);
            // TODO: just add them for now...
            h.fp_error += err;
            h
        })
    }
}

impl<T: Sampleable> Sampleable for Instance<T> {
    fn area(&self) -> Float {
        self.object.area();
        todo!()
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let mut ho = self.object.sample_on(rand_sq);

        ho.ng = self.normal_transform * ho.ng;
        ho.ns = self.normal_transform * ho.ns;
        ho.p = self.transform.transform_point3(ho.p);

        ho
    }

    fn sample_towards(&self, xo: Point, rand_sq: Vec2) -> Direction {
        let xo_local = self.inv_transform.transform_point3(xo);
        let dir_local = self.object.sample_towards(xo_local, rand_sq);

        self.transform.transform_vector3(dir_local)
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> (Float, Option<Hit>) {
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
            let height = ng.dot(self.transform.matrix3 * ng_local).abs();
            let volume = self.transform.matrix3.determinant().abs();
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
    fn translate(self, x: Float, y: Float, z: Float) -> Box<Instance<T>>;

    /// Apply scale `xyz`
    fn scale(self, x: Float, y: Float, z: Float) -> Box<Instance<T>>;

    /// Rotate around x-axis by `r` radians
    fn rotate_x(self, r: Float) -> Box<Instance<T>>;

    /// Rotate around y-axis by `r` radians
    fn rotate_y(self, r: Float) -> Box<Instance<T>>;

    /// Rotate around z-axis by `r` radians
    fn rotate_z(self, r: Float) -> Box<Instance<T>>;

    /// Rotate around `axis` by `r` radisn
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
