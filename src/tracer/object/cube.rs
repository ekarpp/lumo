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
                    DMat3::from_cols(
                        DVec3::new(0.0, 0.0, 1.0),
                        DVec3::new(0.0, 0.0, 0.0),
                        DVec3::new(1.0, 0.0, 0.0),
                    ),/* xz-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(1.0, 1.0, 0.0),
                        DVec3::new(0.0, 1.0, 0.0),
                        DVec3::new(0.0, 1.0, 1.0),
                    ),/* xz-plane +1 */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(0.0, 1.0, 0.0),
                        DVec3::new(0.0, 0.0, 0.0),
                        DVec3::new(0.0, 0.0, 1.0),
                    ), /* yz-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(1.0, 0.0, 1.0),
                        DVec3::new(1.0, 0.0, 0.0),
                        DVec3::new(1.0, 1.0, 0.0),
                    ), /* yz-plane + 1x*/
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(1.0, 0.0, 0.0),
                        DVec3::new(0.0, 0.0, 0.0),
                        DVec3::new(0.0, 1.0, 0.0),
                    ), /* xy-plane */
                    Material::Blank,
                ),
                *Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(0.0, 1.0, 1.0),
                        DVec3::new(0.0, 0.0, 1.0),
                        DVec3::new(1.0, 0.0, 1.0),
                    ), /* xy-plane + 1z */
                    Material::Blank,
                ),
            ],
        }
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

impl Object for Cube {
    /// Shoot ray forwards and backwards. If both hit, we are inside.
    /// Probably faster ways to do this.
    fn inside(&self, p: DVec3) -> bool {
        /* direction shouldn't matter */
        let r = Ray::new(p, DVec3::new(1.0, 0.0, 0.0));

        self.hit(&r, -INFINITY, 0.0).and(self.hit(&r, 0.0, INFINITY))
            .is_some()
    }

    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        self.rectangles.iter()
            .map(|rect| rect.hit(r, t_min, t_max))
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

    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 {
        unimplemented!()
    }

    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        self.choose_rectangle().sample_on(rand_sq)
    }

}
