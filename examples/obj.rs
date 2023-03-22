use rust_tracer::*;
use std::f64::consts::PI;
use std::fs::File;

fn main() -> Result<(), std::io::Error> {
    let camera = Camera::default();
    let def_color = DVec3::splat(0.95);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 0.0))),
        Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 0.0))),
        Material::metal(Texture::Marble(Perlin::new(def_color)), 0.1),
    );

    scene.add(
        Mesh::new(
            load_obj_file(File::open("examples/humanoid.obj")?)?,
            Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 0.0)))
        )
            .scale(DVec3::splat(0.025))
            .rotate_z(PI / 2.0)
            .rotate_x(-PI / 2.0)
            .translate(DVec3::new(0.0, -1.0, -1.5))
            .make_box(),
    );

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_samples(1);
    renderer.render().save("obj.png")?;
    Ok(())
}
