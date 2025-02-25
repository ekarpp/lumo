use lumo::tracer::*;
use lumo::*;

const TEAPOT_URL: &str = "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let o = 0.75 * Vec3::NEG_X + 0.25 * Vec3::Y;
    let t = 0.75 * Vec3::NEG_Y + Vec3::NEG_Z;

    let camera = Camera::builder()
        .origin(o)
        .towards(t)
        .lens_radius(0.03)
        .focal_length(o.distance(t))
        .camera_type(CameraType::Orthographic)
        .build();
    let mut scene = Scene::default();

    scene.add(Plane::new(
        Vec3::NEG_Y,
        Vec3::Y,
        Material::diffuse(Texture::Checkerboard(
            Box::new(Texture::from(Spectrum::BLACK)),
            Box::new(Texture::from(Spectrum::WHITE)),
            10.0,
        ))
    ));

    scene.add_light(Sphere::new(
        4.0,
        Material::Light(Texture::from(Spectrum::WHITE)))
                    .translate(0.0, 8.0, -1.5)
    );

    for i in 0..3 {
        scene.add(
            parser::mesh_from_url(
                TEAPOT_URL,
                Material::diffuse(
                    Texture::Marble(Perlin::default(), Spectrum::from_srgb(255, 245, 255))
                ),
            )?
                .to_unit_size()
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
