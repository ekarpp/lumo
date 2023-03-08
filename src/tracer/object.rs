use crate::{DVec3, DMat3, DVec2};
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::object::triangle::Triangle;

#[cfg(test)]
mod plane_tests;

pub mod sphere;
pub mod triangle;

/* make sure normal points the right way for triangle/plane/sphere */
fn _orient_normal(n: DVec3, r: &Ray) -> DVec3 {
    if n.dot(r.dir) > 0.0 { -n } else { n }
}

/* given a triangle, mirror the middle point around to get a rectangle.
 * this is a dumb way... the triangle order matters now.*/
fn _triangle_to_rect(abc: DMat3) -> DVec3 {
    abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
}

pub trait Object: Sync {
    /* unit length normal for r at p. only called during hit creation
     * => no need to implement for rectangle or cuboid. */
    fn normal_for_at(&self, _r: &Ray, _p: DVec3) -> DVec3 { DVec3::ZERO }
    fn inside(&self, _r: &Ray) -> bool { false }
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
    fn is_translucent(&self) -> bool { self.material().is_translucent() }
    fn size(&self) -> usize { 1 }
}

pub struct Cuboid {
    rectangles: [Rectangle; 6],
    material: Material,
}

impl Cuboid {
    /* be lazy and construct from two rectangles */
    /* this is overall really hacky. might just want to create one for
     * unit cube and apply affines to it. */
    pub fn new(r1: DMat3, r2: DMat3, m: Material) -> Box<Self> {
        let d1 = _triangle_to_rect(r1);
        Box::new(Self {
            material: m,
            rectangles: [
                /* directions given assuming unit cube */
                *Rectangle::new(
                    r1, /* xz-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r2.col(1),
                        r1.col(1),
                        r1.col(2),
                    ), /* yz-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r1.col(0),
                        r1.col(1),
                        r2.col(1),
                    ), /* xy-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r2.col(0),
                        r1.col(0),
                        d1,
                    ), /* yz-plane + 1z*/
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r2.col(2),
                        r1.col(2),
                        d1,
                    ), /* xy-plane + 1x */
                    Material::Blank,
                ),
                *Rectangle::new(
                    r2, /* y-plane + 1y*/
                    Material::Blank,
                ),
            ],
        })
    }
}

impl Object for Cuboid {
    fn size(&self) -> usize { 6 }

    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        self.rectangles.iter().map(|rect| rect.hit(r))
            .fold(None, |closest, hit| {
                if closest.is_none() || (hit.is_some() && hit < closest) {
                    hit
                } else {
                    closest
                }
            })
            .and_then(|mut hit| {
                /* change us as the object to get correct texture for rendering */
                hit.object = self;
                Some(hit)
            })
    }

}

pub struct Rectangle {
    triangles: (Triangle, Triangle),
    material: Material,
}

impl Rectangle {
    /* consider otherways of doing rectangles?
     * (plane aligned, then transform?? [instances in book])
     * seemed boring, check if ray hits plane then check if inside rect */
    pub fn new(abc: DMat3, m: Material) -> Box<Self>
    {
        /* d is b "mirrored" */
        let d = _triangle_to_rect(abc);
        /* figure out the correct order of points... */
        let t1 = Triangle::new(
            abc.col(0), abc.col(1), abc.col(2), Material::Blank
        );
        let t2 = Triangle::new(abc.col(0), d, abc.col(2), Material::Blank);
        Box::new(Self {
            triangles: (*t1, *t2),
            material: m,
        })
    }
}

impl Object for Rectangle {
    fn size(&self) -> usize { 2 }

    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        self.triangles.0.hit(r).or_else(|| self.triangles.1.hit(r))
            .and_then(|mut hit| {
                /* change us as the object to get correct texture for rendering */
                hit.object = self;
                Some(hit)
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
