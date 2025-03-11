use lumo::tracer::*;
use lumo::*;

const SCENE_URL: &str = "https://files.karppinen.xyz/BistroV0.zip";
const NIGHT: bool = true;
const INTERIOR: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let camera = if INTERIOR {
        Camera::builder()
            .origin(0.0, 2.5, 0.0)
            .towards(7.0, 0.0, -4.0)
            .build()
    } else {
        Camera::builder()
            .origin(-16.0, 5.0, -1.0)
            .towards(0.0, 0.0, 0.0)
            .build()
    };

    let scene = if NIGHT && !INTERIOR {
        parser::scene_from_url(
            SCENE_URL,
            "exterior.obj",
            false,
            Some("exterior-night.mtl"),
            Some(("cobblestone_street_night_4k.hdr", 0.001)),
        )?
    } else {
        parser::scene_from_url(
            SCENE_URL,
            if INTERIOR { "interior.obj" } else { "exterior.obj" },
            false,
            None,
            Some(("san_giuseppe_bridge_4k.hdr", 0.05)),
        )?
    };

    let ig = Integrator::BDPathTrace;
    let samples = 1024;
    let file_name = if NIGHT && !INTERIOR {
        "bistro_night.png"
    } else if INTERIOR {
        "bistro_interior.png"
    } else {
        "bistro_day.png"
    };

    Renderer::new(scene, camera)
        .integrator(ig)
        .samples(samples)
        .tone_map(ToneMap::Reinhard)
        .render()
        .save(file_name)?;

    Ok(())
}
