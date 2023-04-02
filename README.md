## Lumo

[![crates.io](https://img.shields.io/crates/v/lumo)](https://crates.io/crates/lumo)
[![docs.rs](https://img.shields.io/docsrs/lumo)](https://docs.rs/lumo)


Lumo is a CPU based multithreaded rendering engine. Made with the goal of learning Rust and physically based rendering :) The renderer is designed to be as modular as possible such that adding new features or algorithms is straightforward.

### Features
* Area light sampling
* Path tracing with next event estimation
* Microfacet BSDF with Beckmann and GGX normal distribution functions
* Multiple importance sampling from VNDF for GGX
* Disney diffuse BRDF with energy normalization used in Frostbite
* Vertex and normal parsing from .OBJ files
* Stratified sampling
* kd-tree with SAH
* Perlin noise generator

### Usage
Once the repository is cloned, the `examples/` folder contains scenes. To run the `hello_sphere.rs` example execute the command:

```bash
cargo run --example hello_sphere
```

The renderer can be configured either through its setter methods or partially through the CLI:

```
Usage: hello_sphere [-s <samples>] [-t <threads>] [-w <width>] [-h <height>] [-d]

Optional CLI configuration of renderer. Renderer setter methods have priority.

Options:
  -s, --samples     number of samples per pixel (defaults to 1)
  -t, --threads     number of threads used (defaults to all)
  -w, --width       width of the rendered image (defaults to 1000)
  -h, --height      height of the rendered image (defaults to 1000)
  -d, --direct      use direct light integrator instead of path tracing.
  --help            display usage information
```

#### Using the API

The `hello_sphere.rs` example is written as follows:

```rust
use lumo::*;
use lumo::tracer::*;
use glam::DVec3;

fn main() -> Result<(), png::EncodingError> {
    let camera = Camera::default();
    let mut scene = Scene::default();

    scene.add(
        Plane::new(
            DVec3::NEG_Y,
            DVec3::Y,
            Material::diffuse(Texture::Solid(srgb_to_linear(190, 200, 210)))
        )
    );

    scene.add(
        Sphere::new(
            8.0 * DVec3::Y + 1.5 * DVec3::NEG_Z,
            4.0,
            Material::Light(Texture::Solid(srgb_to_linear(255, 255, 255)))
        )
    );

    scene.add(
        Sphere::new(
            DVec3::ZERO,
            1.0,
            Material::diffuse(Texture::Solid(srgb_to_linear(0, 0, 255)))
        )
            .scale(0.3, 0.3, 0.3)
            .translate(0.0, -0.7, -1.5)
    );

    let mut renderer = Renderer::new(scene, camera);
    renderer.set_samples(36);
    renderer.render()
        .save("hello.png")
}
```

### TODO/WiP
* Handling exposed backfaces in .OBJ models
* Firefly reduction (better sampling? tone mapping?)
* Isotropic mediums (fog, smoke, clouds, ...)
* Multiple importance sampling in path tracer
* Parallelize kd-tree construction
* kd-tree construction in $n \log n$
* (Texture mapping)
* (Bidirectional path tracing)
* (Subsurface scattering)

### References
* [Physically Based Rendering](https://www.pbr-book.org/)
* [Ray Tracing in One Weekend](https://raytracing.github.io/)
* [Moving Frostbite to Physically Based Rendering](https://seblagarde.files.wordpress.com/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf)
* [Eric Veach's PhD Thesis](http://graphics.stanford.edu/papers/veach_thesis/)
* [ekhzang/rpt](https://github.com/ekzhang/rpt)

### Gallery

| ![Stanford dragon](https://i.imgur.com/Bx0HvsA.png) |
|:--:|
| *Stanford dragon with 871K triangles. Rendered in 13 minutes using 40 threads of Intel Xeon Gold 6248. Hable tone mapping and 2025 samples per pixel.* |

| ![Cornell box](https://i.imgur.com/TG4IjYo.png) |
|:--:|
| *Cornell box displaying reflection and refraction. Rendered in 12 minutes using 40 threads of Intel Xeon Gold 6248. 4096 samples per pixel.* |

| ![Golden Nefertiti](https://i.imgur.com/ze7OwaF.png) |
|:--:|
| *Statue of Nefertiti with 6.4M triangles. Rendered in 7 minutes using 40 threads of Intel Xeon Gold 6248. 1024 samples per pixel.* |

![Circle of spheres](https://i.imgur.com/3FnSev8.png)
