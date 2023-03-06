use std::iter;
use std::f64::consts;
use crate::{DVec3, DQuat, DAffine3, DMat3};
use crate::tracer::object::{Object, Sphere, Plane};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::material::Material;
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
        let ground_y = -0.5;
        let ground: iter::Once<Box<dyn Object>> = iter::once(Plane::new(
            DVec3::new(0.0, ground_y, 0.0),
            DVec3::new(0.0, 1.0, 0.0),
            Material::Phong(
                DVec3::ONE,
            ),
        ));

        /* affine transformation for the origin of random spheres
         * shear+rotate+scale xz-plane to exact view of camera
         * scale y so we don't get too big spheres
         * assume 16/9 aspect ratio and 90 vfov */
        let shear_xz = (consts::PI / 4.0 - (16.0 / 9.0 as f64).atan()).tan();
        let scale_xz = 10.0;
        let scale_y = 0.5; // aka max radius
        let sphere_aff = DAffine3::from_rotation_y(consts::PI)
            /* multiply rotation by shear&scale xz + scale y */
            * DAffine3::from_mat3_translation(
                DMat3::from_cols( // from_rows
                    DVec3::new(scale_xz, shear_xz, 0.0),
                    DVec3::new(0.0, scale_y, 0.0),
                    DVec3::new(0.0, shear_xz, scale_xz),
                ).transpose(),
                DVec3::new(0.0, -scale_y / 2.0, 0.0)
            );

        let n = 10;
        let objects: Vec<Box<dyn Object>> = (0..n)
            .map(|_| -> Box<dyn Object> {
                let m = match rand_utils::rand_f64() {
                    x if x < 0.1 => Material::Glass,
                    x if x < 0.9 => Material::Phong(rand_utils::rand_dvec3()),
                    _ => Material::Mirror,
                };
                let o = sphere_aff.transform_point3(rand_utils::rand_dvec3());
                Sphere::new(
                    o,
                    (o.y - ground_y).abs(),
                    m,
                )
            }).chain(ground).collect();

        let s = DVec3::new(1.0, 0.2, 0.5);
        /* affine transformation for possible positions of point light */
        let light_aff = DAffine3::from_scale_rotation_translation(
            s,
            DQuat::from_rotation_z(consts::PI),
            DVec3::new(-s.x / 2.0, 0.5, -2.0)
        );

        Scene {
            light: light_aff.transform_point3(rand_utils::rand_dvec3()),
            ambient: DVec3::splat(rand_utils::rand_f64()) * 0.5,
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
                    Material::Phong(
                        DVec3::ONE
                    )
                ),
                // right
                Plane::new(
                    DVec3::new(3.0, 0.0, -3.0),
                    DVec3::new(-1.0, 0.0, 1.0),
                    Material::Phong(
                        DVec3::new(0.0, 0.0, 1.0)
                    )
                ),
                // left
                Plane::new(
                    DVec3::new(-3.0, 0.0, -3.0),
                    DVec3::new(1.0, 0.0, 1.0),
                    Material::Phong(
                        DVec3::new(1.0, 0.0, 0.0)
                    )
                ),
                // behind
                Plane::new(
                    DVec3::new(0.0, 0.0, 1.0),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Phong(
                        DVec3::new(1.0, 0.0, 1.0)
                    )
                ),
                Sphere::new(
                    DVec3::new(0.0, 0.0, -1.0),
                    0.5,
                    Material::Phong(
                        DVec3::new(136.0, 8.0, 8.0) / 255.9
                    )
                ),
                Sphere::new(
                    DVec3::new(-0.9, 0.0, -1.0),
                    0.1,
                    Material::Mirror
                ),
                Sphere::new(
                    DVec3::new(-0.4, -0.12, -0.5),
                    0.1,
                    Material::Glass
                ),
                Sphere::new(
                    DVec3::new(0.4, 0.0, -0.5),
                    0.1,
                    Material::Glass
                ),
            ]
        }
    }
}
