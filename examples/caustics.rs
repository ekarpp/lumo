use lumo::tracer::*;
use lumo::*;

const OBJ_URL: &str = "https://www.prinmath.com/csci5229/OBJ/suzanne.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::builder()
        .origin(0.0, 0.0, 2.0)
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

    let suzanne = parser::mesh_from_url(OBJ_URL, Material::Blank)?
        .to_unit_size();

    scene.add(suzanne.clone(Some(Material::mirror()))
              .to_origin()
              .rotate_y(-PI / 8.0)
              .rotate_z(PI / 8.0)
              .rotate_x(-PI / 8.0)
              .translate(0.5, -0.3, -1.0),
    );

    scene.add(suzanne.clone(Some(Material::glass()))
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
