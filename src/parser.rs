use crate::Image;
use crate::tracer::{Scene, Material, Texture, TriangleMesh, Face, Mesh};
use glam::{DVec2, DVec3};
use std::fs::File;
use std::sync::Arc;
use std::io::{self, BufRead, BufReader, Result};
use std::io::{Cursor, Read, Seek, Write};
use std::collections::HashMap;
use zip::ZipArchive;
use regex::Regex;
use mtl::MtlConfig;

/// .obj parser
mod obj;
/// .mtl parser
mod mtl;

/// Function to create io::Error
fn obj_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// For .obj and .mtl parsers
fn parse_double(token: &str) -> Result<f64> {
    token
        .parse()
        .map_err(|_| obj_error("Could not parse double in file"))
}

/// For .obj and .mtl parsers
fn parse_vec2(tokens: &[&str]) -> Result<DVec2> {
    Ok(DVec2::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
    ))
}

/// For .obj and .mtl parsers
fn parse_vec3(tokens: &[&str]) -> Result<DVec3> {
    Ok(DVec3::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
        parse_double(tokens[3])?,
    ))
}

/// For .obj and .mtl parsers
fn parse_idx(token: &str, vec_len: usize) -> Result<usize> {
    token
        .parse::<i32>()
        .map(|idx| {
            if idx > 0 {
                (idx - 1) as usize
            } else {
                (vec_len as i32 + idx) as usize
            }
        })
        .map_err(|_| obj_error("Could not parse index in file"))
}

/* these thigs below could be optimized alot...
 * but its boring work for little? gain */

fn _get_url(url: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();

    ureq::get(url)
        .call()
        .map_err(|_| obj_error("Error during HTTP, error parsing not implemented"))?
        .into_reader()
        .read_to_end(&mut bytes)?;

    Ok(bytes)
}

fn _extract_zip(bytes: Vec<u8>, re: Regex) -> Result<Vec<u8>> {
    println!("Reading .zip");
    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    let mut data = Vec::new();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        if re.is_match(file.name()) {
            println!("Extracting \"{}\"", file.name());
            file.read_to_end(&mut data)?;
            break;
        }
    }

    if data.is_empty() {
        Err(obj_error("Could not find file in the archive"))
    } else {
        Ok(data)
    }
}

fn _bytes_to_file(bytes: Vec<u8>) -> Result<File> {
    let mut tmp_file = tempfile::tempfile()?;

    tmp_file.write_all(&bytes)?;
    tmp_file.rewind()?;

    Ok(tmp_file)
}

/// Loads a .OBJ file at the given path
pub fn mesh_from_path(path: &str, material: Material) -> Result<Mesh> {
    println!("Loading .OBJ file \"{}\"", path);
    obj::load_file(File::open(path)?, material)
}

/// Loads .OBJ file from resource at an URL. Supports direct .OBJ files and
/// .OBJ files within a zip archive.
pub fn mesh_from_url(url: &str, material: Material) -> Result<Mesh> {
    println!("Loading .OBJ from \"{}\"", url);
    let mut bytes = _get_url(url)?;

    if url.ends_with(".zip") {
        println!("Found zip archive, searching for .OBJ files");
        bytes = _extract_zip(bytes, Regex::new(r".+\.obj$").unwrap())?;
    } else if !url.ends_with(".obj") {
        return Err(obj_error(
            "Bad URL, or at least does not end with .zip or .obj",
        ));
    }

    let obj_file = _bytes_to_file(bytes)?;
    obj::load_file(obj_file, material)
}

/// Loads `tex_name` from .zip at `url`
pub fn texture_from_url(url: &str, tex_name: &str) -> Result<Image> {
    if !tex_name.ends_with(".png") {
        return Err(obj_error("Can only load .png files"));
    }
    if !url.ends_with(".zip") {
        return Err(obj_error("Can only extract textures from zip archives"));
    }

    println!("Loading texture \"{}\" from \"{}\"", tex_name, url);

    let resp = _get_url(url)?;

    let file_bytes = _extract_zip(resp, Regex::new(tex_name).unwrap())?;

    let file = _bytes_to_file(file_bytes)?;

    println!("Decoding texture");
    Image::from_file(file)
        .map_err(|decode_error| obj_error(&decode_error.to_string()))
}

/// Parses a whole scene from a .obj file specified by `name`
/// in a .zip archive at `url`
#[allow(clippy::single_match)]
pub fn scene_from_url(url: &str, obj_name: &str) -> Result<Scene> {
    if !url.ends_with(".zip") {
        return Err(obj_error("Can only load scenes from .zip"));
    }
    if !obj_name.ends_with(".obj") {
        return Err(obj_error("Can only parse .obj files"));
    }

    println!("Loading scene \"{}\" from \"{}\"", obj_name, url);

    let resp = _get_url(url)?;

    let obj_bytes = _extract_zip(resp.clone(), Regex::new(obj_name).unwrap())?;

    let obj_file = _bytes_to_file(obj_bytes.clone())?;

    // parse materials first
    let mut materials = HashMap::<String, MtlConfig>::new();
    let reader = BufReader::new(obj_file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match tokens[0] {
            "mtllib" => {
                let mtllib_name = tokens[1];
                let mtl_bytes = _extract_zip(resp.clone(), Regex::new(mtllib_name).unwrap())?;
                let mtl_file = _bytes_to_file(mtl_bytes)?;

                mtl::load_file(mtl_file, &mut materials)?;
            }
            _ => (),
        }
    }

    let obj_file = _bytes_to_file(obj_bytes)?;

    obj::load_scene(obj_file, materials)
}
