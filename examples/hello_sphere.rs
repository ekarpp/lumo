use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), png::EncodingError> {
    let camera = Camera::default(1280, 720);
    let mut scene = Scene::default();

    scene.add(Plane::new(
        Vec3::NEG_Y,
        Vec3::Y,
        Material::diffuse(Texture::Solid(Color::new(190, 200, 210))),
    ));

    scene.add_light(Sphere::new(
        8.0 * Vec3::Y + 1.5 * Vec3::NEG_Z,
        4.0,
        Material::Light(Texture::Solid(Color::new(255, 255, 255))),
    ));

    scene.add(
        Sphere::new(
            Vec3::ZERO,
            1.0,
            Material::diffuse(Texture::Solid(Color::new(0, 0, 255))),
        )
        .scale(0.3, 0.3, 0.3)
        .translate(0.0, -0.7, -1.5),
    );

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_samples(36);
    renderer.render().save("hello.png")
}
