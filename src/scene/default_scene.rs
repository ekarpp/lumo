use super::*;

impl Default for Scene {
    fn default() -> Self {
        Self::new(
            vec![
                Sphere::new(
                    DVec3::new(0.0, 0.5, -1.0),
                    0.25,
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                Sphere::new(
                    DVec3::ZERO,
                    10.0,
                    Material::diffuse(Texture::Solid(DVec3::ZERO)),
                ),
                // floor
                Plane::new(
                    DVec3::new(0.0, -1.0, 0.0),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::Diffuse(Texture::Checkerboard(
                        Box::new(Texture::Checkerboard(
                            Box::new(Texture::Solid(DVec3::ZERO)),
                            Box::new(Texture::Solid(DVec3::ONE)),
                            4.0,
                        )),
                        Box::new(Texture::Marble(
                            Perlin::new(DVec3::splat(192.0) / 255.9)
                        )),
                        1.0,
                    )),
                ),
            ]
        )
    }
}
