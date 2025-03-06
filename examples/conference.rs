use lumo::tracer::*;
use lumo::*;

const SCENE_URL: &str = "https://casual-effects.com/g3d/data10/research/model/conference/conference.zip";
const SCENE_NAME: &str = "conference.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::builder()
        .origin(-50.0, 400.0, -350.0)
        .towards(500.0, 0.0, 250.0)
        .build();

    let mut scene = parser::scene_from_url(SCENE_URL, SCENE_NAME)?;

    scene.add_light(
        Sphere::new(100.0, Material::Light(Texture::from(Spectrum::WHITE)))
            .translate(-200.0, 40.0, -400.0)
    );


    scene.add_light(
        Sphere::new(100.0, Material::Light(Texture::from(Spectrum::WHITE)))
            .translate(900.0, 300.0, -600.0)
    );

    Renderer::new(scene, camera)
        .render()
        .save("conference.png")?;

    Ok(())
}
