use glam::DVec3;
use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::default(1000, 1000);
    let def_color = srgb_to_linear(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        // left
        Material::diffuse(Texture::Solid(srgb_to_linear(255, 0, 255))),
        // right
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 255, 255))),
    );

    scene.add(Sphere::new(
        DVec3::new(-0.45, -0.5, -1.5),
        0.25,
        Material::metal(
            Texture::Solid(srgb_to_linear(255, 255, 255)),
            0.0,
        )
    ));

    scene.add(Sphere::new(
        DVec3::new(0.45, -0.5, -1.3),
        0.25,
        Material::transparent(
            Texture::Solid(srgb_to_linear(255, 255, 255)),
            1.5,
            0.0
        ),
    ));

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("box.png")?;
    Ok(())
}
