use rust_tracer::Renderer;
use rust_tracer::Scene;
use rust_tracer::Camera;

/// Default output filename

fn main() {
    let scene = Scene::obj_scene(1.0);
    let cam = Camera::default();

    let mut renderer = Renderer::new(scene, cam);
    let start_img = std::time::SystemTime::now();
    renderer.render().save("render.png");
    match start_img.elapsed() {
        Ok(v) => println!("rendered scene in {v:?}"),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }
}
