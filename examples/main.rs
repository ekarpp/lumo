use glam::f64::DVec3;
use rust_tracer::Renderer;
use rust_tracer::Scene;
use rust_tracer::Camera;

/// Default output filename
const FNAME: &str = "render.png";

fn main() {
    let fl = 1.0;
    let scene = Scene::obj_scene(fl);
    let cam = Camera::new(
        1.0,
        90.0,
        DVec3::new(0.0, 0.0, 0.0), /* origin */
        DVec3::new(0.0, 0.0, -1.0), /* towards */
        DVec3::new(0.0, 1.0, 0.0), /* up */
        fl, /* focal length */
    );

    let mut renderer = Renderer::new(scene, cam);
    let start_img = std::time::SystemTime::now();
    renderer.render().save(FNAME);
    match start_img.elapsed() {
        Ok(v) => println!("rendered scene in {v:?}"),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }
}
