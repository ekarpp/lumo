use lumo::tracer::*;
use lumo::*;

const DRAGON_URL: &str = "https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip";

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
            DRAGON_URL,
            Material::transparent(
                Texture::Solid(Color::new(255, 0, 255)),
                0.03,
                1.5,
            ),
        )?
        .to_unit_size()
        .to_origin()
        .rotate_y(5.0 * PI / 8.0)
        .scale(1.3, 1.3, 1.3)
        .translate(0.0, -0.35, -1.4)
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("dragon.png")?;
    Ok(())
}
