use super::*;

/// Rectangle defined by two triangles
pub struct Rectangle {
    /// Just a mesh...
    mesh: Mesh,
    /// point b
    origin: Vec3,
    /// b to c
    b0: Vec3,
    /// b to a
    b1: Vec3,
}

impl Rectangle {
    /// Constructs a rectangle from three points. Second point, namely `b`,
    /// is mirrored around the line `ac`
    ///
    /// # Arguments
    /// * `abc` - Points `a,b,c` stored in the rows. Normal in CCW
    /// * `material` - Material of the rectangle
    pub fn new(abc: Mat3, material: Material) -> Box<Self> {
        let (a, b, c) = (abc.y0, abc.y1, abc.y2);
        let origin = b;
        let b0 = c - origin;
        let b1 = a - origin;
        // mirror `b` around `ac`
        let d = origin + b0 + b1;
        let vertices = vec![a, b, c, d];

        let faces = vec![Face::new(vec![0,1,2,3], vec![], vec![])];

        Box::new(Self {
            origin, b0, b1,
            mesh: TriangleMesh::new(vertices, faces, vec![], vec![], material),
        })
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
        self.b0.cross(self.b1).length().abs()
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let xo = self.origin + rand_sq.x * self.b0 + rand_sq.y * self.b1;
        let ng = self.b0.cross(self.b1).normalize();

        let xo_abs = self.origin.abs()
            + (rand_sq.x * self.b0).abs()
            + (rand_sq.y * self.b1).abs();
        let err = efloat::gamma(4) * xo_abs;

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
mod rectangle_tests {
    use super::*;

    test_util::test_sampleable!(Rectangle::new(
        Mat3::new(
            -Point::Y + Point::X,
            Point::Y + Point::X,
            Point::Y + -Point::X,
        ),
        Material::Blank,
    ));
}
