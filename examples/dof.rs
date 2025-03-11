use lumo::tracer::*;
use lumo::*;

const TEAPOT_URL: &str = "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.zip";

fn marble_texture(seed: u64) -> Texture {
    Texture::Marble(
        Perlin::new(seed),
        Spectrum::from_srgb(255, 245, 255)
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let o = 0.75 * -Vec3::X + 0.25 * Vec3::Y;
    let t = 0.75 * -Vec3::Y + -Vec3::Z;

    let camera = Camera::builder()
        .origin(o.x, o.y, o.z)
        .towards(t.x, t.y, t.z)
        .lens_radius(0.03)
        .focal_length(o.distance(t))
        .camera_type(CameraType::Orthographic)
        .build();
    let mut scene = Scene::default();

    let checkerboard = Material::diffuse(Texture::Checkerboard(
        Box::new(Texture::from(Spectrum::BLACK)),
        Box::new(Texture::from(Spectrum::WHITE)),
        100.0,
    ));
    scene.add(Rectangle::unit_xz(checkerboard)
              .scale_uniform(10.0)
              .translate(0.0, -1.0, 0.0)
    );

    scene.add_light(
        Rectangle::unit_xz(Material::light(Texture::from(0.25 * Spectrum::WHITE)))
                    .rotate_z(PI)
                    .scale_uniform(3.0)
                    .translate(0.0, 8.0, -1.5)
    );

    let teapot = parser::mesh_from_url(TEAPOT_URL, Material::Blank)?
        .to_unit_size();

    for i in 0..3 {
        scene.add(
            teapot
                .clone(Some( Material::diffuse(marble_texture(3 * (i + 2))) ))
                .to_origin()
                .rotate_y(-PI / 4.0)
                .translate(0.0, -0.75, -1.0 * i as Float)

        );
    }

    Renderer::new(scene, camera)
        .integrator(Integrator::PathTrace)
        .samples(512)
        .render()
        .save("dof.png")?;
    Ok(())
}
