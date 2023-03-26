use rust_tracer::*;
use std::f64::consts::PI;

// By CosmoWenmann (https://www.thingiverse.com/thing:3974391) licensed under CC BY-NC-SA 4.0
const NEFE_URL: &str = "https://cdn.thingiverse.com/assets/c7/e1/b6/f6/12/SPK_Nefertiti_Scan_FOIA_Response.zip";
const PEDE_PATH: &str = "examples/nefe_pedestal.obj";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::default();
    let def_color = srgb_to_lin(242, 242, 242);
    let silver = srgb_to_lin(192, 192, 192);
    let gold = srgb_to_lin(255, 215, 0);

    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::Solid(srgb_to_lin(255, 0, 0))),
        Material::diffuse(Texture::Solid(srgb_to_lin(0, 255, 0))),
        Material::diffuse(Texture::Solid(def_color)),
    );


    scene.add(
        Mesh::new(
            obj_from_url(NEFE_URL)?,
            Material::metal(Texture::Solid(gold), 0.06)
        )
            .to_unit_size()
            .to_origin()
            .scale(DVec3::splat(0.9))
            .rotate_x(-PI / 2.0)
            .translate(DVec3::new(0.0, 0.0, -1.1))
            .make_box()
    );


    scene.add(
        Mesh::new(
            obj_from_path(PEDE_PATH)?,
            Material::metal(Texture::Solid(silver), 0.06)
        )
            .to_unit_size()
            .to_origin()
            .scale(DVec3::splat(0.8))
            .rotate_y(3.0 * PI / 4.0)
            .translate(DVec3::new(-0.0, -0.66, -1.1))
            .make_box()
    );

    let renderer = Renderer::new(scene, camera);
    renderer.render()
        .save("nefe.png")?;

    Ok(())
}
