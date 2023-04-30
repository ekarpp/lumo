use super::*;

/// A single face of a polygon mesh
pub struct Face {
    /// Indices to vertices
    pub vidx: Vec<usize>,
    /// Indices to shading normals, can be empty
    pub nidx: Vec<usize>,
    /// Indices to texture coordinates, can be empty
    pub tidx: Vec<usize>,
}

impl Face {
    pub fn new(vidx: Vec<usize>, nidx: Vec<usize>, tidx: Vec<usize>) -> Self {
        Self {
            vidx,
            nidx,
            tidx,
        }
    }
}

/// Mesh of triangles accelerated with a kdtree
pub struct TriangleMesh<'a> {
    /// Acceleration structure
    kdtree: KdTree<Triangle<'a>>,
    /// All vertices of the mesh
    pub vertices: Vec<DVec3>,
    /// All shading normals of the mesh
    pub normals: Vec<DVec3>,
    /// All texture coordinates of the mesh
    pub uvs: Vec<DVec2>,
}

impl TriangleMesh<'_> {
    pub fn new(
        vertices: Vec<DVec3>,
        faces: Vec<Face>,
        normals: Vec<DVec3>,
        uvs: Vec<DVec2>,
        material: Material,
    ) -> Box<Self> {
        let mut mesh = Box::new(Self {
            kdtree: KdTree::new(Vec::new(), Material::Blank),
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        });

        let mut triangles = Vec::with_capacity(faces.len());

        for face in faces {
            for i in 1..face.vidx.len() - 1 {
                let (a, b, c) = (0, i, i + 1);
                let vidx = (face.vidx[a], face.vidx[b], face.vidx[c]);

                if Self::degenerate_triangle(
                    vertices[vidx.0],
                    vertices[vidx.1],
                    vertices[vidx.2],
                ) {
                    continue;
                }

                let nidx = if face.nidx.is_empty() {
                    None
                } else {
                    Some((face.nidx[a], face.nidx[b], face.nidx[c]))
                };

                let tidx = if face.tidx.is_empty() {
                    None
                } else {
                    Some((face.tidx[a], face.tidx[b], face.tidx[c]))
                };

                triangles.push(Triangle::new(&mesh, vidx, nidx, tidx));
            }
        }

        mesh.kdtree = KdTree::new(triangles, material);
        mesh.vertices = vertices;
        mesh.normals = normals;
        mesh.uvs = uvs;

        mesh
    }

    fn degenerate_triangle(a: DVec3, b: DVec3, c: DVec3) -> bool {
        let ng = (b - a).cross(c - a);
        ng.length() == 0.0
    }
}

impl Object for TriangleMesh<'_> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        self.kdtree.hit(r, t_min, t_max)
    }
}

impl Bounded for TriangleMesh<'_> {
    fn bounding_box(&self) -> AaBoundingBox { self.kdtree.bounding_box() }
}

impl Sampleable for TriangleMesh<'_> {
    fn area(&self) -> f64 {
        self.kdtree.area()
    }

    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        self.kdtree.sample_on(rand_sq)
    }
}
