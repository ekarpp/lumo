use lumo::tracer::*;
use lumo::*;

const SCENE_URL: &str = "https://casual-effects.com/g3d/data10/research/model/conference/conference.zip";
const SCENE_NAME: &str = "conference.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::perspective(
        Vec3::new(-50.0, 400.0, -350.0),
        500.0 * Vec3::X + 250.0 * Vec3::Z,
        Vec3::Y,
        90.0,
        0.0,
        1.0,
        1024,
        768,
    );

    let mut scene = parser::scene_from_url(SCENE_URL, SCENE_NAME)?;

    scene.add_light(Sphere::new(
        Vec3::new(-200.0, 40.0, -400.0),
        100.0,
        Material::Light(Texture::Solid(Color::WHITE)),
    ));

    scene.add_light(Sphere::new(
        Vec3::new(900.0, 300.0, -600.0),
        100.0,
        Material::Light(Texture::Solid(Color::WHITE)),
    ));

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("conference.png")?;

    Ok(())
}
