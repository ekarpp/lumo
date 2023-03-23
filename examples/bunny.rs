use rust_tracer::*;
use std::fs::File;
use std::io::{Seek, Write, Cursor, Read};
use zip::ZipArchive;

const FNAME_TMP: &str = "bunny.obj.tmp";
const BUNNY_URL: &str = "https://www.prinmath.com/csci5229/OBJ/bunny.zip";

fn bunny() -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    ureq::get(BUNNY_URL)
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)?;

    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    let mut bytes = Vec::new();
    zip.by_name("bunny.obj")?.read_to_end(&mut bytes)?;

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
        Material::metal(Texture::Marble(Perlin::new(def_color)), 0.05),
    );

    scene.add(
        Mesh::new(
            bunny()?,
            Material::specular(Texture::Solid(DVec3::new(0.0, 1.0, 0.0)), 0.2),
        )
            .scale(DVec3::splat(0.1))
        //.translate(DVec3::new(0.0, -1.1, -1.3))
            .translate(DVec3::new(0.0, -0.5, -1.3))
            .make_box()
    );


    let mut renderer = Renderer::new(scene, camera);
    renderer.set_width(500);
    renderer.set_height(500);
    renderer.set_integrator(Integrator::DirectLight);
    renderer.render()
        .save("bunny.png")?;
    Ok(())
}
