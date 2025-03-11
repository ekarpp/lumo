use super::*;

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

impl Object for Cube {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self.mesh.hit(r, t_min, t_max)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self.mesh.hit_t(r, t_min, t_max)
    }

    fn bounding_box(&self) -> AaBoundingBox {
        // we only support unit cubes, so... let instances do the job.
        AaBoundingBox::new(Point::ZERO, Point::ONE)
    }

    fn num_primitives(&self) -> usize { 12 }
}

impl Sampleable for Cube {
    fn area(&self) -> Float {
        // unit cube
        6.0
    }

    fn material(&self) -> &Material { self.mesh.material() }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let rand_sphere = 0.5 * rng::maps::square_to_sphere(rand_sq);
        // reproject to reduce floating point accuracies and move inside unit cube
        let rand_sphere = 0.5 * rand_sphere / rand_sphere.length() + 0.5;

        let mut mi = crate::INF;
        let mut ax = crate::Axis::X; let mut d = 0.0;

        for axis in [crate::Axis::X, crate::Axis::Y, crate::Axis::Z] {
            let p = match axis {
                crate::Axis::X => rand_sphere.x,
                crate::Axis::Y => rand_sphere.y,
                crate::Axis::Z => rand_sphere.z,
            };

            if p.abs() < mi { mi = p.abs(); ax = axis; d = -1.0; }
            if (1.0 - p).abs() < mi { mi = (1.0 - p).abs(); ax = axis; d = 1.0; }
        }
        let ng = match ax {
            crate::Axis::X => Vec3::new(d,   0.0, 0.0),
            crate::Axis::Y => Vec3::new(0.0, d,   0.0),
            crate::Axis::Z => Vec3::new(0.0, 0.0, d),
        };

        let xo = rand_sphere + ng * mi;
        let xo_abs = rand_sphere.abs() + (ng * mi).abs();
        // not accurate
        let err = efloat::gamma(9) * xo_abs;

        Hit::new(
            0.0,
            self.mesh.material(),
            -ng,
            xo,
            err,
            ng,
            ng,
            Vec2::ZERO,
        ).unwrap()
    }
}

#[cfg(test)]
mod cube_tests {
    use super::*;
    test_util::test_sampleable!(Cube::new(Material::Blank));
}
