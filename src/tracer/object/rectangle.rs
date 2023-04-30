use super::*;

#[cfg(test)]
mod rectangle_tests;



/// Rectangle defined by two triangles
pub struct Rectangle<'a> {
    /// Just a mesh...
    mesh: TriangleMesh<'a>
}

impl Rectangle<'_> {
    /// Constructs a rectangle from three points. Fourth point, namely `b`,
    /// is mirrored around the triangle
    ///
    /// # Arguments
    /// * `abc` - Points `a,b,c` stored in the columns. Normal in CCW
    /// * `material` - Material of the rectangle
    pub fn new(abc: DMat3, material: Material) -> Box<Self> {
        let vertices = vec![
            abc.col(0), abc.col(1), abc.col(2), Self::_triangle_to_rect(abc)
        ];

        let faces = vec![Face::new(vec![0,1,2,3], vec![], vec![])];

        Box::new(Self {
            mesh: *TriangleMesh::new(vertices, faces, vec![], vec![], material)
        })
    }

    /// Given a triangle, with points read from the columns of the matrix `abc`,
    /// returns `b` mirrored to define a rectangle.
    fn _triangle_to_rect(abc: DMat3) -> DVec3 {
        abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
    }
}

impl Object for Rectangle<'_> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        self.mesh.hit(r, t_min, t_max)
    }
}

impl Sampleable for Rectangle<'_> {
    fn area(&self) -> f64 {
        self.mesh.area()
    }

    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        self.mesh.sample_on(rand_sq)
    }
}
