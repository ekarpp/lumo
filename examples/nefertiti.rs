use lumo::tracer::*;
use lumo::*;
use std::f64::consts::PI;
use glam::{DVec3, DMat3};
// By CosmoWenmann (https://www.thingiverse.com/thing:3974391) licensed under CC BY-NC-SA 4.0
const IMAGE_FILE: &str = "examples/aem_aem21300_3dsl01_mo08-03_p_img.png";
const NEFE_URL: &str = "https://cdn.thingiverse.com/assets/c7/e1/b6/f6/12/SPK_Nefertiti_Scan_FOIA_Response.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scene = Scene::default();

    /* floor */
    scene.add(Plane::new(
        DVec3::NEG_Y,
        DVec3::Y,
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0)))
    ));

    /* roof */
    scene.add(Plane::new(
        DVec3::Y,
        DVec3::NEG_Y,
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0)))
    ));

    /* left wall */
    scene.add(Plane::new(
        DVec3::NEG_X,
        DVec3::X,
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0))),
    ));

    /* right wall */
    scene.add(Plane::new(
        DVec3::X,
        DVec3::NEG_X,
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0)))
    ));

    /* front */
    scene.add(Plane::new(
        2.0 * DVec3::NEG_Z,
        DVec3::Z,
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0)))
    ));

    /* back */
    scene.add(Plane::new(
        DVec3::Z,
        DVec3::NEG_Z,
        Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 0)))
    ));

    /* bust */
    scene.add(
        Cube::new(Material::specular(
            Texture::Solid(srgb_to_linear(61, 45, 36)),
            0.0,
        ))
        .translate(-0.5, -0.5, -0.5)
        .scale(0.4, 1.2, 0.4)
        .translate(0.0, -1.1, -1.45),
    );

    /* statue */
    scene.add(
        Mesh::new(
            obj::obj_from_url(NEFE_URL)?,
            Material::diffuse(Texture::Image(Image::from_file(IMAGE_FILE)?)),
        )
        .to_unit_size()
        .to_origin()
        .scale(0.45, 0.45, 0.45)
        .rotate_x(-PI / 2.0)
        .translate(0.0, -0.07, -1.45),
    );

    /*
    scene.add(
        Cube::new(Material::specular(Texture::Solid(srgb_to_linear(255, 0, 0)), 0.1))
            .translate(-0.5, -0.5, -0.5)
            .scale(0.2, 0.8, 0.2)
            .translate(0.0, -0.4, -1.45)
    );
    */
    let xy_rect = DMat3::from_cols(
        DVec3::ZERO,
        DVec3::X,
        DVec3::X + DVec3::Y,
    );

    let theta = PI / 4.0;

    /* light */
    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))))
                    .scale(0.2, 0.8, 0.5)
                    .rotate_y(theta)
                    .rotate_axis(DVec3::new(theta.cos(), 0.0, -theta.sin()), PI / 8.0)
                    .translate(-0.95, 0.0, -1.55)
    );

    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))))
                    .scale(0.2, 0.2, 0.2)
                    .rotate_y(-theta)
                    .translate(0.8, 0.0, -1.5)
    );

    let camera = Camera::new(
        DVec3::new(0.15, -0.25, -0.99),
        DVec3::new(0.0, -0.25, -1.45),
        DVec3::Y,
        1.0
    );

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_width(683);
    renderer.render().save("nefe.png")?;

    Ok(())
}
