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
}

impl Object for Cuboid {
    fn inside(&self, _r: &Ray) -> bool { todo!() }

    fn size(&self) -> usize { 12 }

    fn area(&self) -> f64 {
        self.rectangles.iter().map(|r| r.area()).sum()
    }

    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        self.rectangles.iter().map(|rect| rect.hit(r))
            .fold(None, |closest, hit| {
                if closest.is_none() || (hit.is_some() && hit < closest) {
                    hit
                } else {
                    closest
                }
            })
            .and_then(|mut hit| {
                /* change us as the object to get correct texture for rendering */
                hit.object = self;
                Some(hit)
            })
    }

}
