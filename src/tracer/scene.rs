use crate::{DVec3, DMat3, DAffine3};
use std::f64::consts::PI;
use crate::rand_utils;
use crate::perlin::Perlin;
use crate::consts::SHADOW_RAYS;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use crate::tracer::material::Material;
use crate::tracer::object::sphere::Sphere;
use crate::tracer::object::{Object, Plane, Rectangle, Cuboid};

#[cfg(test)]
mod scene_tests;

pub struct Scene {
    pub ambient: DVec3,
    light: Sphere,
    objects: Vec<Box<dyn Object>>,
}

/* temporary constant */
const LIGHT_R: f64 = 0.1;

impl Scene {
    pub fn new(l: DVec3, amb: DVec3, objs: Vec<Box<dyn Object>>) -> Self {
        Self {
            ambient: amb,
            light: *Sphere::new(
                l,
                LIGHT_R,
                /* Material::Light has no implementation.
                 * shading does not call material, yet */
                Material::Light(Texture::Solid(DVec3::ONE)),
            ),
            objects: objs,
        }
    }

    /* might want to print x of this, y of that, ... */
    pub fn size(&self) -> usize {
        self.objects.iter().fold(0, |acc, obj| acc + obj.size())
    }

    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        self.objects.iter().map(|obj| obj.hit(r))
            .fold(None, |closest, hit| {
                if closest.is_none() || (hit.is_some() && hit < closest) {
                    hit
                } else {
                    closest
                }
            })
    }

    pub fn rays_to_light(&self, h: &Hit) -> Vec<Ray> {
        (0..SHADOW_RAYS).map(|_| {
            /* we want to do better than uniformly at random from disk */
            self.light.sample_shadow(h, rand_utils::rand_unit_disk())
        }).filter(|r: &Ray| self.hit_light(r)).collect()
    }

    pub fn hit_light(&self, r: &Ray) -> bool {
        let no_block_light = |obj: &&Box<dyn Object>| -> bool {
            obj.hit(r).filter(|hit| {
                !hit.object.is_translucent()
                /* check if object is behind light */
                    && (hit.p - r.origin).length_squared() <
                    (self.light.origin - r.origin).length_squared()
            }).is_none()
        };

        self.objects.iter().take_while(no_block_light).count()
            == self.objects.len()
    }

    pub fn box_scene() -> Self {
        let l = DVec3::new(0.0, 1.0, -1.0);
        /* y ground */
        let yg = -0.8;
        Self::new(
            l,
            DVec3::splat(0.1),
            vec![
                /* floor */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, yg, 0.0),
                        DVec3::new(yg, yg, 0.0),
                        DVec3::new(yg, yg, 2.0*yg),
                    ),
                    Material::Phong(Texture::Checkerboard(
                        Box::new(Texture::Checkerboard(
                            Box::new(Texture::Solid(DVec3::ZERO)),
                            Box::new(Texture::Solid(DVec3::ONE)),
                            4.0,
                        )),
                        Box::new(Texture::Marble(
                            Perlin::new(DVec3::splat(192.0 / 255.9))
                        )),
                        1.0,
                    )),
                ),
                // front wall
                Plane::new(
                    DVec3::new(0.0, 0.0, 2.0*yg),
                    DVec3::new(0.0, 0.0, 1.0),
                    Material::Phong(
                        Texture::Solid(DVec3::splat(155.0 / 255.9))
                    ),
                ),
                // left wall
                Plane::new(
                    DVec3::new(yg, 0.0, 0.0),
                    DVec3::new(1.0, 0.0, 0.0),
                    Material::Phong(Texture::Checkerboard(
                        Box::new(Texture::Solid(DVec3::ZERO)),
                        Box::new(Texture::Solid(DVec3::new(1.0, 0.0, 0.0))),
                        1.0,
                    ))
                ),
                // right wall
                Plane::new(
                    DVec3::new(-yg, 0.0, 0.0),
                    DVec3::new(-1.0, 0.0, 0.0),
                    Material::Phong(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                ),
                // roof
                Plane::new(
                    DVec3::new(0.0, -yg, 0.0),
                    DVec3::new(0.0, -1.0, 0.0),
                    Material::Phong(Texture::Marble(
                        Perlin::new(DVec3::new(0.0, 0.0, 1.0))
                    )),
                ),
                // background
                Plane::new(
                    DVec3::new(0.0, 0.0, 0.1),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Blank,
                ),
                Sphere::new(
                    DVec3::new(-0.4, -0.6, -1.2),
                    0.2,
                    Material::Mirror,
                ),
                Cuboid::new(
                    DAffine3::from_translation(
                        DVec3::new(0.2, yg, 1.7*yg))
                        * DAffine3::from_scale(DVec3::new(0.2, 0.4, 0.2))
                        * DAffine3::from_rotation_y(PI / 10.0),
                    Material::Phong(Texture::Marble(
                        Perlin::new(DVec3::new(0.0, 192.0, 0.0) / 255.9)
                    ))
                ),
            ],
        )
    }

        pub fn default() -> Self {
            let l = DVec3::new(-0.3, 0.2, -0.1);

            Self::new(
                l,
                DVec3::splat(0.15),
                vec![
                    // floor
                    Plane::new(
                        DVec3::new(0.0, -0.5, 0.0),
                        DVec3::new(0.0, 1.0, 0.0),
                        Material::Phong(Texture::Checkerboard(
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
                        Material::Phong(Texture::Solid(DVec3::new(0.0, 0.0, 1.0))),
                    ),
                    Rectangle::new(
                        DMat3::from_cols(
                            DVec3::new(1.2, 0.2, -0.8),
                            DVec3::new(0.8, 0.6, -0.4),
                            DVec3::new(0.4, 0.6, -0.8),
                        ),
                        Material::Mirror,
                    ),
                    Cuboid::new(
                        DAffine3::from_translation(
                            DVec3::new(-0.4, -0.5, -0.6))
                            * DAffine3::from_scale(DVec3::splat(0.15))
                            * DAffine3::from_rotation_y(PI / 4.0),
                        Material::Phong(Texture::Checkerboard(
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
                        Material::Phong(Texture::Solid(DVec3::new(1.0, 0.0, 0.0))),
                    ),
                    // behind
                    Plane::new(
                        DVec3::new(0.0, 0.0, 1.0),
                        DVec3::new(0.0, 0.0, -1.0),
                        Material::Phong(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                    ),
                    Sphere::new(
                        DVec3::new(0.0, 0.0, -1.0),
                        0.5,
                        Material::Phong(Texture::Solid(
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
                        Material::Phong(Texture::Marble(Perlin::new(
                            DVec3::new(255.0, 182.0, 193.0) / 255.9
                        ))),
                    ),
                ]
            )
        }
    }
