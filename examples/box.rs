use glam::DVec3;
use lumo::tracer::*;
use lumo::*;
use std::f64::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::default(1000, 1000);
    let def_color = srgb_to_linear(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        // left
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 255, 255))),
        // right
        Material::diffuse(Texture::Solid(srgb_to_linear(255, 0, 255))),
    );

    scene.add(Sphere::new(
        DVec3::new(-0.4, -0.8, -1.4),
        0.2,
        Material::Mirror,
    ));

    scene.add(Sphere::new(
        DVec3::new(0.3, -0.8, -1.2),
        0.1,
        Material::transparent(
            Texture::Solid(srgb_to_linear(255, 255, 255)),
            1.5,
            0.0
        ),
    ));

    scene.add(
        Cube::new(Material::metal(
            Texture::Solid(srgb_to_linear(0, 230, 0)),
            0.1,
        ))
        .rotate_y(PI / 10.0)
        .scale(0.2, 0.4, 0.2)
        .translate(0.2, -1.0, -1.7),
    );

    scene.set_medium(
        Medium::new(0.05, DVec3::ZERO, srgb_to_linear(255, 255, 255))
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render().save("box.png")?;
    Ok(())
}
