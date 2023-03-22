use super::*;

impl Scene {
    pub fn empty_box(
        def_color: DVec3,
        mat_left: Material,
        mat_right: Material,
        mat_floor: Material,
    ) -> Self {

        // aka Y of ground, negate for roof
        let ground = -1.0;
        // aka X of right wall, negate for left wall
        let right = 1.0;
        // aka Z of front wall, negate for background
        let front = -2.0;
        // 0.5x of sidelength of area light
        let light_dim = 0.4;

        Self::new(
            vec![
                /* rectangular area light */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(
                            -light_dim,
                            -ground - EPSILON,
                            0.5 * front + light_dim,
                        ),
                        DVec3::new(
                            -light_dim,
                            -ground - EPSILON,
                            0.5 * front - light_dim,
                        ),
                        DVec3::new(
                            light_dim,
                            -ground - EPSILON,
                            0.5 * front - light_dim,
                        ),
                    ),
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                /* floor */
                Plane::new(
                    DVec3::new(0.0, ground, 0.0),
                    DVec3::new(0.0, 1.0, 0.0),
                    mat_floor,
                ),
                /* left wall */
                Plane::new(
                    DVec3::new(-right, 0.0, 0.0),
                    DVec3::new(1.0, 0.0, 0.0),
                    mat_left,
                ),
                /* right wall */
                Plane::new(
                    DVec3::new(right, 0.0, 0.0),
                    DVec3::new(-1.0, 0.0, 0.0),
                    mat_right,
                ),
                /* roof */
                Plane::new(
                    DVec3::new(0.0, -ground, 0.0),
                    DVec3::new(0.0, -1.0, 0.0),
                    Material::diffuse(Texture::Solid(def_color)),
                ),
                /* front wall */
                Plane::new(
                    DVec3::new(0.0, 0.0, front),
                    DVec3::new(0.0, 0.0, 1.0),
                    Material::diffuse(Texture::Solid(def_color)),
                ),
                /* background */
                Plane::new(
                    DVec3::new(0.0, 0.0, -front),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::diffuse(Texture::Solid(DVec3::ZERO)),
                ),
            ],
        )
    }
}
