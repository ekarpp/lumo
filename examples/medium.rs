use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::cornell_box();
    let mut scene = Scene::cornell_box();

    scene.set_medium(
        Medium::new(
            RGB::from(Vec3::splat(0.5)),
            RGB::from(Vec3::splat(0.1)),
            0.9,
        )
    );


    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(512)
        .render()
        .save("cornell_medium.png")?;


    Ok(())
}
