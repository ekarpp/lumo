use rust_tracer::*;
use std::fs::File;
use std::io::{Seek, Write, Cursor, Read};
use std::f64::consts::PI;
use zip::ZipArchive;

const FNAME_TMP: &str = "dragon.obj.tmp";
const DRAGON_URL: &str = "https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip";

fn dragon() -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    ureq::get(DRAGON_URL)
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)?;

    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    let mut bytes = Vec::new();
    zip.by_name("dragon.obj")?.read_to_end(&mut bytes)?;

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
            dragon()?,
            Material::metal(
                Texture::Solid(DVec3::new(242.0, 104.0, 74.0) / 256.0),
                0.2
            ),
        )
            .scale(DVec3::splat(1.2))
            .rotate_y(5.0 * PI / 8.0)
            .translate(DVec3::new(0.0, -0.68, -1.4))
            .make_box()
    );

    rayon::ThreadPoolBuilder::new().num_threads(30).build_global().unwrap();

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_width(1000);
    renderer.set_height(1000);
    renderer.set_samples(1);
    renderer.set_integrator(Integrator::PathTrace);
    renderer.render()
        .save("dragon.png")?;
    Ok(())
}
