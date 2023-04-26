use lumo::tracer::*;
use lumo::*;

const BUNNY_URL: &str = "https://www.prinmath.com/csci5229/OBJ/bunny.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default(1000, 1000);
    let def_color = srgb_to_linear(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_linear(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 255, 0))),
    );

    scene.add(
        Mesh::new(
            obj::obj_from_url(BUNNY_URL)?,
            Material::transparent(
                Texture::Solid(srgb_to_linear(0, 255, 0)),
                1.5,
                0.1),
        )
        .scale(0.3, 0.3, 0.3)
        .translate(0.0, -0.65, -1.5),
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("bunny.png")?;
    Ok(())
}
