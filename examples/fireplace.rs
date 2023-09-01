use lumo::tracer::*;
use lumo::*;

const SCENE_URL: &str = "https://casual-effects.com/g3d/data10/research/model/fireplace_room/fireplace_room.zip";
const SCENE_NAME: &str = "fireplace_room.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::perspective(
        Vec3::new(4.0, 1.0, -3.0),
        0.5 * Vec3::Y,
        Vec3::Y,
        90.0,
        0.0,
        1.0,
        1024,
        768,
    );

    let scene = parser::scene_from_url(SCENE_URL, SCENE_NAME)?;

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("fireplace.png")?;

    Ok(())
}
