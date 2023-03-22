use rust_tracer::*;
use std::f64::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::default();
    let def_color = DVec3::splat(0.95);
    let mut scene = Scene::empty_box(
        def_color,
        // left
        Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 1.0))),
        // right
        Material::diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
        // floor
        Material::metal(
            Texture::Checkerboard(
                Box::new(Texture::Solid(def_color)),
                Box::new(Texture::Solid(DVec3::new(0.0, 0.0, 0.9))),
                2.42,
            ),
            0.01,
        ),
    );

    scene.add(Sphere::new(
        DVec3::new(-0.4, -0.8, -1.4),
        0.2,
        Material::Mirror,
    ));

    scene.add(Sphere::new(
        DVec3::new(0.0, -0.9, -1.2),
        0.1,
        Material::Glass,
    ));

    scene.add(
        Cube::new(
            Material::specular(
                Texture::Solid(DVec3::new(0.0, 0.9, 0.0)),
                0.1,
            ))
            .rotate_y(PI / 10.0)
            .scale(DVec3::new(0.2, 0.4, 0.2))
            .translate(DVec3::new(0.2, -1.0, -1.7))
            .make_box()
    );


    let renderer = Renderer::new(scene, camera);
    renderer.render().save("box.png")?;
    Ok(())
}
