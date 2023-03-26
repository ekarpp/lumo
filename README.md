A CPU based multithreaded rendering engine. Made with the goal of learning Rust and physically based rendering :) The renderer is designed to be as modular as possible such that adding new features and algorithms is straightforward.

### Features
* Area light sampling
* Path tracing with next event estimation
* Microfacet BSDF with multiple importance sampling and GGX distribution
* Vertex and normal parsing from .OBJ files
* kd-tree that handles meshes containing up to 5 million triangles with reasonable render times
* Tone mapping
* Perlin noise generator

### TODO/WIP
* Refraction of transparent microfacet materials
* Isotropic mediums (fog, smoke, clouds, ...)
* Multiple importance sampling in path tracer
* Sampling from distribution of visible normals in microfacets
* (Bidirectional path tracing)
* (Subsurface scattering)

### References
* [Physically Based Rendering](https://www.pbr-book.org/)
* [Ray Tracing in One Weekend](https://raytracing.github.io/)
* [Eric Veach's Ph.D Thesis](http://graphics.stanford.edu/papers/veach_thesis/)
* [ekhzang/rpt](https://github.com/ekzhang/rpt)

![Stanford dragon with 871K triangles. Rendered in 45 minutes using 30 Intel Xeon Gold 6248 threads. 1024 samples per pixel.](https://i.imgur.com/zREVJF3.png)
![Circle of spheres](https://i.imgur.com/3FnSev8.png)
![Statue of Nefertiti with 6.4M triangles. Rendered in 196 minutes using 40 Intel Xeon Gold 6248 threads. 1024 samples per pixel.](https://i.imgur.com/MNgV9xa.png)
