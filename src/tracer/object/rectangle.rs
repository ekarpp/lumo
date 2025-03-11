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
        Self::_from_abc(a, b, c, material)
    }

    fn _from_abc(a: Vec3, b: Vec3, c: Vec3, material: Material) -> Box<Self> {
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

    /// Unit size XZ plane with Y up centered at origin
    pub fn unit_xz(material: Material) -> Box<Self> {
        Self::_from_abc(
            0.5 * (Point::X - Point::Z),
            -0.5 * (Point::X + Point::Z),
            0.5 * (Point::Z - Point::X),
            material,
        )
    }

    /// Plane with center at `origin` and size `extent` in directions `bx` and `by`.
    /// Normal is `bx.cross(by)`
    pub fn plane(
        origin: Point,
        bx: Vec3,
        by: Vec3,
        extent: Vec2,
        material: Material
    ) -> Box<Self> {
        let bx = bx.normalize();
        let by = by.normalize();

        let b = origin + bx * extent.x - by * extent.y;
        let a = origin - 2.0 * by * extent.y;
        let c = origin - 2.0 * bx * extent.x;

        Self::_from_abc(a, b, c, material)
    }
}

impl Object for Rectangle {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self.mesh.hit(r, t_min, t_max)
            .map(|mut h| {
                let uv = Vec2::new(
                    self.b0.dot(h.p),
                    self.b1.dot(h.p),
                );
                h.uv = Hit::wrap_uv(uv);

                h
            })
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self.mesh.hit_t(r, t_min, t_max)
    }

    fn bounding_box(&self) -> AaBoundingBox {
        let a = self.b1 + self.origin;
        let b = self.origin;
        let c = self.b0 + self.origin;
        let d = self.origin + self.b0 + self.b1;

        AaBoundingBox::new(
            a.min(b).min(c).min(d),
            a.max(b).max(c).max(d),
        )
    }

    fn num_primitives(&self) -> usize { 2 }
}

impl Sampleable for Rectangle {
    fn area(&self) -> Float {
        self.b0.cross(self.b1).length().abs()
    }

    fn material(&self) -> &Material { self.mesh.material() }

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
