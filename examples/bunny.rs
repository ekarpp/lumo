use rust_tracer::*;

const BUNNY_URL: &str = "https://www.prinmath.com/csci5229/OBJ/bunny.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = srgb_to_lin(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_lin(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_lin(0, 255, 0))),
        Material::metal(Texture::Marble(Perlin::new(def_color)), 0.05),
    );

    scene.add(
        Mesh::new(
            obj_from_url(BUNNY_URL)?,
            Material::specular(Texture::Solid(srgb_to_lin(0, 255, 0)), 0.2)
        )
            .scale(0.3, 0.3, 0.3)
            .translate(0.0, -0.9, -1.5)
            .make_box()
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("bunny.png")?;
    Ok(())
}
