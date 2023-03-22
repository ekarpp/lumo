use rust_tracer::*;
use std::fs::File;
use std::f64::consts::PI;
use rust_tracer::obj;
use rust_tracer::texture::Texture;
use rust_tracer::material::Material;
use rust_tracer::tracer::object::kdtree::Mesh;
use rust_tracer::object::instance::Instanceable;


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
            obj::load_obj_file(File::open("test.obj")?)?,
            Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 0.0)))
        )
            .scale(DVec3::splat(0.025))
            .rotate_z(PI / 2.0)
            .rotate_x(-PI / 2.0)
            .translate(DVec3::new(0.0, -1.0, -1.5))
            .make_box(),
    );

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_samples(4);
    renderer.render().save("render.png")?;
    Ok(())
}
