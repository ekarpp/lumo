use super::*;

#[cfg(test)]
mod cube_tests;

/// A unit cube consisting of 6 squares
pub struct Cube<'a> {
    /// Just a mesh...
    mesh: TriangleMesh<'a>
}

impl Cube<'_> {
    /// Constructs an unit cube. To get the desired shape, one should instance
    /// this.
    ///
    /// # Arguments
    /// * `material` - Material of the cube
    pub fn new(material: Material) -> Box<Self> {
        let vertices = vec![
            DVec3::X, DVec3::ZERO, DVec3::Y, DVec3::X + DVec3::Y,
            DVec3::Z, DVec3::Z + DVec3::X, DVec3::ONE, DVec3::Z + DVec3::Y,
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
            Face::new(vec![6, 0, 1, 4], vec![], vec![]),
        ];

        Box::new(Self {
            mesh: *TriangleMesh::new(vertices, faces, vec![], vec![], material)
        })
    }
}

impl Bounded for Cube<'_> {
    fn bounding_box(&self) -> AaBoundingBox {
        // we only support unit cubes, so... let instances do the job.
        AaBoundingBox::new(DVec3::ZERO, DVec3::ONE)
    }
}

impl Object for Cube<'_> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        self.mesh.hit(r, t_min, t_max)
    }
}

impl Sampleable for Cube<'_> {
    fn area(&self) -> f64 {
        self.mesh.area()
    }

    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        self.mesh.sample_on(rand_sq)
    }
}
