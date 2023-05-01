use super::*;
use crate::srgb_to_linear;

impl Scene {
    /// Constructs an empty "Cornell box". Middle of the box is at
    /// `(0.0, 0.0, -1.0)` and it has dimensions `2x2x2`.
    /// Perfect for the default camera.
    ///
    /// # Arguments
    /// * `def_color` - Color of the roof, and the front wall
    /// * `mat_left` - Material of the left wall
    /// * `mat_right` - Material of the right wall
    pub fn empty_box(
        def_color: DVec3,
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
        let l_dim = 0.3;

        let mut scene = Self::default();

        /* rectangular area light */
        scene.add_light(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(-l_dim, ceiling - EPSILON, 0.5 * front + l_dim),
                DVec3::new(-l_dim, ceiling - EPSILON, 0.5 * front - l_dim),
                DVec3::new(l_dim, ceiling - EPSILON, 0.5 * front - l_dim),
            ),
            Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))),
        ));

        /* left wall */
        scene.add(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(left, ground, back),
                DVec3::new(left, ground, front),
                DVec3::new(left, ceiling, front),
            ),
            mat_left,
        ));

        /* right wall */
        scene.add(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(right, ground, front),
                DVec3::new(right, ground, back),
                DVec3::new(right, ceiling, back),
            ),
            mat_right,
        ));

        /* floor */
        scene.add(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(left, ground, back),
                DVec3::new(right, ground, back),
                DVec3::new(right, ground, front),
            ),
            Material::diffuse(Texture::Solid(def_color)),
        ));

        /* roof */
        scene.add(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(left, ceiling, front),
                DVec3::new(right, ceiling, front),
                DVec3::new(right, ceiling, back),
            ),
            Material::diffuse(Texture::Solid(def_color)),
        ));

        /* front wall */
        scene.add(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(left, ground, front),
                DVec3::new(right, ground, front),
                DVec3::new(right, ceiling, front),
            ),
            Material::diffuse(Texture::Solid(def_color)),
        ));

        scene
    }
}
