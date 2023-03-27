use rust_tracer::*;
use rust_tracer::tracer::*;
use std::f64::consts::PI;

// By CosmoWenmann (https://www.thingiverse.com/thing:3974391) licensed under CC BY-NC-SA 4.0
const NEFE_URL: &str = "https://cdn.thingiverse.com/assets/c7/e1/b6/f6/12/SPK_Nefertiti_Scan_FOIA_Response.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = srgb_to_linear(242, 242, 242);
    let silver = srgb_to_linear(192, 192, 192);
    let gold = srgb_to_linear(255, 215, 0);

    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_linear(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 255, 0))),
        Material::diffuse(Texture::Solid(def_color)),
    );


    scene.add(
        Mesh::new(
            obj::obj_from_url(NEFE_URL)?,
            Material::metal(Texture::Solid(gold), 0.06)
        )
            .to_unit_size()
            .to_origin()
            .scale(0.9, 0.9, 0.9)
            .rotate_x(-PI / 2.0)
            .translate(0.0, -0.07, -1.35)
            .make_box()
    );

    scene.add(
        Cube::new(Material::specular(Texture::Marble(Perlin::new(silver)), 0.06))
            .translate(-0.5, -0.5, -0.5)
            .scale(0.4, 1.2, 0.4)
            .translate(0.0, -1.1, -1.35)
            .make_box()
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("nefe.png")?;

    Ok(())
}
