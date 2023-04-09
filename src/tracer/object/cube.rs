use super::*;

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
/*
    /// Choose a rectangle uniformly at random
    fn choose_rectangle(&self) -> &Rectangle {
        let idx = {
            let rnd = rand_utils::rand_f64() * 6.0;
            rnd.floor() as usize
        };

        &self.rectangles[idx]
    }
*/
}

impl Bounded for Cube {
    fn bounding_box(&self) -> AaBoundingBox {
        // we only support unit cubes, so... let instances do the job.
        AaBoundingBox::new(DVec3::ZERO, DVec3::ONE)
    }
}

impl Solid for Cube {
    fn inside(&self, xo: DVec3) -> bool {
        // again, only unit cubes
        xo.min_element() >= 0.0 && xo.max_element() <= 1.0
    }
}

impl Object for Cube {
    fn material(&self) -> &Material {
        &self.material
    }

    fn hit(&self, r: &Ray, t_min: f64, mut t_max: f64) -> Option<Hit> {
        let mut h = None;

        for rectangle in &self.rectangles {
            // if we hit an object, it must be closer than what we have
            h = rectangle.hit(r, t_min, t_max).or(h);
            // update distance to closest found so far
            t_max = h.as_ref().map_or(t_max, |hit| hit.t);
        }

        h
    }
}
/*
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray {
        /* add normal to rectangle, now can do visible area of cube??
         * add middle point of rectangle? 0.5a + 0.5c
         * (dot prod with all normals. need direction? dot < 0.0 => visible)
         * weight faces that have lower dot prod.. interesting.. */
        unimplemented!()
    }

    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 {
        unimplemented!()
    }

    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        self.choose_rectangle().sample_on(rand_sq)
    }
}
*/
