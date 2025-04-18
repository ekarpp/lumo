use super::*;
use crate::tracer::KdTree;

/// https://github.com/ekzhang/rpt/blob/master/src/io.rs
/// https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html
pub fn load_file<T: Read + Sized>(file: T, material: Material) -> Result<Mesh> {
    let mut vertices: Vec<Point> = Vec::new();
    let mut normals: Vec<Normal> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        parse_tokens(tokens, &mut vertices, &mut normals, &mut uvs, &mut faces)?;
    }

    Ok(TriangleMesh::new(vertices, faces, normals, uvs, material))
}


pub fn load_scene<T: Read + Sized>(
    file: T,
    materials: Vec<Material>,
    material_indices: FxHashMap<String, usize>,
) -> Result<Scene> {
    let mut scene = Scene::default();
    let mut vertices: Vec<Point> = Vec::new();
    let mut normals: Vec<Normal> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut faces: Vec<Face> = Vec::new();
    let mut meshes: Vec<(Vec<Face>, usize)> = Vec::new();
    let mut midx = usize::MAX;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match tokens[0] {
            "g" | "o" => {
                if !faces.is_empty() {
                    meshes.push((faces, midx));
                    faces = Vec::new();
                    midx = usize::MAX;
                }
            }
            "usemtl" => {
                if !faces.is_empty() {
                    meshes.push((faces, midx));
                    faces = Vec::new();
                }
                match material_indices.get(tokens[1]) {
                    Some(idx) => midx = *idx,
                    None => {
                        return Err(obj_error(
                            &format!("Could not find material {}", tokens[1])
                        ));
                    }
                }
            }
            _ => {
                parse_tokens(
                    tokens,
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut faces
                )?
            }
        }
    }

    meshes.push((faces, midx));

    let mesh = Arc::new(TriangleMesh {
        vertices,
        normals,
        uvs,
        materials,
    });

    for (faces, midx) in meshes {
        let is_light = mesh.materials[midx].is_light();
        let triangles = TriangleMesh::triangles_from_faces(
            Arc::clone(&mesh),
            faces,
            midx,
        );

        if is_light {
            for triangle in triangles {
                scene.add_light(Box::new(triangle));
            }
        } else {
            scene.add(Box::new(KdTree::new(triangles)));
        }
    }

    Ok(scene)
}

fn parse_tokens(
    tokens: Vec<&str>,
    vertices: &mut Vec<Point>,
    normals: &mut Vec<Normal>,
    uvs: &mut Vec<Vec2>,
    faces: &mut Vec<Face>,
) -> Result<()> {
    match tokens[0] {
        "v" => {
            let vertex = parse_vec3(&tokens)?;
            vertices.push(vertex);
        }
        "vn" => {
            let normal = parse_vec3(&tokens)?;
            let normal = if normal.length_squared() == 0.0 {
                #[cfg(debug_assertions)]
                println!("Found degenerate normal in mesh");
                Normal::Z
            } else {
                normal.normalize()
            };
            normals.push(normal);
        }
        "vt" => {
            let uv = parse_vec2(&tokens)?;
            uvs.push(uv);
        }
        "f" => {
            let face = parse_face(&tokens, vertices, normals, uvs)?;
            faces.extend(face);
        }
        _ => (),
    }
    Ok(())
}

/// Parses a face from a .obj file
fn parse_face(
    tokens: &[&str],
    vertices: &[Point],
    normals: &[Normal],
    uvs: &[Vec2],
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
