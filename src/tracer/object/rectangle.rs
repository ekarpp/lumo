use super::*;

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

    fn area(&self) -> f64 { 2.0 * self.triangles.0.area() }

    fn material(&self) -> &Material { &self.material }

    fn sample_from(&self, p: DVec3, rand_sq: DVec2) -> DVec3 {
        if rand_utils::rand_f64() > 0.5 {
            self.triangles.0.sample_from(p, rand_sq)
        } else {
            self.triangles.1.sample_from(p, rand_sq)
        }
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
