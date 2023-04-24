use lumo::tracer::*;
use lumo::*;
use std::f64::consts::PI;

const DRAGON_URL: &str = "https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip";

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
            obj::obj_from_url(DRAGON_URL)?,
            Material::transparent(
                Texture::Solid(srgb_to_linear(255, 0, 255)),
                1.5,
                0.03
            ),
        )
        .to_unit_size()
        .to_origin()
        .rotate_y(5.0 * PI / 8.0)
        .scale(1.3, 1.3, 1.3)
        .translate(0.0, -0.55, -1.4)
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("dragon.png")?;
    Ok(())
}
