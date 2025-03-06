use super::*;
use crate::{ Mat3, Point };
use crate::tracer::Spectrum;

impl Scene {
    const LIGHT_EPS: crate::Float = 0.001;
    /// Constructs an empty "Cornell box". Middle of the box is at
    /// `(0.0, 0.0, -1.0)` and it has dimensions `2x1.6x2`.
    /// Perfect for the default camera.
    ///
    /// # Arguments
    /// * `def_color` - Color of the roof, and the front wall
    /// * `mat_left` - Material of the left wall
    /// * `mat_right` - Material of the right wall
    pub fn empty_box(
        def_color: Spectrum,
        mat_left: Material,
        mat_right: Material,
    ) -> Self {
        // aka Y of ground
        let ground = -0.8;
        let ceiling = -ground;
        // aka X of right wall, negate for left wall
        let right = 1.0;
        let left = -right;
        // aka Z of front wall
        let front = -2.0;
        let back = 0.0;

        // 0.5x of sidelength of area light
        let l_dim = 0.1;

        let mut scene = Self::default();

        let light: Spectrum = 32.0 * Spectrum::from_srgb(252, 201, 138);
        /* rectangular area light */
        scene.add_light(Rectangle::new(
            Mat3::new(
                Point::new(-l_dim, ceiling - Self::LIGHT_EPS, 0.6 * front + l_dim),
                Point::new(-l_dim, ceiling - Self::LIGHT_EPS, 0.6 * front - l_dim),
                Point::new(l_dim, ceiling - Self::LIGHT_EPS, 0.6 * front - l_dim),
            ),
            Material::Light(Texture::from(light))
        ));

        /* left wall */
        scene.add(Rectangle::new(
            Mat3::new(
                Point::new(left, ground, back),
                Point::new(left, ground, front),
                Point::new(left, ceiling, front),
            ),
            mat_left,
        ));

        /* right wall */
        scene.add(Rectangle::new(
            Mat3::new(
                Point::new(right, ground, front),
                Point::new(right, ground, back),
                Point::new(right, ceiling, back),
            ),
            mat_right,
        ));

        /* floor */
        scene.add(Rectangle::new(
            Mat3::new(
                Point::new(left, ground, back),
                Point::new(right, ground, back),
                Point::new(right, ground, front),
            ),
            Material::diffuse(Texture::from(def_color.clone())),
        ));

        /* roof */
        scene.add(Rectangle::new(
            Mat3::new(
                Point::new(left, ceiling, front),
                Point::new(right, ceiling, front),
                Point::new(right, ceiling, back),
            ),
            Material::diffuse(Texture::from(def_color.clone())),
        ));

        /* front wall */
        scene.add(Rectangle::new(
            Mat3::new(
                Point::new(left, ground, front),
                Point::new(right, ground, front),
                Point::new(right, ceiling, front),
            ),
            Material::diffuse(Texture::from(def_color)),
        ));

        scene
    }
}
