use super::*;

pub struct Rectangle {
    triangles: (Triangle, Triangle),
    material: Material,
}

impl Rectangle {
    /* consider otherways of doing rectangles?
     * (plane aligned, then transform?? [instances in book])
     * seemed boring, check if ray hits plane then check if inside rect */

    /// Constructs a rectangle from three points. Fourth point, namely `b`,
    /// is mirrored around the triangle
    ///
    /// # Arguments
    /// * `abc` - Points `a,b,c` stored in the columns
    /// * `norm_dir` - Direction towards which the normal should point
    /// * `material` - Material of the rectangle
    pub fn new(abc: DMat3, norm_dir: DVec3, material: Material) -> Box<Self>
    {
        /* figure out the correct order of points... */
        let t1 = Triangle::new(
            abc.col(0), abc.col(1), abc.col(2), norm_dir, Material::Blank
        );
        let t2 = {
            /* d is b "mirrored" */
            let d = _triangle_to_rect(abc);
            Triangle::new(
                abc.col(0), d, abc.col(2), norm_dir, Material::Blank
            )
        };
        Box::new(Self {
            triangles: (*t1, *t2),
            material,
        })
    }

    /// Choose either of the triangles uniformly at random
    fn choose_triangle(&self) -> &Triangle {
        if rand_utils::rand_f64() > 0.5 {
            &self.triangles.0
        } else {
            &self.triangles.1
        }
    }
}

impl Object for Rectangle {
    fn size(&self) -> usize { 2 }

    fn area(&self) -> f64 { 2.0 * self.triangles.0.area() }

    fn material(&self) -> &Material { &self.material }

    fn sample_towards(&self, p: DVec3, rand_sq: DVec2) -> DVec3 {
        self.choose_triangle().sample_towards(p, rand_sq)
    }

    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        self.choose_triangle().sample_on(rand_sq)
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        self.triangles.0.hit(r).or_else(|| self.triangles.1.hit(r))
            .and_then(|mut hit| {
                /* change us as the object to get correct texture for rendering */
                hit.object = self;
                Some(hit)
            })
    }
}
