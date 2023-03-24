use super::*;

impl Default for Scene {
    fn default() -> Self {
        Self::new(
            vec![
                Sphere::new(
                    DVec3::new(0.0, 0.5, -1.0),
                    0.25,
                    Material::Light(Texture::Solid(srgb_to_lin(255, 255, 255))),
                ),
                Sphere::new(
                    DVec3::ZERO,
                    10.0,
                    Material::diffuse(Texture::Solid(srgb_to_lin(0, 0, 0))),
                ),
                // floor
                Plane::new(
                    DVec3::new(0.0, -1.0, 0.0),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::diffuse(Texture::Checkerboard(
                        Box::new(Texture::Checkerboard(
                            Box::new(Texture::Solid(srgb_to_lin(0, 0, 0))),
                            Box::new(Texture::Solid(srgb_to_lin(255, 255, 255))),
                            4.0,
                        )),
                        Box::new(Texture::Marble(
                            Perlin::new(srgb_to_lin(192, 192, 192))
                        )),
                        1.0,
                    )),
                ),
            ]
        )
    }
}
