use rust_tracer::*;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::default();
    let scene = Scene::default();

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_width(1920);
    renderer.set_height(1080);
    renderer.render().save("def.png")?;
    Ok(())
}
