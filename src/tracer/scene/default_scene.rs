use super::*;

/* temporary constant */
const LIGHT_R: f64 = 0.1;

impl Scene {
    pub fn default_scene() -> Self {
        Self::new(
            vec![
                Sphere::new(
                    DVec3::new(-0.3, 0.2, -0.1),
                    LIGHT_R,
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                // floor
                Plane::new(
                    DVec3::new(0.0, -0.5, 0.0),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::Diffuse(Texture::Checkerboard(
                        Box::new(Texture::Checkerboard(
                            Box::new(Texture::Solid(DVec3::ZERO)),
                            Box::new(Texture::Solid(DVec3::ONE)),
                            4.0,
                        )),
                        /* share same perlin between all textures?
                         * could make cool checkers that way */
                        Box::new(Texture::Marble(
                            Perlin::new(DVec3::splat(192.0) / 255.9)
                        )),
                        1.0,
                    )),
                ),
                // right
                Plane::new(
                    DVec3::new(3.0, 0.0, -3.0),
                    DVec3::new(-1.0, 0.0, 1.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(0.0, 0.0, 1.0))),
                ),
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(1.2, 0.2, -0.8),
                        DVec3::new(0.8, 0.6, -0.4),
                        DVec3::new(0.4, 0.6, -0.8),
                    ),
                    DVec3::new(-1.0, 0.0, 0.0),
                    Material::Mirror,
                ),
                Cuboid::new(
                    DAffine3::from_translation(
                        DVec3::new(-0.4, -0.5, -0.6))
                        * DAffine3::from_scale(DVec3::splat(0.15))
                        * DAffine3::from_rotation_y(PI / 4.0),
                    Material::Diffuse(Texture::Checkerboard(
                        Box::new(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                        Box::new(Texture::Solid(
                            DVec3::new(50.0, 205.0, 50.0) / 255.9
                        )),
                        9.0,
                    )),
                ),
                // left
                Plane::new(
                    DVec3::new(-3.0, 0.0, -3.0),
                    DVec3::new(1.0, 0.0, 1.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 0.0))),
                ),
                // behind
                Plane::new(
                    DVec3::new(0.0, 0.0, 1.0),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                ),
                Sphere::new(
                    DVec3::new(0.0, 0.0, -1.0),
                    0.5,
                    Material::Diffuse(Texture::Solid(
                        DVec3::new(136.0, 8.0, 8.0) / 255.9
                    )),
                ),
                Sphere::new(
                    DVec3::new(-0.9, 0.0, -1.0),
                    0.1,
                    Material::Mirror,
                ),
                Sphere::new(
                    DVec3::new(-0.4, -0.12, -0.5),
                    0.1,
                    Material::Glass,
                ),
                Sphere::new(
                    DVec3::new(0.4, -0.2, -0.5),
                    0.1,
                    Material::Diffuse(Texture::Marble(Perlin::new(
                        DVec3::new(255.0, 182.0, 193.0) / 255.9
                    ))),
                ),
            ]
        )
    }
}
