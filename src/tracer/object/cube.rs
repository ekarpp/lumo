use super::*;

#[cfg(test)]
mod cube_tests;

/// A unit cube consisting of 6 squares
pub struct Cube {
    /// Just a mesh...
    mesh: Mesh
}

impl Cube {
    /// Constructs an unit cube. To get the desired shape, one should instance
    /// this.
    ///
    /// # Arguments
    /// * `material` - Material of the cube
    pub fn new(material: Material) -> Box<Self> {
        let vertices = vec![
            Point::X, Point::ZERO,         Point::Y,   Point::X + Point::Y,
            Point::Z, Point::Z + Point::X, Point::ONE, Point::Z + Point::Y,
        ];

        let faces = vec![
            // xy, z = 0
            Face::new(vec![0, 1, 2, 3], vec![], vec![]),
            // xy, z = 1
            Face::new(vec![4, 5, 6, 7], vec![], vec![]),
            // yz, x = 0
            Face::new(vec![1, 4, 7, 2], vec![], vec![]),
            // yz, x = 1
            Face::new(vec![5, 0, 3, 6], vec![], vec![]),
            // xz, y = 0
            Face::new(vec![0, 5, 4, 1], vec![], vec![]),
            // xz, y = 1
            Face::new(vec![6, 3, 2, 7], vec![], vec![]),
        ];

        Box::new(Self {
            mesh: TriangleMesh::new(vertices, faces, vec![], vec![], material)
        })
    }
}

impl Bounded for Cube {
    fn bounding_box(&self) -> AaBoundingBox {
        // we only support unit cubes, so... let instances do the job.
        AaBoundingBox::new(Point::ZERO, Point::ONE)
    }
}

impl Object for Cube {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self.mesh.hit(r, t_min, t_max)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self.mesh.hit_t(r, t_min, t_max)
    }
}

impl Sampleable for Cube {
    fn area(&self) -> Float {
        self.mesh.area()
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        self.mesh.sample_on(rand_sq)
    }
}
