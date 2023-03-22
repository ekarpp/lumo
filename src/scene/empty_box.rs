use super::*;

impl Scene {
    pub fn empty_box(
        mat_left: Material,
        mat_right: Material,
        mat_floor: Material
    ) -> Self {

        // aka Y of ground, negate for roof
        let ground = -1.0;
        // aka X of right wall, negate for left wall
        let right = 1.0;
        // aka Z of front wall, negate for background
        let front = -2.0;
        // 0.5x of sidelength of area light
        let light_dim = 0.4;

        let def_color = DVec3::splat(0.95);

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
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(right, ground, -front),
                        DVec3::new(right, ground, front),
                        DVec3::new(-right, ground, front),
                    ),
                    mat_floor,
                ),
                /* left wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-right, ground, -front),
                        DVec3::new(-right, ground, front),
                        DVec3::new(-right, -ground, front),
                    ),
                    mat_left,
                ),
                /* right wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(right, ground, front),
                        DVec3::new(right, ground, -front),
                        DVec3::new(right, -ground, -front),
                    ),
                    mat_right,
                ),

                /* roof */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(right, -ground, front),
                        DVec3::new(right, -ground, -front),
                        DVec3::new(-right, -ground, -front),
                    ),
                    Material::diffuse(Texture::Solid(def_color)),
                ),
                /* front wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(right, ground, front),
                        DVec3::new(right, -ground, front),
                        DVec3::new(-right, -ground, front),
                    ),
                    Material::diffuse(Texture::Solid(def_color)),
                ),
                /* background */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(right, -ground, -front),
                        DVec3::new(right, ground, -front),
                        DVec3::new(-right, ground, -front),
                    ),
                    Material::diffuse(Texture::Solid(DVec3::ZERO)),
                ),
            ],
        )
    }
}
