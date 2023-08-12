use lumo::tracer::*;
use lumo::*;
use glam::DVec3;

const SCENE_URL: &str = "https://casual-effects.com/g3d/data10/common/model/CornellBox/CornellBox.zip";
const SCENE_NAME: &str = "CornellBox-Sphere.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::perspective(
        0.8 * DVec3::Y + 1.7 * DVec3::Z,
        0.8 * DVec3::Y + 1.7 * DVec3::NEG_Z,
        DVec3::Y,
        90.0,
        0.0,
        1.0,
        1024,
        768,
    );

    let scene = parser::scene_from_url(SCENE_URL, SCENE_NAME)?;

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("cornell.png")?;
    Ok(())
}
