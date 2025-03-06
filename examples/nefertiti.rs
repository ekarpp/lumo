use lumo::tracer::*;
use lumo::*;

// By CosmoWenmann (https://www.thingiverse.com/thing:3974391) licensed under CC BY-NC-SA 4.0
const TEX_FILE: &str = "aem_aem21300_3dsl01_mo08-03_p_img.png";
const NEFE_URL: &str = "https://cdn.thingiverse.com/assets/c7/e1/b6/f6/12/SPK_Nefertiti_Scan_FOIA_Response.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scene = Scene::default();

    /* floor */
    scene.add(Plane::new(
        -Vec3::Y,
        Vec3::Y,
        Material::diffuse(Texture::from(Spectrum::BLACK))
    ));

    /* roof */
    scene.add(Plane::new(
        Vec3::Y,
        -Vec3::Y,
        Material::diffuse(Texture::from(Spectrum::BLACK))
    ));

    /* left wall */
    scene.add(Plane::new(
        -Vec3::X,
        Vec3::X,
        Material::diffuse(Texture::from(Spectrum::BLACK)),
    ));

    /* right wall */
    scene.add(Plane::new(
        Vec3::X,
        -Vec3::X,
        Material::diffuse(Texture::from(Spectrum::BLACK))
    ));

    /* front */
    scene.add(Plane::new(
        2.0 * -Vec3::Z,
        Vec3::Z,
        Material::diffuse(Texture::from(Spectrum::BLACK))
    ));

    /* back */
    scene.add(Plane::new(
        Vec3::Z,
        -Vec3::Z,
        Material::diffuse(Texture::from(Spectrum::BLACK))
    ));

    /* bust */
    scene.add(
        Cube::new(
            Material::metal(
                Texture::from(Spectrum::from_srgb(218, 138, 103)),
                0.1, 2.5, 3.0
            ),
        )
            .translate(-0.5, -0.5, -0.5)
            .scale(0.45, 0.5, 0.45)
            .translate(0.0, -0.75, -1.45),
    );

    /* statue */
    if cfg!(debug_assertions) {
	    scene.add(
            Cylinder::new(
		        0.6,
		        0.1,
		        Material::diffuse(Texture::from(Spectrum::RED)))
		        .translate(0.0, -0.5, -1.45)
	    );
    } else {
	    scene.add(
	        parser::mesh_from_url(
                NEFE_URL,
		        Material::microfacet(
                    0.8,
                    1.5,
                    0.0,
                    false,
                    false,
                    Texture::Image(parser::texture_from_url(NEFE_URL, TEX_FILE)?),
                    Texture::from(Spectrum::WHITE),
                    Texture::from(Spectrum::BLACK),
                )
            )?
		        .to_unit_size()
		        .to_origin()
		        .scale_uniform(0.5)
		        .rotate_x(-PI / 2.0)
		        .translate(0.0, -0.2499, -1.45),
	    );
    }

    let xy_rect = Mat3::new(
        Vec3::ZERO,
        Vec3::X,
        Vec3::X + Vec3::Y,
    );

    let theta = PI / 4.0;

    /* light */
    let disk_towards = Vec3::new(-0.046, -0.192, -1.314);
    let disk_dir = Vec3::new(-0.9132592442858147, 0.38194206881899756, -0.05342207528313975);

    // left, w.r.t camera
    scene.add_light(Disk::new(
	    disk_towards + 0.3 * disk_dir,
	    -disk_dir,
	    0.05,
	    Material::Light(Texture::from(30.0 * Spectrum::WHITE))
    ));

    let right_disk_origin = Vec3::new(0.6, 0.15, -1.6);
    scene.add_light(Disk::new(
	    right_disk_origin,
	    disk_towards + 0.28 * Vec3::X - right_disk_origin,
	    0.08,
	    Material::Light(Texture::from(20.0 * Spectrum::WHITE))
    ));

    scene.add_light(Rectangle::new(
        xy_rect.clone(),
        Material::Light(Texture::from(2.0 * Spectrum::WHITE)))
                    .scale(0.4, 0.4, 1.0)
                    .rotate_y(-theta)
                    .rotate_x(theta / 2.0)
                    .rotate_z(theta / 2.0)
                    .translate(0.6, 0.2, -1.7)
    );

    // behind
    scene.add_light(Rectangle::new(
        xy_rect.clone(),
        Material::Light(Texture::from(Spectrum::WHITE)))
                    .scale(0.3, 0.3, 1.0)
                    .rotate_x(2.0 * PI - 2.0 * theta)
                    .translate(-0.15, 0.5, -0.8)
    );
    scene.add_light(Rectangle::new(
	    xy_rect.clone(),
	    Material::Light(Texture::from(2.0 * Spectrum::WHITE)))
		            .scale(0.3, 0.3, 1.0)
		            .rotate_x(PI)
		            .translate(-0.1, 0.0, 0.0)
    );

    // above
    scene.add_light(Rectangle::new(
        xy_rect,
        Material::Light(Texture::from(2.0 * Spectrum::WHITE)))
                    .scale(0.4, 0.4, 1.0)
                    .rotate_x(PI / 2.0)
                    .translate(-0.2, 0.5, -1.5)
    );

    let camera = if cfg!(debug_assertions) {
	    Camera::builder()
	        .origin(0.0, 0.0, 0.5)
            .resolution((1000, 1000))
            .build()
    } else {
	    Camera::builder()
            .origin(0.12, -0.23, -1.205)
            .towards(0.0, -0.26, -1.45)
            .zoom(5.0)
            .resolution((641, 939))
            .camera_type(CameraType::Orthographic)
            .build()
    };

    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(1024)
        .render()
        .save("nefe.png")?;
    Ok(())
}
