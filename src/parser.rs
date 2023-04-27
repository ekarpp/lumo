use crate::Image;
use crate::tracer::{Material, Texture, Triangle};
use glam::{DMat3, DVec2, DVec3};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Result};
use std::io::{Cursor, Read, Seek, Write};
use std::collections::HashMap;
use zip::ZipArchive;

/// .obj parser
mod obj;
/// .mtl parser
mod mtl;

/// Function to create io::Error
fn obj_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn parse_double(token: &str) -> Result<f64> {
    token
        .parse()
        .map_err(|_| obj_error("Could not parse double in file"))
}

fn parse_vec2(tokens: &[&str]) -> Result<DVec2> {
    Ok(DVec2::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
    ))
}

fn parse_vec3(tokens: &[&str]) -> Result<DVec3> {
    Ok(DVec3::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
        parse_double(tokens[3])?,
    ))
}

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

/// Loads a .OBJ file at the given path
pub fn obj_from_path(path: &str) -> Result<Vec<Vec<Triangle>>> {
    println!("Loading .OBJ file \"{}\"", path);
    obj::load_file(File::open(path)?)
}

/// Loads `tex_name` from .zip at `url`
pub fn texture_from_url(url: &str, tex_name: &str) -> Result<Image> {
    println!("Loading texture \"{}\" from \"{}\"", tex_name, url);
    if !tex_name.ends_with(".png") {
        return Err(obj_error("Can only load .png files"));
    }

    if !url.ends_with(".zip") {
        return Err(obj_error("Can only extract textures from zip archives"));
    }

    let mut bytes = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|_| obj_error("Error during HTTP, error parsing not implemented"))?
        .into_reader()
        .read_to_end(&mut bytes)?;

    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    bytes = Vec::new();
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        if file.name().eq(tex_name) {
            println!("Extracting \"{}\"", file.name());
            file.read_to_end(&mut bytes)?;
            break;
        }
    }
    if bytes.is_empty() {
        return Err(obj_error("Could not find texture in the archive"));
    }

    let mut tmp_file = tempfile::tempfile()?;
    tmp_file.write_all(&bytes)?;
    tmp_file.rewind()?;

    println!("Decoding texture");
    Image::from_file(tmp_file)
        .map_err(|decode_error| obj_error(&decode_error.to_string()))
}

/// Loads .OBJ file from resource at an URL. Supports direct .OBJ files and
/// .OBJ files within a zip archive.
pub fn obj_from_url(url: &str) -> Result<Vec<Vec<Triangle>>> {
    println!("Loading .OBJ from \"{}\"", url);
    let mut bytes = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|_| obj_error("Error during HTTP, error parsing not implemented"))?
        .into_reader()
        .read_to_end(&mut bytes)?;

    if url.ends_with(".zip") {
        println!("Found zip archive, searching for .OBJ files");
        let mut zip = ZipArchive::new(Cursor::new(bytes))?;
        bytes = Vec::new();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;

            if file.name().ends_with("obj") {
                println!("Extracting \"{}\"", file.name());
                file.read_to_end(&mut bytes)?;
                break;
            }
        }
        if bytes.is_empty() {
            return Err(obj_error("Could not find .OBJ files in the archive"));
        }
    } else if !url.ends_with(".obj") {
        return Err(obj_error(
            "Bad URL, or at least does not end with .zip or .obj",
        ));
    }

    let mut tmp_file = tempfile::tempfile()?;
    tmp_file.write_all(&bytes)?;
    tmp_file.rewind()?;
    obj::load_file(tmp_file)
}