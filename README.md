## Lumo
[![crates.io](https://img.shields.io/crates/v/lumo)](https://crates.io/crates/lumo)
[![docs.rs](https://img.shields.io/docsrs/lumo)](https://docs.rs/lumo)
[![Coverage](https://img.shields.io/coverallsCoverage/github/ekarpp/lumo)](https://coveralls.io/github/ekarpp/lumo)

Lumo is a CPU based multithreaded rendering engine. Made with the goal of learning Rust and physically based rendering :)

### Features
* Area light sampling
* Path tracing and bidirectional path tracing with [MIS](http://iliyan.com/publications/ImplementingVCM)
* [Cook-Torrance microfacet BSDF](https://doi.org/10.1145/357290.357293) with [Beckmann and GGX](http://dx.doi.org/10.2312/EGWR/EGSR07/195-206) normal distribution functions
* [Multiple importance sampling from VNDF for GGX](https://jcgt.org/published/0007/04/01/)
* .OBJ file parsing
* [Surface area hierarchy based kD-trees](https://www.irisa.fr/prive/kadi/Sujets_CTR/kadi/Kadi_sujet2_article_Kdtree.pdf)

### Usage
Once the repository is cloned, the `examples/` folder contains scenes. To run the `hello_sphere.rs` example execute the command:

```bash
cargo run --example hello_sphere
```

The renderer can be configured either through its setter methods in the examples or partially through the CLI:

```
Usage: hello_sphere [-s <samples>] [-t <threads>] [-d] [-b]

Optional CLI configuration of renderer. Renderer setter methods have priority.

Options:
  -s, --samples     number of samples per pixel (defaults to 1)
  -t, --threads     number of threads used (defaults to all)
  -d, --direct      use direct light integrator instead of path tracing.
  -b, --bdpt        use bidirectional path tracing instead of path tracing.
  --help            display usage information
```

#### Using the API
The `hello_sphere.rs` example is written as follows:

```rust
use glam::DVec3;
use lumo::tracer::*;
use lumo::*;

fn main() -> Result<(), png::EncodingError> {
    let camera = Camera::default(1280, 720);
    let mut scene = Scene::default();

    scene.add(Plane::new(
        DVec3::NEG_Y,
        DVec3::Y,
        Material::diffuse(Texture::Solid(srgb_to_linear(190, 200, 210))),
    ));

    scene.add_light(Sphere::new(
        8.0 * DVec3::Y + 1.5 * DVec3::NEG_Z,
        4.0,
        Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255))),
    ));

    scene.add(
        Sphere::new(
            DVec3::ZERO,
            1.0,
            Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 255))),
        )
        .scale(0.3, 0.3, 0.3)
        .translate(0.0, -0.7, -1.5),
    );

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_samples(36);
    renderer.render().save("hello.png")
}
```

### References
* [Physically Based Rendering](https://www.pbr-book.org/)
* [Ray Tracing in One Weekend](https://raytracing.github.io/)
* [Moving Frostbite to Physically Based Rendering](https://seblagarde.files.wordpress.com/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf)
* [Eric Veach's PhD Thesis](http://graphics.stanford.edu/papers/veach_thesis/)
* [ekhzang/rpt](https://github.com/ekzhang/rpt)

### Gallery
![Bust of Nefertiti](https://i.imgur.com/mF1qa0C.png)
![Cornell box](https://i.imgur.com/uM7eQe8.png)
![Stanford dragon](https://i.imgur.com/XE1OLp8.png)
![Circle of spheres](https://i.imgur.com/zraIbaH.png)
