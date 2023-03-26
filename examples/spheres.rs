use rust_tracer::*;
use rust_tracer::tracer::*;
use glam::DVec3;
use std::f64::consts::PI;

// consider xyz in cylinder as HSV and map to RGB
fn color_map(x: f64, y: f64, _z: f64, cube_dim: f64) -> DVec3 {
    let v = ((y + cube_dim) / cube_dim).min(1.0);
    let h = x.acos().to_degrees();

    let f = |n: f64| {
        let k = (n + h / 60.0) % 6.0;
        v - v * k.min(4.0 - k).min(1.0).max(0.0)
    };

    let rgb = DVec3::new(
        f(5.0),
        f(3.0),
        f(1.0),
    ) * 255.0;

    srgb_to_lin(
        rgb.x as u8,
        rgb.y as u8,
        rgb.z as u8,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dist = 5.0;
    let camera = Camera::new(
        DVec3::splat(dist), // origin
        DVec3::new(0.0, 0.0, 0.0), // towards
        DVec3::new(-1.0, 1.0, -1.0), // up
        1.0,
    );

    let mut scene = Scene::default();

    let r = 0.1;

    scene.add(
        Sphere::new(
            DVec3::ZERO,
            r,
            Material::Light(Texture::Solid(srgb_to_lin(255, 255, 255))),
        )
    );

    let cube_dim = 10.0;
    let s = 2.0 * cube_dim;
    scene.add(
        Cube::new(Material::diffuse(Texture::Solid(srgb_to_lin(192, 192, 192))))
            .translate(-0.5, -0.5, -0.5)
            .scale(s, s, s)
            .make_box()
    );

    let spheres_in_circle = 10;
    let levels = 20;
    let height = cube_dim / (levels as f64);

    for i in 1..=levels {
        let y = -height * (i as f64);
        let theta_offset = (i as f64 / levels as f64) * 2.0 * PI;
        for j in 0..spheres_in_circle {
            let theta = (j as f64 / spheres_in_circle as f64) * 2.0 * PI
                + theta_offset;
            let x = theta.cos();
            let z = theta.sin();
            let col = color_map(x, y, z, cube_dim);
            scene.add(
                Sphere::new(
                    DVec3::new(x, y, z),
                    r,
                    Material::diffuse(Texture::Solid(col))
                )
            );
        }
    }


    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("spheres.png")?;
    Ok(())
}
