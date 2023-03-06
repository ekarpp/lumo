use std::iter;
use std::f64::consts;
use crate::{DVec3, DQuat, DAffine3, DMat3};
use crate::tracer::object::{Object, Sphere, Plane};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::material::Material;
use crate::tracer::texture::Texture;
use crate::rand_utils;

#[cfg(test)]
mod scene_tests;

pub struct Scene {
    pub light: DVec3,
    pub ambient: DVec3,
    objects: Vec<Box<dyn Object>>,
}

impl Scene {
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let mut closest_hit: Option<Hit> = None;
        for sphere in &self.objects {
            let h = sphere.hit(r);
            // make cleaner?
            if closest_hit.is_none() {
                closest_hit = h;
            }
            else if h.is_some() && h < closest_hit {
                closest_hit = h;
            }
        }
        closest_hit
    }

    pub fn hit_light(&self, r: &Ray) -> bool {
        let block_light = |h: &Hit| -> bool {
            !h.object.material().is_translucent()
                && (h.p - r.origin).length_squared() <
                (self.light - r.origin).length_squared()
        };

        for object in &self.objects {
            let h = object.hit(r);
            // h.is_some_and
            if h.filter(block_light).is_some() {
                return false;
            }
        }
        true
    }

    pub fn random() -> Scene {
        /* this controls the minumum radius, at least that is the idea.. */
        let ground_y = -0.1;
        let scale_y = 0.01;
        let ground: iter::Once<Box<dyn Object>> = iter::once(Plane::new(
            DVec3::new(0.0, ground_y, 0.0),
            DVec3::new(0.0, 1.0, 0.0),
            Material::Phong(Texture::Solid(DVec3::ONE)),
        ));

        /* affine transformation for the origin of random spheres
         * shear+rotate+scale xz-plane to exact view of camera.
         * scale y so we don't get too big spheres.
         * assume 16/9 aspect ratio and 90 vfov */
        let shear_xz = (consts::PI / 4.0 - (16.0 / 9.0 as f64).atan()).tan();
        let scale_xz = 1.2;
        let sphere_aff =
            /* rotate to camera direction */
            DAffine3::from_rotation_y(3.0 * consts::PI / 4.0)
            /* shear to get same hvof as camera */
            * DAffine3::from_mat3(
                DMat3::from_cols( // from_rows
                    DVec3::new(1.0, 0.0, shear_xz),
                    DVec3::new(shear_xz, 1.0, shear_xz),
                    DVec3::new(shear_xz, 0.0, 1.0),
                ).transpose())
            * DAffine3::from_scale(
                DVec3::new(scale_xz, scale_y, scale_xz));

        /* divide the unit xz-plane to dim x dim divisions.
         * place a sphere at random location in each division */
        let dim = 15;
        let txd = 1.0 / dim as f64;

        let objects: Vec<Box<dyn Object>> = (1..dim).flat_map(|z| {
            (1..dim).map(|x| -> Box<dyn Object> {
                let m = match rand_utils::rand_f64() {
                    f if f < 0.1 => Material::Glass,
                    f if f < 0.9 => Material::Phong(
                        Texture::Solid(rand_utils::rand_dvec3())
                    ),
                    _ => Material::Mirror,
                };
                /* scale xz-plane first to division size, then
                 * translate to a correct position */
                let o = sphere_aff.transform_point3(
                    DVec3::ZERO * DVec3::new(txd, 1.0, txd)
                        + DVec3::new(txd*x as f64, 0.0, txd*z as f64)
                );
                Sphere::new(
                    o,
                    (o.y - ground_y).abs(),
                    m,
                )
            }).collect::<Vec<Box<dyn Object>>>()
        }).chain(ground).collect();

        let s = DVec3::new(1.0, 0.2, 0.5);
        /* affine transformation for possible positions of point light */
        let _light_aff = DAffine3::from_scale_rotation_translation(
            s,
            DQuat::from_rotation_z(consts::PI),
            DVec3::new(-s.x / 2.0, 0.5, -2.0)
        );

        Scene {
            light: DVec3::new(0.0, 3.0, 0.0),
            ambient: DVec3::splat(rand_utils::rand_f64()) * 0.7,
            objects: objects,
        }
    }

    pub fn default() -> Scene {
        let l = DVec3::new(-0.3, 0.2, -0.1);
        Scene {
            light: l,
            ambient: DVec3::splat(0.15),
            objects: vec![
                // floor
                Plane::new(
                    DVec3::new(0.0, -0.5, 0.0),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::Phong(Texture::Checkerboard),
                ),
                // right
                Plane::new(
                    DVec3::new(3.0, 0.0, -3.0),
                    DVec3::new(-1.0, 0.0, 1.0),
                    Material::Phong(Texture::Solid(DVec3::new(0.0, 0.0, 1.0))),
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
                    DVec3::new(0.4, 0.0, -0.5),
                    0.1,
                    Material::Glass,
                ),
            ]
        }
    }
}
