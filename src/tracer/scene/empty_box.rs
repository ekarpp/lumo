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
        // aka Y of ground, negate for roof
        let ground = -0.75;
        // aka X of right wall, negate for left wall
        let right = 1.0;
        // aka Z of front wall, negate for background
        let front = -2.0;
        // 0.5x of sidelength of area light
        let light_dim = 0.3;

        let mut scene = Self::default();

        /* rectangular area light */
        scene.add_light(Rectangle::new(
            DMat3::from_cols(
                DVec3::new(-light_dim, -ground - EPSILON, 0.65 * front + light_dim),
                DVec3::new(-light_dim, -ground - EPSILON, 0.65 * front - light_dim),
                DVec3::new(light_dim, -ground - EPSILON, 0.65 * front - light_dim),
            ),
            Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))),
        ));

        /* left wall */
        scene.add(
            Plane::new(DVec3::NEG_X * right, DVec3::X, mat_left),
        );

        /* right wall */
        scene.add(
            Plane::new(DVec3::X * right, DVec3::NEG_X, mat_right),
        );

        /* floor */
        scene.add(Plane::new(
            DVec3::Y * ground,
            DVec3::Y,
            Material::diffuse(Texture::Solid(def_color)),
        ));

        /* roof */
        scene.add(Plane::new(
            DVec3::NEG_Y * ground,
            DVec3::NEG_Y,
            Material::diffuse(Texture::Solid(def_color)),
        ));

        /* front wall */
        scene.add(Plane::new(
            DVec3::Z * front,
            DVec3::Z,
            Material::diffuse(Texture::Solid(def_color)),
        ));

        /* background */
        scene.add(Plane::new(
            DVec3::NEG_Z * front,
            DVec3::NEG_Z,
            // make blank?
            Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0))),
        ));

        scene
    }
}
