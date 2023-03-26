use rust_tracer::*;
use std::f64::consts::PI;

const DRAGON_URL: &str = "https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = srgb_to_lin(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_lin(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_lin(0, 255, 0))),
        Material::diffuse(Texture::Solid(def_color)),
    );

    scene.add(
        Mesh::new(
            obj_from_url(DRAGON_URL)?,
            Material::metal(
                Texture::Solid(srgb_to_lin(242, 104, 74)),
                0.2
            ),
        )
            .scale(1.2, 1.2, 1.2)
            .rotate_y(5.0 * PI / 8.0)
            .translate(0.0, -0.68, -1.4)
            .make_box()
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("dragon.png")?;
    Ok(())
}
