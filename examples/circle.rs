use lumo::tracer::*;
use lumo::*;

// s = v = 1.0. h in radians
fn hsv_to_rgb(h: Float) -> Spectrum {
    let f = |n: Float| {
        let k = (n + h / (PI / 3.0)) % 6.0;
        1.0 - k.min(4.0 - k).clamp(0.0, 1.0)
    };

    let rgb = Vec3::new(f(5.0), f(3.0), f(1.0)) * 255.0;

    Spectrum::from_srgb(rgb.x as u8, rgb.y as u8, rgb.z as u8)
}

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::builder()
        .origin(0.0, 1.0, 1.5)
        .towards(0.0, -0.5, 0.0)
        .up(0.0, 1.0, -1.0)
        .resolution((512, 512))
        .build();

    let mut scene = Scene::default();
    let ground = -0.2;

    // ground
    scene.add(Disk::new(
        ground * Vec3::Y,
        Vec3::Y,
        100.0,
        Material::mirror(),
    ));

    let r = 0.2;
    scene.add_light(Sphere::new(
        r,
        Material::light(Texture::from(0.01 * Spectrum::WHITE)))
                    .translate(0.0, ground + r + 0.1, 0.0)
    );

    let circle_s = 8;
    let offset = PI / circle_s as Float;

    for i in 0..circle_s {
        let theta = (i as Float / circle_s as Float) * 2.0 * PI + offset;
        let x = theta.cos();
        let y = ground + r;
        let z = theta.sin();

        scene.add(Sphere::new(
            r,
            Material::diffuse(Texture::from(hsv_to_rgb(theta - offset))))
                  .translate(x, y, z)
        );
    }

    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(512)
        .render()
        .save("circle.png")?;
    Ok(())
}
