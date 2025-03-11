use crate::{Vec2, Vec3, Image, Float, Normal, Point};
use crate::tracer::{
    Scene, Material, Texture,
    TriangleMesh, Face, Mesh, Spectrum
};
use std::fs::{ self, File };
use std::sync::Arc;
use std::io::{
    self, BufRead, BufReader, Result,
    Cursor, Read, Write,
};
use rustc_hash::FxHashMap;
use zip::ZipArchive;

/// .obj parser
mod obj;
/// .mtl parser
mod mtl;

const SCENE_DIR: &str = "./scenes/";

/*
 * BEWARE WHO ENTERS! HERE BE DRAGONS!
 */

/// Function to create io::Error
fn obj_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// For .obj and .mtl parsers
fn parse_double(token: &str) -> Result<Float> {
    token
        .parse()
        .map_err(|_| obj_error("Could not parse double in file"))
}

/// For .obj and .mtl parsers
fn parse_vec2(tokens: &[&str]) -> Result<Vec2> {
    Ok(Vec2::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
    ))
}

/// For .obj and .mtl parsers
fn parse_vec3(tokens: &[&str]) -> Result<Vec3> {
    Ok(Vec3::new(
        parse_double(tokens[1])?,
        parse_double(tokens[2])?,
        parse_double(tokens[3])?,
    ))
}

/// For .obj and .mtl parsers
fn parse_idx(token: &str, vec_len: usize) -> Result<usize> {
    token
        .parse::<i64>()
        .map(|idx| {
            if idx > 0 {
                (idx - 1) as usize
            } else {
                (vec_len as i64 + idx) as usize
            }
        })
        .map_err(|_| obj_error("Could not parse index in file"))
}

/* these thigs below could be optimized alot...
 * but its boring work for little? gain */

fn _get_url(url: &str, bytes: &mut Vec<u8>) -> Result<()> {
    let mut curl = curl::easy::Easy::new();
    curl.url(url)
        .map_err(|e| obj_error(e.description()))?;

    let mut transfer = curl.transfer();
    transfer.write_function(|data| { bytes.extend_from_slice(data); Ok(data.len()) })
        .map_err(|e| obj_error(e.description()))?;

    transfer.perform()
        .map_err(|e| obj_error(e.description()))?;

    Ok(())
}

/// Extracts file matching `re` from zip file in `bytes`
fn _extract_zip(bytes: &[u8], end_match: &str) -> Result<Vec<u8>> {
    println!("Reading .zip");
    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    let mut data = Vec::new();
    let end_match = end_match.to_lowercase();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        if file.name().to_lowercase().ends_with(&end_match) {
            if data.is_empty() {
                println!("Extracting \"{}\"", file.name());
                file.read_to_end(&mut data)?;
            } else {
                return Err(
                    obj_error(&format!("Found multiple {} in the archive", end_match))
                );
            }
        }
    }

    if data.is_empty() {
        Err(obj_error(&format!("Could not find {} in the archive", end_match)))
    } else {
        Ok(data)
    }
}

/// Loads `tex_name` from `zip` to an `Image`
fn _img_from_zip(zip: &[u8], tex_name: &str) -> Result<Image<Spectrum>> {
    let file_bytes = _extract_zip(zip, tex_name)?;
    println!("Decoding texture");
    Image::from_file(file_bytes.as_slice())
        .map_err(|decode_error| obj_error(&decode_error.to_string()))
}

/// Loads a .OBJ file at the given path
pub fn mesh_from_path(path: &str, material: Material) -> Result<Mesh> {
    println!("Loading .OBJ file \"{}\"", path);
    obj::load_file(File::open(path)?, material)
}

