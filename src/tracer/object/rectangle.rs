use super::*;

/// Given a triangle, with points read from the columns of the matrix `abc`,
/// returns `b` mirrored to define a rectangle.
fn _triangle_to_rect(abc: DMat3) -> DVec3 {
    abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
}

/// Rectangle defined by two triangles
pub struct Rectangle {
    triangles: (Triangle, Triangle),
    material: Material,
}

impl Rectangle {
    /// Constructs a rectangle from three points. Fourth point, namely `b`,
    /// is mirrored around the triangle
    ///
    /// # Arguments
    /// * `abc` - Points `a,b,c` stored in the columns
    /// * `norm_dir` - Direction towards which the normal should point
    /// * `material` - Material of the rectangle
    pub fn new(abc: DMat3, material: Material) -> Box<Self>
    {
        /* figure out the correct order of points... */
        let t1 = Triangle::new(
            (abc.col(0), abc.col(1), abc.col(2)), Material::Blank
        );
        let t2 = {
            /* d is b "mirrored" */
            let d = _triangle_to_rect(abc);
            Triangle::new(
                (abc.col(2), d, abc.col(0)), Material::Blank
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
    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        self.triangles.0.hit(r, t_min, t_max)
            .or_else(|| self.triangles.1.hit(r, t_min, t_max))
            .map(|mut h| {
                h.object = self;
                h
            })
    }

    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        self.choose_triangle().sample_towards(xo, rand_sq)
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> f64 {
        /* ray can hit either of the triangles. sum pdf from both
         * (if miss, pdf = 0) and divide by two. (rectangle = two identical
         * triangles => area two times bigger) */
        (self.triangles.0.sample_towards_pdf(ri)
         + self.triangles.1.sample_towards_pdf(ri))
            / 2.0
    }

    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        self.choose_triangle().sample_on(rand_sq)
    }
}
