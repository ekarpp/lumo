use super::*;

/// https://github.com/ekzhang/rpt/blob/master/src/io.rs
/// https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html
pub fn load_file(file: File) -> Result<Vec<Vec<Triangle>>> {
    let mut vertices: Vec<DVec3> = Vec::new();
    let mut normals: Vec<DVec3> = Vec::new();
    let mut uvs: Vec<DVec2> = Vec::new();
    let mut meshes: Vec<Vec<Triangle>> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match tokens[0] {
            "g" => {
                if !triangles.is_empty() {
                    meshes.push(triangles);
                    triangles = Vec::new();
                }
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
                triangles.extend(face);
            }
            _ => (),
        }
    }
    meshes.push(triangles);
    println!("Parsed .OBJ file with {} meshes and a total of {} triangles",
             meshes.len(),
             meshes.iter().fold(0, |sum, mesh| sum + mesh.len())
    );
    Ok(meshes)
}

/// Some .objs have degenerate triangles. This filters them out.
fn degenerate_triangle(abc: DMat3) -> bool {
    let a = abc.col(0); let b = abc.col(1); let c = abc.col(2);
    let ng = (b - a).cross(c - a);
    ng.length() == 0.0
}

/// Some .objs have zero vector normals. This fixes them to geometric normal.
fn fixed_normals(abc: DMat3, na: DVec3, nb: DVec3, nc: DVec3) -> DMat3 {
    let a = abc.col(0); let b = abc.col(1); let c = abc.col(2);
    // cant be degenerate at this point
    let ng = (b - a).cross(c - a);
    let ng = ng.normalize();

    DMat3::from_cols(
        if na.length() == 0.0 { ng } else { na },
        if nb.length() == 0.0 { ng } else { nb },
        if nc.length() == 0.0 { ng } else { nc },
    )
}

/// Parses a face from a .obj file
fn parse_face(
    tokens: &[&str],
    vertices: &[DVec3],
    normals: &[DVec3],
    uvs: &[DVec2],
) -> Result<Vec<Triangle>> {
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

    let mut triangles: Vec<Triangle> = Vec::new();

    for i in 1..vidxs.len() - 1 {
        let (a, b, c) = (0, i, i + 1);
        let (va, vb, vc) = (vidxs[a], vidxs[b], vidxs[c]);
        let abc = DMat3::from_cols(vertices[va], vertices[vb], vertices[vc]);

        if degenerate_triangle(abc) {
            continue;
        }

        let nabc = if nidxs.is_empty() {
            None
        } else {
            let (na, nb, nc) = (nidxs[a], nidxs[b], nidxs[c]);
            Some(fixed_normals(abc, normals[na], normals[nb], normals[nc]))
        };

        let tabc = if tidxs.is_empty() {
            None
        } else {
            let (ta, tb, tc) = (tidxs[a], tidxs[b], tidxs[c]);
            Some(DMat3::from_cols(
                uvs[ta].extend(0.0),
                uvs[tb].extend(0.0),
                uvs[tc].extend(0.0),
            ))
        };

        triangles.push(*Triangle::new(abc, nabc, tabc, Material::Blank));
    }

    Ok(triangles)
}
