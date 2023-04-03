use lumo::tracer::*;
use lumo::*;

const TEAPOT_URL: &str = "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = srgb_to_linear(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_linear(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 255, 0))),
        Material::diffuse(Texture::Solid(def_color)),
    );

    scene.add(
        Mesh::new(
            obj::obj_from_url(TEAPOT_URL)?,
            Material::diffuse(Texture::Marble(Perlin::default())),
        )
        .scale(0.1, 0.1, 0.1)
        .translate(0.0, -0.5, -1.5),
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("teapot.png")?;
    Ok(())
}
