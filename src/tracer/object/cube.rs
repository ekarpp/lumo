use super::*;

#[cfg(test)]
mod cube_tests;

/// A cube consisting of 6 rectangles
pub struct Cube {
    /// The rectangle faces of the cube
    rectangles: [Rectangle; 6],
    /// Material of the cube. Make the rectangles have their own material?
    material: Material,
}

impl Cube {
    /// Constructs an unit cube. To get the desired shape, one should instance
    /// this.
    ///
    /// # Arguments
    /// * `material` - Material of the cube
    pub fn new(material: Material) -> Self {
        /* triangles are parallel to xz-plane */
        Self {
            material,
            rectangles: [
                *Rectangle::new(
                    DMat3::from_cols(DVec3::Z, DVec3::ZERO, DVec3::X), /* xz-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(DVec3::X + DVec3::Y, DVec3::Y, DVec3::Y + DVec3::Z), /* xz-plane +1 */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(DVec3::Y, DVec3::ZERO, DVec3::Z), /* yz-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(DVec3::X + DVec3::Z, DVec3::X, DVec3::X + DVec3::Y), /* yz-plane + 1x*/
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(DVec3::X, DVec3::ZERO, DVec3::Y), /* xy-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(DVec3::Y + DVec3::Z, DVec3::Z, DVec3::X + DVec3::Z), /* xy-plane + 1z */
                    Material::Blank,
                ),
            ],
        }
    }
}

impl Bounded for Cube {
    fn bounding_box(&self) -> AaBoundingBox {
        // we only support unit cubes, so... let instances do the job.
        AaBoundingBox::new(DVec3::ZERO, DVec3::ONE)
    }
}

impl Object for Cube {
    fn hit(&self, r: &Ray, t_min: f64, mut t_max: f64) -> Option<Hit> {
        let mut h = None;

        for rectangle in &self.rectangles {
            // if we hit an object, it must be closer than what we have
            h = rectangle.hit(r, t_min, t_max).or(h);
            // update distance to closest found so far
            t_max = h.as_ref().map_or(t_max, |hit| hit.t);
        }

        h.map(|mut h| { h.material = &self.material; h })
    }
}
