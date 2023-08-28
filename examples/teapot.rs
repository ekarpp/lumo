use lumo::tracer::*;
use lumo::*;

const TEAPOT_URL: &str = "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.zip";

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
            TEAPOT_URL,
            Material::transparent(
                Texture::Marble(Perlin::default(), Color::new(255, 245, 255)),
                0.0,
                1.5,
            ),
        )?
            .to_unit_size()
            .to_origin()
            .rotate_y(PI / 4.0)
            .translate(-0.3, -0.5, -1.3),
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("teapot.png")?;
    Ok(())
}
