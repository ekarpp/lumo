use lumo::tracer::*;
use lumo::*;

const BUNNY_URL: &str = "https://www.prinmath.com/csci5229/OBJ/bunny.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = Camera::builder().build();
    let def_color = Spectrum::from_srgb(242, 242, 242);
    let mut scene = Scene::empty_box(
        def_color,
        Material::diffuse(Texture::from(Spectrum::RED)),
        Material::diffuse(Texture::from(Spectrum::GREEN)),
    );

    scene.add(
        parser::mesh_from_url(
            BUNNY_URL,
            Material::metal(
                Texture::from(Spectrum::YELLOW),
                0.1,
                2.5,
                3.0,
            ),
        )?
            .to_unit_size()
            .to_origin()
            .set_y(-0.799)
            .translate(0.0, 0.0, -1.5),
    );

    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(256)
        .render()
        .save("bunny.png")?;
    Ok(())
}
