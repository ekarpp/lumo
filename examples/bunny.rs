use lumo::tracer::*;
use lumo::*;

const BUNNY_URL: &str = "https://www.prinmath.com/csci5229/OBJ/bunny.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default(1024, 768);
    let def_color = Color::new(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(Color::new(255, 0, 0))),
        Material::diffuse(Texture::Solid(Color::new(0, 255, 0))),
    );

    scene.add(
        parser::mesh_from_url(
            BUNNY_URL,
            Material::transparent(
                Texture::Solid(Color::new(0, 255, 0)),
                0.1,
                1.5,
            ),
        )?
        .scale(0.3, 0.3, 0.3)
        .translate(0.0, -0.65, -1.5),
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("bunny.png")?;
    Ok(())
}
