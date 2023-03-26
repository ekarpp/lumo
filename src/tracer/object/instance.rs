#![allow(dead_code)]
use super::*;

/// Instance of an object i.e. an object to which affine transformations have
/// been applied
pub struct Instance<T> {
    /// Object to be instanced
    object: T,
    /// Transformation applied to the object
    transform: DAffine3,
    /// Inverse transformation
    inv_transform: DAffine3,
    /// Transformation for normals.
    /// Transpose of `inv_transform` without translation.
    normal_transform: DMat3,
}

impl<T> Instance<T> {
    /// Constructs an instance of `object` that is transformed with
    /// `transform`.
    pub fn new(object: T, transform: DAffine3) -> Self {
        let inv_transform = transform.inverse();
        let normal_transform = inv_transform.matrix3.transpose();

        Self {
            object,
            transform,
            inv_transform,
            normal_transform,
        }
    }

    /// Helper to make boxing easy
    pub fn make_box(self) -> Box<Instance<T>> {
        Box::new(self)
    }
}

impl<T: Bounded> Instance<T> {
    /// Translate `self`, such that bounding box center is at the origin
    pub fn to_origin(self) -> Instance<T> {
        let AaBoundingBox { ax_min, ax_max } = self.bounding_box();
        let ax_mid = (ax_min + ax_max) / 2.0;

        self.translate(-ax_mid)
    }
}

impl<T: Bounded> Bounded for Instance<T> {
    fn bounding_box(&self) -> AaBoundingBox {
        let AaBoundingBox { ax_min, ax_max } = self.object.bounding_box();
        let v0 = self.transform
            .transform_point3(DVec3::new(ax_min.x, ax_min.y, ax_min.z));
        let v1 = self.transform
            .transform_point3(DVec3::new(ax_min.x, ax_min.y, ax_max.z));
        let v2 = self.transform
            .transform_point3(DVec3::new(ax_min.x, ax_max.y, ax_min.z));
        let v3 = self.transform
            .transform_point3(DVec3::new(ax_min.x, ax_max.y, ax_max.z));
        let v4 = self.transform
            .transform_point3(DVec3::new(ax_max.x, ax_min.y, ax_min.z));
        let v5 = self.transform
            .transform_point3(DVec3::new(ax_max.x, ax_min.y, ax_max.z));
        let v6 = self.transform
            .transform_point3(DVec3::new(ax_max.x, ax_max.y, ax_min.z));
        let v7 = self.transform
            .transform_point3(DVec3::new(ax_max.x, ax_max.y, ax_max.z));

        AaBoundingBox::new(
            v0.min(v1).min(v2).min(v3).min(v4).min(v5).min(v6).min(v7),
            v0.max(v1).max(v2).max(v3).max(v4).max(v5).max(v6).max(v7),
        )
    }
}

impl<T: Object> Object for Instance<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let ray_local = r.transform(self.inv_transform);

        self.object.hit(&ray_local, t_min, t_max)
            .map(|mut h| {
                h.norm = (self.normal_transform * h.norm).normalize();
                // something smarter should be done here...
                h.p = self.transform.transform_point3(h.p);
                h
            })
    }

    fn material(&self) -> &Material { self.object.material() }
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 { panic!() }
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray { panic!() }
    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 { panic!() }

}

/// Object that can be instanced
pub trait Instanceable<T> {
    /// Translate object by `t`
    fn translate(self, t: DVec3) -> Instance<T>;

    /// Apply scale `s`
    fn scale(self, s: DVec3) -> Instance<T>;

    /// Rotate around x-axis by `r` radians
    fn rotate_x(self, r: f64) -> Instance<T>;

    /// Rotate around y-axis by `r` radians
    fn rotate_y(self, r: f64) -> Instance<T>;

    /// Rotate around z-axis by `r` radians
    fn rotate_z(self, r: f64) -> Instance<T>;
}

/// To make applying transformations to objects easy
impl<T: Object> Instanceable<T> for T {
    fn translate(self, t: DVec3) -> Instance<T> {
        Instance::new(self, DAffine3::from_translation(t))
    }

    fn scale(self, s: DVec3) -> Instance<T> {
        Instance::new(self, DAffine3::from_scale(s))
    }

    fn rotate_x(self, r: f64) -> Instance<T> {
        Instance::new(self, DAffine3::from_rotation_x(r))
    }

    fn rotate_y(self, r: f64) -> Instance<T> {
        Instance::new(self, DAffine3::from_rotation_y(r))
    }

    fn rotate_z(self, r: f64) -> Instance<T> {
        Instance::new(self, DAffine3::from_rotation_z(r))
    }
}

/// Prevent nested Instance structs
impl<T: Object> Instance<T> {
    /// Apply translation AFTER curret transformations
    pub fn translate(self, t: DVec3) -> Self {
        Self::new(
            self.object,
            DAffine3::from_translation(t) * self.transform,
        )
    }

    /// Apply scale AFTER current transformations
    pub fn scale(self, s: DVec3) -> Self {
        Self::new(
            self.object,
            DAffine3::from_scale(s) * self.transform,
        )
    }

    /// Apply x-rotation AFTER current transformations.
    /// Looking at positive x, rotation in clockwise direction.
    pub fn rotate_x(self, r: f64) -> Self {
        Self::new(
            self.object,
            DAffine3::from_rotation_x(r) * self.transform,
        )
    }

    /// Apply y-rotation AFTER current transformations
    pub fn rotate_y(self, r: f64) -> Self {
        Self::new(
            self.object,
            DAffine3::from_rotation_y(r) * self.transform,
        )
    }

    /// Apply z-rotation AFTER current transformations
    pub fn rotate_z(self, r: f64) -> Self {
        Self::new(
            self.object,
            DAffine3::from_rotation_z(r) * self.transform,
        )
    }
}
