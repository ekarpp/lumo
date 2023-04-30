use super::*;

/// https://github.com/ekzhang/rpt/blob/master/src/io.rs
/// https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html
pub fn load_file(file: File, material: Material) -> Result<Box<TriangleMesh<'static>>> {
    let mut vertices: Vec<DVec3> = Vec::new();
    let mut normals: Vec<DVec3> = Vec::new();
    let mut uvs: Vec<DVec2> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match tokens[0] {
            "g" => {
                /*
                if !triangles.is_empty() {
                    meshes.push(triangles);
                    triangles = Vec::new();
                }
                 */
            }
            "v" => {
                let vertex = parse_vec3(&tokens)?;
                vertices.push(vertex);
            }
            "vn" => {
                let normal = parse_vec3(&tokens)?;
                normals.push(normal);
            }
            "vt" => {
                let uv = parse_vec2(&tokens)?;
                uvs.push(uv);
            }
            "f" => {
                let face = parse_face(&tokens, &vertices, &normals, &uvs)?;
                faces.extend(face);
            }
            _ => (),
        }
    }

    Ok(TriangleMesh::new(vertices, faces, normals, uvs, material))
}

/// Parses a face from a .obj file
fn parse_face(
    tokens: &[&str],
    vertices: &[DVec3],
    normals: &[DVec3],
    uvs: &[DVec2],
) -> Result<Vec<Face>> {
    let mut vidxs: Vec<usize> = Vec::new();
    let mut nidxs: Vec<usize> = Vec::new();
    let mut tidxs: Vec<usize> = Vec::new();

    for token in &tokens[1..] {
        let arguments: Vec<&str> = token.split('/').collect();

        let vidx = parse_idx(arguments[0], vertices.len())?;
        vidxs.push(vidx);

        if arguments.len() > 1 && !arguments[1].is_empty() {
            let tidx = parse_idx(arguments[1], uvs.len())?;
            tidxs.push(tidx);
        }

        if arguments.len() > 2 {
            let nidx = parse_idx(arguments[2], normals.len())?;
            nidxs.push(nidx);
        }
    }

    let mut faces: Vec<Face> = Vec::new();

    for i in 1..vidxs.len() - 1 {
        let (a, b, c) = (0, i, i + 1);
        let vidx = vec![vidxs[a], vidxs[b], vidxs[c]];

        let nidx = if nidxs.is_empty() {
            Vec::new()
        } else {
            vec![nidxs[a], nidxs[b], nidxs[c]]
        };

        let tidx = if tidxs.is_empty() {
            Vec::new()
        } else {
            vec![tidxs[a], tidxs[b], tidxs[c]]
        };

        faces.push(Face::new(vidx, nidx, tidx));
    }

    Ok(faces)
}
