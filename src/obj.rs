use crate::DVec3;
use std::fs::File;
use std::io::{self, Result, BufReader, BufRead};
use crate::tracer::{Material, Triangle};

/// Function to create io::Error
fn io_error(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// https://github.com/ekzhang/rpt/blob/master/src/io.rs
/// https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html
pub fn load_obj_file(file: File) -> Result<Vec<Triangle>> {
    let mut vertices: Vec<DVec3> = Vec::new();
    let mut normals: Vec<DVec3> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match tokens[0] {
            "v" => {
                let vertex = parse_vec3(&tokens)?;
                vertices.push(vertex);
            }
            "vn" => {
                let normal = parse_vec3(&tokens)?;
                normals.push(normal);
            }
            "f" => {
                let face = parse_face(&tokens, &vertices, &normals)?;
                triangles.extend(face);
            }
            _ => println!("skipping {} during .OBJ parsing", tokens[0]),
        }
    }

    println!("parsed .OBJ file with {} triangles", triangles.len());
    Ok(triangles)
}

fn parse_double(token: &str) -> Result<f64> {
    token.parse()
        .map_err(|_| io_error("could not parse double in .OBJ".to_string()))
}

fn parse_vec3(tokens: &[&str]) -> Result<DVec3> {
    Ok(DVec3::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
        parse_double(tokens[3])?,
    ))
}

fn parse_idx(token: &str, vec_len: usize) -> Result<usize> {
    token.parse::<i32>().map(|idx| {
        if idx > 0 {
            (idx - 1) as usize
        } else {
            (vec_len as i32 + idx) as usize
        }
    }).map_err(|_| io_error("could not parse index in .OBJ".to_string()))
}

fn parse_face(
    tokens: &[&str],
    vertices: &[DVec3],
    normals: &[DVec3]
) -> Result<Vec<Triangle>> {
    let mut vidxs: Vec<usize> = Vec::new();
    let mut nidxs: Vec<usize> = Vec::new();

    for token in &tokens[1..] {
        let arguments: Vec<&str> = token.split('/').collect();

        let vidx = parse_idx(arguments[0], vidxs.len())?;
        vidxs.push(vidx);
        if arguments.len() >= 3 {
            let nidx = parse_idx(arguments[2], nidxs.len())?;
            nidxs.push(nidx);
        }
    }

    let mut triangles: Vec<Triangle> = Vec::new();

    for i in 1..vidxs.len()-1 {
        let (a, b, c) = (0, i, i+1);
        let (va, vb, vc) = (vidxs[a], vidxs[b], vidxs[c]);
        let abc = (vertices[va], vertices[vb], vertices[vc]);

        if nidxs.is_empty() {
            triangles.push(*Triangle::new(abc, Material::Blank));
        } else {
            let (na, nb, nc) = (nidxs[a], nidxs[b], nidxs[c]);
            let nabc = (normals[na], normals[nb], normals[nc]);
            triangles.push(Triangle::new_w_normals(abc, nabc));
        }
    }

    Ok(triangles)
}
