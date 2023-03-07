use crate::{DVec3, DMat3};
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;

#[cfg(test)]
mod sphere_tests;
#[cfg(test)]
mod plane_tests;
#[cfg(test)]
mod triangle_tests;

pub trait Object: Sync {
    // unit length normal at p
    fn normal_at(&self, p: DVec3) -> DVec3;
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
}

/* FIGURE OUT BEING INSIDE. CANT BE INSIDE PLANES OR TRIANGLES. (OR RECTANGLES)*/

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
    fn material(&self) -> &Material {
        &self.material
    }

    fn normal_at(&self, p: DVec3) -> DVec3 {
        self.tria.0.normal_at(p)
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        self.tria.0.hit(r).or_else(|| self.tria.1.hit(r))
            .and_then(|mut h: Hit| {
                h.object = self;
                Some(h)
            })
    }
}

/* barycentir interpolation ~ different texture at each point of triangle */
/* normal inside?? */
pub struct Triangle {
    a: DVec3,
    b: DVec3,
    c: DVec3,
    norm: DVec3,
    material: Material,
}

impl Triangle {
    /* assume non-degenerate */
    pub fn new(a: DVec3, b: DVec3, c: DVec3, m: Material) -> Box<Self> {
        let norm = -(b - a).cross(c - a).normalize();

        Box::new(Self {
            a: a,
            b: b,
            c: c,
            norm: norm,
            material: m,
        })
    }
}

impl Object for Triangle {
    fn material(&self) -> &Material { &self.material }

    fn normal_at(&self, _p: DVec3) -> DVec3 { self.norm }

    /* barycentric triangle intersection with Cramer's rule */
    fn hit(&self, r: &Ray) -> Option<Hit> {
        /* can store a-c, a-b, and a instead. saves some computation.
         * compiler should do it? */

        let mat_a = DMat3::from_cols(
            self.a - self.b,
            self.a - self.c,
            r.dir
        );

        let det_a = mat_a.determinant();

        if det_a.abs() < crate::EPSILON {
            return None;
        }

        let vec_b = self.a - r.origin;

        let beta = DMat3::from_cols(
            vec_b,
            mat_a.col(1),
            mat_a.col(2),
        ).determinant() / det_a;

        let gamma = DMat3::from_cols(
            mat_a.col(0),
            vec_b,
            mat_a.col(2),
        ).determinant() / det_a;

        if beta < 0.0 || gamma < 0.0
            || beta + gamma > 1.0 {
            return None;
        }

        let t = DMat3::from_cols(
            mat_a.col(0),
            mat_a.col(1),
            vec_b,
        ).determinant() / det_a;

        if t < crate::EPSILON {
            None
        } else {
            Hit::new(
                t,
                self,
                r.at(t),
            )
        }
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
    fn normal_at(&self, _p: DVec3) -> DVec3 { self.norm }

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
                r.at(t),
            )
        }
    }
}

pub struct Sphere {
    pub origin: DVec3,
    pub radius: f64,
    material: Material,
}

impl Sphere {
    /* assume r != 0 */
    pub fn new(origin: DVec3, r: f64, mat: Material) -> Box<Self> {
        Box::new(Self {
            origin: origin,
            radius: r,
            material: mat,
        })
    }
}

impl Object for Sphere {
    fn material(&self) -> &Material { &self.material }

    fn normal_at(&self, p: DVec3) -> DVec3 {
        (p - self.origin) / self.radius
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        // .dot faster than .length_squared, recheck
        let a = r.dir.dot(r.dir);
        let half_b = tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius*self.radius;
        let disc = half_b*half_b - a*c;

        if disc < 0.0 {
            return None;
        }
        let disc_root = disc.sqrt();
        let mut t = (-half_b - disc_root) / a;
        if t < crate::EPSILON {
            t = (-half_b + disc_root) / a;
            if t < crate::EPSILON {
                return None;
            }
        }
        Hit::new(
            t,
            self,
            r.at(t),
        )
     }
}
