use rust_tracer::*;
use std::fs::File;
use std::io::{Seek, Write};

const FNAME_TMP: &str = "homer.obj.tmp";
const HOMER_URL: &str = "https://raw.githubusercontent.com/alecjacobson/common-3d-test-models/master/data/homer.obj";

fn homer() -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let resp = ureq::get(HOMER_URL)
        .call()?;

    let mut bytes: Vec<u8> = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)?;
    let mut file = File::create(FNAME_TMP)?;
    file.write_all(&bytes)?;
    file.rewind()?;
    let tringls = load_obj_file(File::open(FNAME_TMP)?)?;

    Ok(tringls)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = DVec3::splat(0.95);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(DVec3::new(1.0, 0.0, 0.0))),
        Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 0.0))),
        Material::diffuse(Texture::Solid(def_color)),
    );

    scene.add(
        Mesh::new(
            homer()?,
            Material::diffuse(Texture::Solid(DVec3::new(1.0, 1.0, 0.0))),
        )
            .scale(DVec3::splat(1.5))
            .translate(DVec3::new(-0.73, -1.23, -2.0))
            .make_box()
    );


    let mut renderer = Renderer::new(scene, camera);
    renderer.set_width(1000);
    renderer.set_height(1000);
    renderer.set_integrator(Integrator::PathTrace);
    renderer.render()
        .save("homer.png")?;
    Ok(())
}
