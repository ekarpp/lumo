#![allow(dead_code)]
use super::*;

/// Instance of an object i.e. an object to which affine transformations have
/// been applied
pub struct Instance<T> {
    object: T,
    transform: DAffine3,
    inv_transform: DAffine3,
    normal_transform: DMat3,
}

impl<T> Instance<T> {
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

impl<T: Object> Object for Instance<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let ray_local = Ray::new(
            self.inv_transform.transform_point3(r.origin),
            self.inv_transform.transform_vector3(r.dir),
        );

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

    /// Apply x-rotation AFTER current transformations
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
