use super::*;

impl Scene {
    pub fn box_scene(focal_length: f64) -> Self {
        /* y ground */
        let yg = -focal_length;
        let col = DVec3::new(255.0, 253.0, 208.0) / 255.9;
        let light_z = yg;
        let light_xy = 0.2*focal_length;
        let r = 0.2*focal_length;

        Self::new(
            vec![
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-2.0*light_xy, -yg - EPSILON, light_z + 2.0*light_xy),
                        DVec3::new(-2.0*light_xy, -yg - EPSILON, light_z - 2.0*light_xy),
                        DVec3::new(2.0*light_xy, -yg - EPSILON, light_z - 2.0*light_xy),
                    ),
                    DVec3::new(0.0, -1.0, 0.0),
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                Sphere::new(
                    DVec3::new(-2.0*light_xy, yg+r, light_z - 2.0*light_xy),
                    r,
                    Material::Mirror,
                ),
                Sphere::new(
                    DVec3::new(0.0, yg + r, light_z - light_xy),
                    0.5*r,
                    Material::Glass,
                ),
                Cuboid::new(
                    DAffine3::from_translation(
                        DVec3::new(light_xy, yg, 1.7*light_z))
                        * DAffine3::from_scale(
                            DVec3::new(light_xy, 2.0*light_xy, light_xy))
                        * DAffine3::from_rotation_y(PI / 10.0),
                    Material::Microfacet(
                        Texture::Solid(DVec3::new(0.0, 0.9, 0.0)),
                        MfDistribution::Ggx(0.5),
                    )
                ),
                /* roof */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg + light_xy, -yg, 2.0*light_z),
                        DVec3::new(yg - light_xy, -yg, 2.0*light_z),
                        DVec3::new(yg - light_xy, -yg, 0.0),
                    ),
                    DVec3::new(0.0, -1.0, 0.0),
                    Material::Diffuse(Texture::Solid(col)),
                ),

                /* floor */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, yg, light_z),
                        DVec3::new(yg, yg, light_z),
                        DVec3::new(yg, yg, 2.0*light_z),
                    ),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::Diffuse(Texture::Solid(col)),
                ),
                /* floor extension, can apply texture to other part */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, yg, light_z - EPSILON),
                        DVec3::new(yg, yg, light_z - EPSILON),
                        DVec3::new(yg, yg, 0.0),
                    ),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::Diffuse(Texture::Solid(col)),
                ),
                /* front wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, -yg, 2.0*light_z + 4.0*EPSILON),
                        DVec3::new(yg, -yg, 2.0*light_z + 4.0*EPSILON),
                        DVec3::new(yg, yg, 2.0*light_z + 4.0*EPSILON),
                    ),
                    DVec3::new(0.0, 0.0, 1.0),
                    Material::Diffuse(Texture::Solid(col)),
                ),
                /* left wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(yg, -yg + light_xy, 0.0),
                        DVec3::new(yg, -yg + light_xy, 2.0*light_z),
                        DVec3::new(yg, yg - light_xy, 2.0*light_z),
                    ),
                    DVec3::new(1.0, 0.0, 0.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 1.0))),
                ),
                /* right wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg - 5.0*EPSILON, -yg + 5.0*EPSILON, 0.0),
                        DVec3::new(-yg - 5.0*EPSILON, -yg + 5.0*EPSILON, 2.0*light_z),
                        DVec3::new(-yg - 5.0*EPSILON, yg - 5.0*EPSILON, 2.0*light_z),
                    ),
                    DVec3::new(-1.0, 0.0, 0.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                ),
                // background
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, yg, 0.0),
                        DVec3::new(-yg, -yg, 0.0),
                        DVec3::new(yg, -yg, 0.0),
                    ),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Diffuse(Texture::Solid(DVec3::ZERO)),
                ),
            ],
        )
    }
}
