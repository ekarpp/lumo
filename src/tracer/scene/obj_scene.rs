use super::*;
use std::fs::File;
use std::io::Result;
use crate::obj;
use crate::tracer::object::triangle::Triangle;
use crate::tracer::object::kdtree::Mesh;

impl Scene {
    fn load_obj(fname: &str) -> Result<Vec<Triangle>> {
        obj::load_obj_file(File::open(fname)?)
    }

    pub fn obj_scene() -> Self {
        let tringl = match Self::load_obj("test.obj") {
            Err(e) => panic!("{}", e),
            Ok(ts) => ts,
        };

        let mesh = Mesh::new(
            tringl,
            Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 0.0)))
        );

        Self::new(
            vec![
                Box::new(mesh),
                Sphere::new(
                    DVec3::new(-2.0, 1.0, -2.0),
                    0.5,
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                Sphere::new(
                    DVec3::ZERO,
                    15.0,
                    Material::diffuse(Texture::Solid(DVec3::splat(0.5)))
                ),
                Plane::new(
                    DVec3::new(0.0, -0.5, 0.0),
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
