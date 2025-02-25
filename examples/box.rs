use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::builder().build();
    let def_color = Color::new(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        // left
        Material::diffuse(Texture::from(Color::new(255, 0, 255))),
        // right
        Material::diffuse(Texture::from(Color::new(0, 255, 255))),
    );

    scene.add(Sphere::new(0.25, Material::mirror())
              .translate(-0.45, -0.5, -1.5));

    scene.add(Sphere::new(0.25, Material::glass())
              .translate(0.45, -0.5, -1.3));

    Renderer::new(scene, camera)
        .render()
        .save("box.png")?;
    Ok(())
}
