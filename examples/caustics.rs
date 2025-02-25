use lumo::tracer::*;
use lumo::*;

const OBJ_URL: &str = "https://www.prinmath.com/csci5229/OBJ/suzanne.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::builder()
        .origin(2.0 * Vec3::Z)
        .zoom(3.0)
        .build();
    let def_color = Spectrum::from_srgb(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        // left
        Material::diffuse(Texture::from(Spectrum::MAGENTA)),
        // right
        Material::diffuse(Texture::from(Spectrum::CYAN)),
    );

    scene.add(
        parser::mesh_from_url(
            OBJ_URL,
            Material::mirror(),
        )?
            .to_unit_size()
            .to_origin()
            .rotate_y(-PI / 8.0)
            .rotate_z(PI / 8.0)
            .rotate_x(-PI / 8.0)
            .translate(0.5, -0.3, -1.0),
    );

    scene.add(
        parser::mesh_from_url(
            OBJ_URL,
            Material::glass(),
        )?
            .to_unit_size()
            .to_origin()
            .rotate_y(PI / 8.0)
            .rotate_z(-PI / 8.0)
            .rotate_x(PI / 16.0)
            .translate(-0.35, 0.25, -1.25),
    );

    Renderer::new(scene, camera)
        .integrator(Integrator::BDPathTrace)
        .samples(2048)
        .render()
        .save("caustics.png")?;
    Ok(())
}
