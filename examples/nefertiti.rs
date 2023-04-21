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
        Cube::new(Material::metal(
            Texture::Solid(srgb_to_linear(61, 45, 36)),
            0.0,
        ))
        .translate(-0.5, -0.5, -0.5)
        .scale(0.4, 0.5, 0.4)
        .translate(0.0, -0.75, -1.45),
    );

    /* statue */
    if cfg!(debug_assertions) {
	scene.add(
            Cylinder::new(
		0.0,
		0.6,
		0.1,
		Material::diffuse(Texture::Solid(srgb_to_linear(255, 0, 0))))
		.translate(0.0, -0.5, -1.45)
	);
    } else {
	scene.add(
            Mesh::new(
		obj::obj_from_url(NEFE_URL)?,
		Material::diffuse(Texture::Image(Image::from_file(IMAGE_FILE)?)),
            )
		.to_unit_size()
		.to_origin()
		.scale(0.6, 0.6, 0.6)
		.rotate_x(-PI / 2.0)
		.translate(0.0, -0.2, -1.45),
	);
    }

    let xy_rect = DMat3::from_cols(
        DVec3::ZERO,
        DVec3::X,
        DVec3::X + DVec3::Y,
    );

    let theta = PI / 4.0;

    /* light */
    // left, w.r.t camera
    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::Solid(3.0 * DVec3::ONE)))
                    .scale(0.3, 1.1, 1.0)
                    .rotate_y(theta)
                    .rotate_axis(DVec3::new(theta.cos(), 0.0, -theta.sin()), PI / 8.0)
                    .translate(-0.95, 0.0, -1.55)
    );

    // right
    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::Solid(2.0 * DVec3::ONE)))
                    .scale(0.2, 1.0, 1.0)
                    .rotate_y(-theta)
		    .rotate_axis(DVec3::new(theta.cos(), 0.0, theta.sin()), PI / 8.0)
                    .translate(0.6, 0.2, -1.7)
    );

    // behind
    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))))
                    .scale(0.3, 0.3, 1.0)
                    .rotate_x(-theta)
                    .translate(-0.15, 0.6, 0.5)
    );

    // above
    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))))
                    .scale(0.4, 0.4, 1.0)
                    .rotate_x(PI / 2.0)
                    .translate(-0.2, 0.99, -1.5)
    );

    let camera = if cfg!(debug_assertions) {
	Camera::default(1000, 1000)
    } else {
	Camera::orthographic(
            DVec3::new(0.15, -0.2, -1.17),
            DVec3::new(0.0, -0.25, -1.45),
            DVec3::Y,
            0.26,
            683,
            1000,
	)
    };

    let mut renderer = Renderer::new(scene, camera);
    if cfg!(debug_assertions) {
	renderer.set_samples(9)
    } else {
	renderer.set_samples(25)
    }
    renderer.render().save("nefe.png")?;

    Ok(())
}