/// Loads .OBJ file from resource at an URL. Supports direct .OBJ files and
/// .OBJ files within a zip archive.
pub fn mesh_from_url(url: &str, material: Material) -> Result<Mesh> {
    let path = _check_cached(url)?;
    println!("Loading .OBJ from \"{}\"", &path);
    let mut bytes = fs::read(path)?;

    if url.ends_with(".zip") {
        println!("Found zip archive, searching for .OBJ files");
        bytes = _extract_zip(&bytes, ".obj")?;
    } else if !url.ends_with(".obj") {
        return Err(obj_error(
            "Bad URL, or at least does not end with .zip or .obj",
        ));
    }

    obj::load_file(bytes.as_slice(), material)
}

fn _check_cached(url: &str) -> Result<String> {
    if !fs::exists(SCENE_DIR)? {
        fs::create_dir(SCENE_DIR)?;
    }
    let fname = url.rsplit('/').next()
        .ok_or(obj_error("couldn't regex parse url"))?;
    let path = format!("{}{}", SCENE_DIR, fname);
    if !fs::exists(&path)? {
        println!("\"{}\" not found, downloading from \"{}\"", path, url);
        let mut resp = Vec::new();
        _get_url(url, &mut resp)?;
        let mut file = fs::File::create(&path)?;
        file.write_all(&resp)?;
    }

    Ok(path)
}

/// Loads `tex_name` from .zip at `url`. zip cached to `SCENE_DIR`
pub fn texture_from_url(url: &str, tex_name: &str) -> Result<Image<Spectrum>> {
    if !tex_name.ends_with(".png") {
        return Err(obj_error("Can only load .png files"));
    }
    if !url.ends_with(".zip") {
        return Err(obj_error("Can only extract textures from zip archives"));
    }

    let path = _check_cached(url)?;
    println!("Loading texture \"{}\" from \"{}\"", tex_name, path);

    _img_from_zip(&fs::read(path)?, tex_name)
}

/// Parses a whole scene from a .obj file specified by `name`
/// in a .zip archive at `url`. Cache `url` to `SCENE_DIR`
pub fn scene_from_url(
    url: &str,
    obj_name: &str,
    map_ks: bool,
    mtllib: Option<&str>,
    env_map: Option<(&str, Float)>
) -> Result<Scene> {
    if !url.ends_with(".zip") {
        return Err(obj_error("Can only load scenes from .zip"));
    }
    if !obj_name.ends_with(".obj") {
        return Err(obj_error("Can only parse .obj files"));
    }

    let path = _check_cached(url)?;

    scene_from_file(&path, obj_name, map_ks, mtllib, env_map)
}

/// Load a scene from zip file at `pth`
pub fn scene_from_file(
    path: &str,
    obj_name: &str,
    map_ks: bool,
    mtllib: Option<&str>,
    env_map: Option<(&str, Float)>
) -> Result<Scene> {
    println!("Loading scene \"{}\" from \"{}\"", obj_name, path);
    let f = fs::read(path)?;
    let obj_bytes = _extract_zip(&f, obj_name)?;

    // parse materials first
    let mut materials = Vec::new();
    let mut material_indices = FxHashMap::<String, usize>::default();

    if let Some(mtllib_name) = mtllib {
        let mtl_bytes = _extract_zip(&f, mtllib_name)?;
        mtl::load_file(
            mtl_bytes.as_slice(),
            map_ks,
            Some(&f),
            &mut materials,
            &mut material_indices,
        )?;
    }

    let reader = BufReader::new(obj_bytes.as_slice());
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        if tokens[0] == "mtllib" {
            let mtllib_name = tokens[1];
            let mtl_bytes = _extract_zip(&f, mtllib_name)?;
            mtl::load_file(
                mtl_bytes.as_slice(),
                map_ks,
                Some(&f),
                &mut materials,
                &mut material_indices,
            )?;
        }
    }

    let mut scene = obj::load_scene(
        obj_bytes.as_slice(),
        materials,
        material_indices,
    )?;

    if let Some((map_file, scale)) = env_map {
        let map_bytes = _extract_zip(&f, map_file)?;
        scene.set_environment_map(
            Texture::Image(Image::from_hdri_bytes(map_bytes.as_slice())?), scale,
        );
    }

    Ok(scene)
}
