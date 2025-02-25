use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = CameraBuilder::new()
        .origin(Vec3::new(278.0, 273.0, -800.0))
        .towards(Vec3::new(278.0, 273.0, 0.0))
        .zoom(2.8)
        .focal_length(0.035)
        .resolution((512, 512))
        .build();

    let scene = Scene::cornell_box();

    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(512)
        .render()
        .save("cornell.png")?;
    Ok(())
}
