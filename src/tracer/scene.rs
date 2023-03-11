use crate::{DVec3, DMat3, DAffine3, DVec2};
use std::f64::consts::PI;
#[allow(unused_imports)]
use crate::samplers::{UniformSampler, JitteredSampler};
use crate::perlin::Perlin;
use crate::consts::{EPSILON, SHADOW_RAYS};
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
    /* vec of indices to objects that are lights */
    lights: Vec<usize>,
    /* TODO */
    pub objects: Vec<Box<dyn Object>>,
}

/* temporary constant */
const LIGHT_R: f64 = 0.1;

type PixelSampler = JitteredSampler;

impl Scene {
    pub fn new(amb: DVec3, objs: Vec<Box<dyn Object>>) -> Self {
        let lights = (0..objs.len()).map(|i: usize| match objs[i].material() {
            Material::Light(_) => i,
            _ => objs.len(),
        }).filter(|i: &usize| *i != objs.len()).collect();

        Self {
            ambient: amb,
            lights: lights,
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

    pub fn sample_lights_from(&self, h: &Hit) -> Vec<Hit> {
        self.lights.iter().flat_map(|light_idx: &usize| {
            let light = &self.objects[*light_idx];
            PixelSampler::new(SHADOW_RAYS).filter_map(|rand_sq: DVec2| {
                self.hit_light(&light.sample_from(h, rand_sq), &light)
            }).collect::<Vec<Hit>>()
        }).collect()
    }

    fn hit_light<'a>(&'a self, r: &Ray, light: &'a Box<dyn Object>)
                     -> Option<Hit> {
        let light_hit = light.hit(r).and_then(|mut h| {
            h.t -= EPSILON;
            Some(h)
        });

        let no_block_light = |obj: &&Box<dyn Object>| -> bool {
            obj.is_translucent() || obj.hit(r).is_none()
                || obj.hit(r) > light_hit
        };

        let reached_light = self.objects.iter().take_while(no_block_light)
            .count() == self.objects.len();

        if reached_light { light_hit } else { None }
    }

    pub fn box_scene(focal_length: f64) -> Self {
        /* y ground */
        let yg = -focal_length;
        let col = DVec3::new(255.0, 253.0, 208.0) / 255.9;
        let light_z = yg;
        let light_xy = 0.2*focal_length;
        let r = 0.2*focal_length;
        Self::new(
            DVec3::splat(0.0),
            vec![
                /* if light and roof have same y, then light should come
                 * before roof in this vector and everything should be ok */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-light_xy, -yg, light_z + light_xy),
                        DVec3::new(-light_xy, -yg, light_z - light_xy),
                        DVec3::new(light_xy, -yg, light_z - light_xy),
                    ),
                    Material::Light(Texture::Solid(DVec3::ONE)),
                ),
                Sphere::new(
                    DVec3::new(-2.0*light_xy, yg+r, light_z - 2.0*light_xy),
                    r,
                    Material::Mirror,
                ),
                Cuboid::new(
                    DAffine3::from_translation(
                        DVec3::new(light_xy, yg, 1.7*light_z))
                        * DAffine3::from_scale(
                            DVec3::new(light_xy, 2.0*light_xy, light_xy))
                        * DAffine3::from_rotation_y(PI / 10.0),
                    Material::Diffuse(
                        Texture::Solid(DVec3::new(0.0, 0.9, 0.0))
                    )
                ),
                // roof
                Plane::new(
                    DVec3::new(0.0, -yg, 0.0),
                    DVec3::new(0.0, -10000.0, 0.0),
                    Material::Diffuse(Texture::Solid(col)),
                ),
                /* floor */
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, yg, light_z),
                        DVec3::new(yg, yg, light_z),
                        DVec3::new(yg, yg, 2.0*light_z),
                    ),
                    /* can do texture here */
                    Material::Diffuse(Texture::Solid(col)),
                ),
                Rectangle::new(
                    DMat3::from_cols(
                        DVec3::new(-yg, yg, light_z),
                        DVec3::new(yg, yg, light_z),
                        DVec3::new(yg, yg, 0.0),
                    ),
                    Material::Diffuse(Texture::Solid(col)),
                ),
                // front wall
                Plane::new(
                    DVec3::new(0.0, 0.0, 2.0*light_z),
                    DVec3::new(0.0, 0.0, 100.0),
                    Material::Diffuse(Texture::Solid(col)),
                ),
                // left wall
                Plane::new(
                    DVec3::new(yg, 0.0, 0.0),
                    DVec3::new(10000.0, 0.0, 0.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 1.0))),
                ),
                // right wall
                Plane::new(
                    DVec3::new(-yg, 0.0, 0.0),
                    DVec3::new(-1000.0, 0.0, 0.0),
                    Material::Diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                ),
                // background
                Plane::new(
                    DVec3::new(0.0, 0.0, 0.1),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Blank,
                ),
            ],
        )
    }

    pub fn default() -> Self {
        Self::new(
            DVec3::splat(0.0),
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
