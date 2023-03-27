use rust_tracer::*;
use rust_tracer::tracer::*;
use glam::DVec3;
use std::f64::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::default();
    let def_color = srgb_to_lin(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        // left
        Material::diffuse(Texture::Solid(srgb_to_lin(0, 255, 255))),
        // right
        Material::diffuse(Texture::Solid(srgb_to_lin(255, 0, 255))),
        // floor
        Material::metal(
            Texture::Checkerboard(
                Box::new(Texture::Solid(def_color)),
                Box::new(Texture::Solid(srgb_to_lin(0, 0, 230))),
                2.42,
            ),
            0.001,
        ),
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
            Texture::Solid(srgb_to_lin(255, 255, 255)),
            1.5,
            0.001,
        ),
    ));

    scene.add(
        Cube::new(
            Material::specular(
                Texture::Solid(srgb_to_lin(0, 230, 0)),
                0.07,
            ))
            .rotate_y(PI / 10.0)
            .scale(0.2, 0.4, 0.2)
            .translate(0.2, -1.0, -1.7)
            .make_box()
    );

    let mut renderer = Renderer::new(scene, camera);
    //renderer.set_tone_map(ToneMap::HableFilmic);
    renderer.render()
        .save("box.png")?;
    Ok(())
}
