use rust_tracer::*;
use std::fs::File;
use std::io::{Seek, Write};

const FNAME_TMP: &str = "teapot.obj.tmp";
const TEAPOT_URL: &str = "https://graphics.stanford.edu/courses/cs148-10-summer/as3/code/as3/teapot.obj";

fn teapot() -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let resp = ureq::get(TEAPOT_URL)
        .call()?;
    assert!(resp.has("Content-Length"));
    let len: usize = resp.header("Content-Length")
        .unwrap()
        .parse()?;

    let mut bytes: Vec<u8> = Vec::with_capacity(len);
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
    let mut scene = Scene::default();

    scene.add(
        Mesh::new(
            teapot()?,
            Material::diffuse(Texture::Solid(DVec3::new(0.0, 1.0, 0.0))),
        )
            .scale(DVec3::splat(0.1))
            .rotate_y(3.14)
            .translate(DVec3::new(0.0, -0.5, -1.5))
            .make_box()
    );


    let mut renderer = Renderer::new(scene, camera);
    renderer.set_width(640);
    renderer.set_height(360);
    renderer.set_integrator(Integrator::DirectLight);
    renderer.render()
        .save("def.png")?;
    Ok(())
}
