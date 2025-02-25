use lumo::tracer::*;
use lumo::*;

const DRAGON_URL: &str = "https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::builder().build();
    let def_color = Spectrum::from_srgb(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::from(Spectrum::RED)),
        Material::diffuse(Texture::from(Spectrum::GREEN)),
    );

    scene.add(
        parser::mesh_from_url(
            DRAGON_URL,
            Material::transparent(
                Texture::from(Spectrum::MAGENTA),
                0.03,
                1.5,
            ),
        )?
            .to_unit_size()
            .to_origin()
            .rotate_y(5.0 * PI / 8.0)
            .scale_uniform(1.3)
            .set_y(-0.799)
            .translate(0.0, 0.0, -1.4)
    );

    Renderer::new(scene, camera)
        .render()
        .save("dragon.png")?;
    Ok(())
}
