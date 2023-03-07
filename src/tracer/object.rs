use crate::{DVec3, DMat3};
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::object::triangle::Triangle;

#[cfg(test)]
mod sphere_tests;
#[cfg(test)]
mod plane_tests;
#[cfg(test)]
mod triangle_tests;

pub mod sphere;
pub mod triangle;

/* make sure normal points the right way for triangle/plane/sphere */
fn _orient_normal(n: DVec3, r: &Ray) -> DVec3 {
    if n.dot(r.dir) > 0.0 { -n } else { n }
}

pub trait Object: Sync {
    // unit length normal at p
    fn normal_for_at(&self, r: &Ray, p: DVec3) -> DVec3;
    fn inside(&self, _r: &Ray) -> bool { false }
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
}

pub struct Cuboid {

}

pub struct Rectangle {
    tria: (Triangle, Triangle),
    material: Material,
}

impl Rectangle {
    /* consider otherways of doing rectangles?
     * (plane aligned, then transform?? [instances in book])
     * seemed boring, check if ray hits plane then check if inside rect */
    pub fn new(a: DVec3, b: DVec3, c: DVec3, m: Material) -> Box<Self>
    {
        /* d is b "mirrored" */
        let d = b + (a - b) + (c - b);
        /* figure out the correct order of points... */
        let t1 = Triangle::new(a, b, c, Material::Blank);
        let t2 = Triangle::new(a, d, c, Material::Blank);
        Box::new(Self {
            tria: (*t1, *t2),
            material: m,
        })
    }
}

impl Object for Rectangle {
    fn material(&self) -> &Material { &self.material }

    fn normal_for_at(&self, r: &Ray, p: DVec3) -> DVec3 {
        self.tria.0.normal_for_at(r, p)
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        self.tria.0.hit(r).or_else(|| self.tria.1.hit(r))
            .and_then(|mut h: Hit| {
                h.object = self;
                Some(h)
            })
    }
}

pub struct Plane {
    norm: DVec3,
    material: Material,
    d: f64, // for hit calc, store instead of point
}

impl Plane {
    /* assume n != 0 */
    pub fn new(p: DVec3, n: DVec3, m: Material) -> Box<Self> {
        let norm = n.normalize();
        Box::new(Self {
            norm: norm,
            material: m,
            d: p.dot(-norm),
        })
    }
}

impl Object for Plane {
    fn material(&self) -> &Material { &self.material }
    // check that point is on plane?? or assume we are smart
    fn normal_for_at(&self, r: &Ray, _p: DVec3) -> DVec3 {
        _orient_normal(self.norm, r)
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        /* check if plane and ray are parallel. use epsilon instead?
         * or fail only if we get div by zero?? */
        if self.norm.dot(r.dir) == 0.0 {
            return None;
        }

        let t = -(self.d + self.norm.dot(r.origin)) / self.norm.dot(r.dir);
        if t < crate::EPSILON {
            None
        } else {
            Hit::new(
                t,
                self,
                r,
            )
        }
    }
}
