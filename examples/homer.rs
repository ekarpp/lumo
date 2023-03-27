use rust_tracer::tracer::*;
use rust_tracer::*;

const HOMER_URL: &str =
    "https://raw.githubusercontent.com/alecjacobson/common-3d-test-models/master/data/homer.obj";

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
            obj::obj_from_url(HOMER_URL)?,
            Material::diffuse(Texture::Solid(srgb_to_linear(255, 255, 0))),
        )
        .scale(1.5, 1.5, 1.5)
        .translate(-0.73, -1.23, -2.0),
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("homer.png")?;
    Ok(())
}
