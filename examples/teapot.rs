use rust_tracer::*;

const TEAPOT_URL: &str = "https://graphics.stanford.edu/courses/cs148-10-summer/as3/code/as3/teapot.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let mut scene = Scene::default();

    scene.add(
        Sphere::new(
            DVec3::ZERO,
            10.0,
            Material::diffuse(Texture::Solid(srgb_to_lin(0, 0, 0)))
        )
    );

    scene.add(
        Sphere::new(
            DVec3::new(0.0, 0.5, -1.0),
            0.25,
            Material::Light(Texture::Solid(srgb_to_lin(255, 255, 255))),
        ),
    );

    scene.add(
        Mesh::new(
            obj_from_url(TEAPOT_URL)?,
            Material::diffuse(Texture::Solid(srgb_to_lin(0, 255, 0))),
        )
            .scale(DVec3::splat(0.1))
            .rotate_y(3.14)
            .translate(DVec3::new(0.5, -0.5, -1.5))
            .make_box()
    );


    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("teapot.png")?;
    Ok(())
}
