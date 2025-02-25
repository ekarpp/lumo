use super::*;

#[cfg(test)]
mod rectangle_tests;

/// Rectangle defined by two triangles
pub struct Rectangle {
    /// Just a mesh...
    mesh: Mesh,
}

impl Rectangle {
    /// Constructs a rectangle from three points. Fourth point, namely `b`,
    /// is mirrored around the triangle
    ///
    /// # Arguments
    /// * `abc` - Points `a,b,c` stored in the columns. Normal in CCW
    /// * `material` - Material of the rectangle
    pub fn new(abc: Mat3, material: Material) -> Box<Self> {
        let vertices = vec![
            abc.col(0), abc.col(1), abc.col(2), Self::_triangle_to_rect(abc)
        ];

        let faces = vec![Face::new(vec![0,1,2,3], vec![], vec![])];

        Box::new(Self {
            mesh: TriangleMesh::new(vertices, faces, vec![], vec![], material)
        })
    }

    /// Given a triangle, with points read from the columns of the matrix `abc`,
    /// returns `b` mirrored to define a rectangle.
    fn _triangle_to_rect(abc: Mat3) -> Point {
        let a = abc.col(0);
        let b = abc.col(1);
        let c = abc.col(2);

        b + (a - b) + (c - b)
    }
}

impl Object for Rectangle {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self.mesh.hit(r, t_min, t_max)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self.mesh.hit_t(r, t_min, t_max)
    }
}

impl Sampleable for Rectangle {
    fn area(&self) -> Float {
        self.mesh.area()
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        self.mesh.sample_on(rand_sq)
    }
}
