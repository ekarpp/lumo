use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::cornell_box();
    let scene = Scene::cornell_box();

    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(512)
        .render()
        .save("cornell.png")?;

    Ok(())
}
