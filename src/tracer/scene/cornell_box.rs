use super::*;
use crate::Vec3;
use crate::tracer::{ Color, Face, TriangleMesh };

impl Scene {

    /// Cornell box ported from the original data
    pub fn cornell_box() -> Self {
        let material = |c: Color| -> Material {
            Material::diffuse(Texture::from(c))
        };

        let floor = material(Color::splat(0.9));
        let back_wall = material(Color::splat(0.9));
        let ceiling = material(Color::splat(0.9));
        let left_wall = material(Color::RED);
        let right_wall = material(Color::GREEN);
        let big_box = material(Color::splat(1.1));
        let small_box = material(Color::splat(1.1));

        let c = Color::from(Vec3::new(20.7798, 10.8476, 2.77055));
        let light = Material::Light(Texture::from(c));

        let mut scene = Scene::default();

        let box_faces = || -> Vec<Face> {
            let mut faces = vec!();
            for i in 0..=4 {
                let v0 = i * 4;
                faces.push(Face::new(vec!(v0, v0 + 1, v0 + 2), vec!(), vec!()));
                faces.push(Face::new(vec!(v0, v0 + 2, v0 + 3), vec!(), vec!()));
            }
            faces
        };

        let mut add_object = |v: Vec<Vec3>, m: Material, f: Option<Vec<Face>>| {
            let f = f.unwrap_or(vec!(
                Face::new(vec!(0,1,2), vec!(), vec!()),
                Face::new(vec!(0,2,3), vec!(), vec!()),
            ));
            let light = matches!(m, Material::Light(..));
            let mesh = Box::new(TriangleMesh::new(v, f, vec!(), vec!(), m));
            if light {
                scene.add_light(mesh);
            } else {
                scene.add(mesh);
            }
        };


        /* floor */
        {
            let vertices = vec!(
                Vec3::new(552.8, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 559.2),
                Vec3::new(549.6, 0.0, 559.2),
            );
            add_object(vertices, floor, None);
        }

        /* ceil */
        {
            let vertices = vec!(
                Vec3::new(556.0, 548.8, 0.0),
                Vec3::new(556.0, 548.8, 559.2),
                Vec3::new(0.0, 548.8, 559.2),
                Vec3::new(0.0, 548.8, 0.0),
            );
            add_object(vertices, ceiling, None);
        }

        /* back wall */
        {
            let vertices = vec!(
                Vec3::new(549.6, 0.0, 559.2),
                Vec3::new(0.0, 0.0, 559.2),
                Vec3::new(0.0, 548.8, 559.2),
                Vec3::new(556.0, 548.8, 559.2),
            );
            add_object(vertices, back_wall, None);
        }

        /* right wall */
        {
            let vertices = vec!(
                Vec3::new(0.0, 0.0, 559.2),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 548.8, 0.0),
                Vec3::new(0.0, 548.8, 559.2),
            );
            add_object(vertices, right_wall, None);
        }

        /* left wall */
        {
            let vertices = vec!(
                Vec3::new(552.8, 0.0, 0.0),
                Vec3::new(549.6, 0.0, 559.2),
                Vec3::new(556.0, 548.8, 559.2),
                Vec3::new(556.0, 548.8, 0.0),
            );
            add_object(vertices, left_wall, None);
        }

        /* light */
        {
            let vertices = vec!(
                Vec3::new(343.0, 548.8, 227.0),
                Vec3::new(343.0, 548.8, 332.0),
                Vec3::new(213.0, 548.8, 332.0),
                Vec3::new(213.0, 548.8, 227.0),
            );
            add_object(vertices, light, None);
        }


        /* small box */
        {
            let vertices = vec!(
                Vec3::new(130.0, 165.0, 65.0),
                Vec3::new(82.0, 165.0, 225.0),
                Vec3::new(240.0, 165.0, 272.0),
                Vec3::new(290.0, 165.0, 114.0),

                Vec3::new(290.0, 0.0, 114.0),
                Vec3::new(290.0, 165.0, 114.0),
                Vec3::new(240.0, 165.0, 272.0),
                Vec3::new(240.0, 0.0, 272.0),

                Vec3::new(130.0, 0.0, 65.0),
                Vec3::new(130.0, 165.0, 65.0),
                Vec3::new(290.0, 165.0, 114.0),
                Vec3::new(290.0, 0.0, 114.0),

                Vec3::new(82.0, 0.0, 225.0),
                Vec3::new(82.0, 165.0, 225.0),
                Vec3::new(130.0, 165.0, 65.0),
                Vec3::new(130.0, 0.0, 65.0),

                Vec3::new(240.0, 0.0, 272.0),
                Vec3::new(240.0, 165.0, 272.0),
                Vec3::new(82.0, 165.0, 225.0),
                Vec3::new(82.0, 0.0, 225.0),
            );

            add_object(vertices, small_box, Some(box_faces()));
        }

        /* big box */
        {
            let vertices = vec!(
                Vec3::new(423.0, 330.0, 247.0),
                Vec3::new(265.0, 330.0, 296.0),
                Vec3::new(314.0, 330.0, 456.0),
                Vec3::new(472.0, 330.0, 406.0),

                Vec3::new(423.0, 0.0, 247.0),
                Vec3::new(423.0, 330.0, 247.0),
                Vec3::new(472.0, 330.0, 406.0),
                Vec3::new(472.0, 0.0, 406.0),

                Vec3::new(472.0, 0.0, 406.0),
                Vec3::new(472.0, 330.0, 406.0),
                Vec3::new(314.0, 330.0, 456.0),
                Vec3::new(314.0, 0.0, 456.0),

                Vec3::new(314.0, 0.0, 456.0),
                Vec3::new(314.0, 330.0, 456.0),
                Vec3::new(265.0, 330.0, 296.0),
                Vec3::new(265.0, 0.0, 296.0),

                Vec3::new(265.0, 0.0, 296.0),
                Vec3::new(265.0, 330.0, 296.0),
                Vec3::new(423.0, 330.0, 247.0),
                Vec3::new(423.0, 0.0, 247.0),
            );

            add_object(vertices, big_box, Some(box_faces()));
        }

        scene
    }
}
