use super::*;

pub struct Cuboid {
    rectangles: [Rectangle; 6],
    material: Material,
}

impl Cuboid {

    /* applies the aff to the unit cube. some affines might break this */
    /// Constructs a cuboid by applying an affine transformation
    /// to the unit cube. NOTE! Some affines may break this.
    ///
    /// # Arguments
    /// * `aff` - Affine transformation to be applied to the unit cube
    /// * `m` - Material of the cuboid
    pub fn new(aff: DAffine3, m: Material) -> Box<Self> {
        /* triangles are parallel to xz-plane */
        Self::from_triangles(
            DMat3::from_cols(
                aff.transform_point3(DVec3::new(1.0, 0.0, 0.0)),
                aff.transform_point3(DVec3::new(0.0, 0.0, 0.0)),
                aff.transform_point3(DVec3::new(0.0, 0.0, 1.0)),
            ),
            DMat3::from_cols(
                aff.transform_point3(DVec3::new(1.0, 1.0, 0.0)),
                aff.transform_point3(DVec3::new(0.0, 1.0, 0.0)),
                aff.transform_point3(DVec3::new(0.0, 1.0, 1.0)),
            ),
            aff.transform_point3(DVec3::new(0.0, -1.0, 0.0)),
            m,
        )
    }

    /* be lazy and construct from two triangles */
    /* this is overall really hacky. might just want to create one for
     * unit cube and apply affines to it. */
    /* columns of r1 and r2 define the triangles. the order of columns
     * matters.*/
    /// Helper function to construct cuboids from affine transformations.
    /// `n1` is the direction of the normal defined by `r1`.
    fn from_triangles(r1: DMat3, r2: DMat3, n1: DVec3, m: Material)
                      -> Box<Self> {
        let d1 = _triangle_to_rect(r1);

        let norm_xz = n1.normalize();
        let norm_yz = DQuat::from_rotation_z(-PI / 2.0).mul_vec3(norm_xz);
        let norm_xy = DQuat::from_rotation_x(PI / 2.0).mul_vec3(norm_xz);
        Box::new(Self {
            material: m,
            rectangles: [
                /* directions given assuming unit cube */
                *Rectangle::new(
                    r1, /* xz-plane */
                    norm_xz,
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r2.col(1),
                        r1.col(1),
                        r1.col(2),
                    ), /* yz-plane */
                    norm_yz,
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r1.col(0),
                        r1.col(1),
                        r2.col(1),
                    ), /* xy-plane */
                    norm_xy,
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r2.col(0),
                        r1.col(0),
                        d1,
                    ), /* yz-plane + 1z*/
                    -norm_yz,
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        r2.col(2),
                        r1.col(2),
                        d1,
                    ), /* xy-plane + 1x */
                    -norm_xy,
                    Material::Blank,
                ),
                *Rectangle::new(
                    r2, /* xz-plane + 1y*/
                    -norm_xz,
                    Material::Blank,
                ),
            ],
        })
    }

    /// Choose a rectangle uniformly at random
    fn choose_rectangle(&self) -> &Rectangle {
        let idx = {
            let rnd = rand_utils::rand_f64() * 6.0;
            rnd.floor() as usize
        };

        &self.rectangles[idx]
    }
}

impl Object for Cuboid {
    fn area(&self) -> f64 {
        self.rectangles.iter().map(|r| r.area()).sum()
    }

    /// If vector towards each face from `p` points to the same direction
    /// as the normal of that face, we must be inside.
    fn inside(&self, _p: DVec3) -> bool {
        todo!()
        /*
         * add t_min and t_max to hit. (both, so we arent inside cubes behind us
         * now can do cube inside and medium better. */
    }

    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        self.rectangles.iter().map(|rect| rect.hit(r, t_min, t_max))
            .fold(None, |closest, hit| {
                if closest.is_none() || (hit.is_some() && hit < closest) {
                    hit
                } else {
                    closest
                }
            })
            .map(|mut h| {
                h.object = self;
                h
            })
    }

    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray {
        /* add normal to rectangle, now can do visible area of cube??
         * add middle point of rectangle? 0.5a + 0.5c
         * (dot prod with all normals. need direction? dot < 0.0 => visible)
         * weight faces that have lower dot prod.. interesting.. */
        unimplemented!()
    }

    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        self.choose_rectangle().sample_on(rand_sq)
    }
}
