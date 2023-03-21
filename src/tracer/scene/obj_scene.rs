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

    pub fn obj_scene(focal_length: f64) -> Self {
        let tringl = match Self::load_obj("test.obj") {
            Err(e) => panic!("{}", e),
            Ok(ts) => ts,
        };

        let yg = -focal_length;
        let col = DVec3::splat(0.95);
        let light_z = yg;
        let light_xy = 0.2*focal_length;
        let r = 0.2*focal_length;

        Self::new(
            vec![
                Mesh::new(
                    tringl,
                    Material::diffuse(
                        Texture::Solid(DVec3::new(0.0, 1.0, 0.0))
                    ))
                    .scale(DVec3::splat(0.025))
                    .rotate_z(PI / 2.0)
                    .rotate_x(-PI / 2.0)
                    .translate(DVec3::new(0.0, -0.5, -0.5))
                    .make_box(),
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-2.0*light_xy, -yg - EPSILON, light_z + 2.0*light_xy),
                        DVec3::new(-2.0*light_xy, -yg - EPSILON, light_z - 2.0*light_xy),
                        DVec3::new(2.0*light_xy, -yg - EPSILON, light_z - 2.0*light_xy),
                    ),
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                /* roof */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(yg - light_xy, -yg, 0.0),
                        DVec3::new(yg - light_xy, -yg, 2.0*light_z),
                        DVec3::new(-yg + light_xy, -yg, 2.0*light_z),
                    ),
                    Material::diffuse(Texture::Solid(col)),
                ),
                /* floor */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(yg, yg, 2.0*light_z),
                        DVec3::new(yg, yg, 0.0),
                        DVec3::new(-yg, yg, 0.0),
                    ),
                    Material::metal(
                        Texture::Checkerboard(
                            Box::new(Texture::Solid(col)),
                            Box::new(Texture::Solid(DVec3::new(0.0, 0.0, 0.9))),
                            2.42,
                        ),
                        0.01,
                    ),
                ),
                /* front wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, -yg, 2.0*light_z + 4.0*EPSILON),
                        DVec3::new(yg, -yg, 2.0*light_z + 4.0*EPSILON),
                        DVec3::new(yg, yg, 2.0*light_z + 4.0*EPSILON),
                    ),
                    Material::diffuse(Texture::Solid(col)),
                ),
                /* left wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(yg, yg - light_xy, 2.0*light_z),
                        DVec3::new(yg, -yg + light_xy, 2.0*light_z),
                        DVec3::new(yg, -yg + light_xy, 0.0),
                    ),
                    Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 1.0))),
                ),
                /* right wall */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg - 5.0*EPSILON, -yg + 5.0*EPSILON, 0.0),
                        DVec3::new(-yg - 5.0*EPSILON, -yg + 5.0*EPSILON, 2.0*light_z),
                        DVec3::new(-yg - 5.0*EPSILON, yg - 5.0*EPSILON, 2.0*light_z),
                    ),
                    Material::diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                ),
                // background
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(yg, -yg, 0.0),
                        DVec3::new(-yg, -yg, 0.0),
                        DVec3::new(-yg, yg, 0.0),
                    ),
                    Material::diffuse(Texture::Solid(DVec3::ZERO)),
                ),
            ]
        )
    }
}
