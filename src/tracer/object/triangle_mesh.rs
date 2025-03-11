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
    /// Constructs a new face fron the indices. `nidx` and/or `tidx` may be empty
    pub fn new(vidx: Vec<usize>, nidx: Vec<usize>, tidx: Vec<usize>) -> Self {
        Self {
            vidx,
            nidx,
            tidx,
        }
    }
}

/// Mesh of triangles accelerated with a kdtree
pub struct TriangleMesh {
    /// All vertices of the mesh
    pub vertices: Vec<Point>,
    /// All shading normals of the mesh
    pub normals: Vec<Normal>,
    /// All texture coordinates of the mesh
    pub uvs: Vec<Vec2>,
    /// All materials used in the mesh
    pub materials: Vec<Material>,
}

impl TriangleMesh {
    /// Constructs a mesh, i.e. kdtree, from the given data.
    /// `normals` and/or `uvs` may be empty.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        vertices: Vec<Point>,
        faces: Vec<Face>,
        normals: Vec<Normal>,
        uvs: Vec<Vec2>,
        material: Material,
    ) -> Mesh {
        let mesh = Arc::new(Self {
            vertices,
            normals,
            uvs,
            materials: vec!(material),
        });

        KdTree::new(Self::triangles_from_faces(mesh, faces, 0))
    }

    /// Constructs the triangles defined by `faces` using material with `midx`
    pub fn triangles_from_faces(mesh: Arc<Self>, faces: Vec<Face>, midx: usize) -> Vec<Triangle> {
        let mut triangles = Vec::with_capacity(faces.len());

        for face in faces {
            for i in 1..face.vidx.len() - 1 {
                let (a, b, c) = (0, i, i + 1);
                let vidx = (face.vidx[a], face.vidx[b], face.vidx[c]);

                if Self::degenerate_triangle(
                    mesh.vertices[vidx.0],
                    mesh.vertices[vidx.1],
                    mesh.vertices[vidx.2],
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

                triangles.push(Triangle::new(Arc::clone(&mesh), vidx, midx, nidx, tidx));
            }
        }

        triangles
    }

    fn degenerate_triangle(a: Point, b: Point, c: Point) -> bool {
        let ng = (b - a).cross(c - a);
        ng.length() == 0.0
    }
}
