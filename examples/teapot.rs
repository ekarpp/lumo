use rust_tracer::*;

const TEAPOT_URL: &str = "https://graphics.stanford.edu/courses/cs148-10-summer/as3/code/as3/teapot.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = srgb_to_lin(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_lin(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_lin(0, 255, 0))),
        Material::diffuse(Texture::Solid(def_color)),
    );

    /*
    let mut scene = Scene::default();
    scene.add(
        Sphere::new(
            DVec3::new(0.0, 1.0, -1.0),
            0.2,
            Material::Light(Texture::Solid(srgb_to_lin(255, 255, 255)))
        )
    );
     */
    scene.add(
        Mesh::new(
            obj_from_url(TEAPOT_URL)?,
            Material::diffuse(Texture::Solid(srgb_to_lin(0, 0, 255)))//Marble(Perlin::default()))
        )
            .scale(DVec3::splat(0.1))
            .translate(DVec3::new(0.0, -0.5, -1.5))
            .make_box()
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("teapot.png")?;
    Ok(())
}
