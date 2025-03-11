use super::*;
use crate::tracer::{RGB, Spectrum};

/// Holds the properties of a microfacet material
pub struct MtlConfig {
    /// Diffuse color of the material
    pub Kd: Spectrum,
    /// Texture map
    pub map_Kd: Option<Image<Spectrum>>,
    /// Specular color of the material
    pub Ks: Spectrum,
    /// Specular texture map
    pub map_Ks: Option<Image<Spectrum>>,
    /// Emittance of the material. If not zero vector, then createas a light
    pub Ke: Spectrum,
    /// Emission map
    pub map_Ke: Option<Image<Spectrum>>,
    /// Bump map for the material
    pub map_Bump: Option<Image<Normal>>,
    /// How much each light channel passes on transmission
    pub Tf: Spectrum,
    /// Refraction index of the material
    pub eta: Float,
    /// Roughness of the material
    pub roughness: Float,
    /// Absorption coefficient
    pub k: Float,
    /// Is fresnel enabled for the material a.k.a. is it a conductor/dielectric
    pub fresnel_enabled: bool,
    /// Is the material dielectric?
    pub is_transparent: bool,
}

impl Default for MtlConfig {
    fn default() -> Self {
        Self {
            Kd: Spectrum::BLACK,
            Ks: Spectrum::BLACK,
            Ke: Spectrum::BLACK,
            Tf: Spectrum::BLACK,
            eta: 1.5,
            k: 0.0,
            roughness: 1.0,
            fresnel_enabled: false,
            is_transparent: false,
            map_Kd: None,
            map_Ks: None,
            map_Ke: None,
            map_Bump: None,
        }
    }
}

impl MtlConfig {
    pub fn build_material(self) -> Material {
        if !self.Ke.is_black() || self.map_Ke.is_some() {
            if let Some(img) = self.map_Ke {
                Material::light(Texture::Image(img))
            } else {
                Material::light(Texture::from(self.Ke))
            }
        } else {
            let kd = if let Some(img) = self.map_Kd {
                Texture::Image(img)
            } else {
                Texture::from(self.Kd)
            };
            let ks = if let Some(img) = self.map_Ks {
                Texture::Image(img)
            } else {
                Texture::from(self.Ks)
            };
            let tf = Texture::from(self.Tf);

            Material::microfacet(
                self.roughness,
                self.eta,
                self.k,
                self.is_transparent,
                self.fresnel_enabled,
                kd, ks, tf,
                self.map_Bump,
            )
        }
    }
}

pub fn load_file<T: Read + Sized>(
    bytes: T,
    map_ks: bool,
    zip_file: Option<&Vec<u8>>,
    materials: &mut Vec<Material>,
    material_indices: &mut FxHashMap<String, usize>,
) -> Result<()> {
    let reader = BufReader::new(bytes);

    let mut mtl = MtlConfig::default();
    let mut mtl_name = String::default();

    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match tokens[0] {
            "newmtl" => {
                if !mtl_name.is_empty() {
                    materials.push(mtl.build_material());
                    material_indices.insert(mtl_name, materials.len() - 1);
                }
                mtl = MtlConfig::default();
                mtl_name = tokens[1].to_string();
                if material_indices.contains_key(&mtl_name) {
                    mtl_name.clear();
                }
            }
            /* diffuse color */
            "Kd" => {
                let kd = parse_vec3(&tokens)?;
                mtl.Kd = Spectrum::from_rgb(RGB::from(kd));
            }
            /* texture map */
            "map_Kd" => {
                if let Some(zip) = zip_file {
                    let tex_name = tokens[1..].join(" ").replace('\\', "/");
                    let img = super::_img_from_zip(zip, &tex_name)?;
                    mtl.map_Kd = Some(img);
                }
            }
            /* emission color */
            "Ke" => {
                let ke = parse_vec3(&tokens)?;
                mtl.Ke = Spectrum::from_rgb(RGB::from(ke));
            }
            "map_Ke" => {
                if let Some(zip) = zip_file {
                    let tex_name = tokens[1..].join(" ").replace('\\', "/");
                    let img = super::_img_from_zip(zip, &tex_name)?;
                    mtl.map_Ke = Some(img);
                }
            }
            /* specular color */
            "Ks" => {
                let ks = parse_vec3(&tokens)?;
                mtl.Ks = Spectrum::from_rgb(RGB::from(ks));
            }
            "map_Ks" => {
                if let Some(zip) = zip_file {
                    let tex_name = tokens[1..].join(" ").replace('\\', "/");
                    let bytes = super::_extract_zip(zip, &tex_name)?;
                    if !map_ks {
                        // occlusion, roughness, metalness
                        let orm = Image::<Vec3>::mean_vec3_from_file(bytes.as_slice())?;
                        mtl.roughness = orm.y;
                        mtl.k = orm.z;
                        mtl.Ks = Spectrum::WHITE;
                    } else {
                        let img = super::_img_from_zip(zip, &tex_name)?;
                        mtl.map_Ks = Some(img);
                    }
                }
            }
            /* bump map */
            "map_Bump" => {
                if let Some(zip) = zip_file {
                    let map_name = tokens[1..].join(" ").replace('\\', "/");
                    let bytes = super::_extract_zip(zip, &map_name)?;
                    mtl.map_Bump = Some(Image::bump_from_file(bytes.as_slice())?);
                }
            }
            /* transmission filter */
            "Tf" => {
                let tf = parse_vec3(&tokens)?;
                mtl.Tf = Spectrum::from_rgb(RGB::from(tf));
            }
            /* refraction index */
            "Ni" => {
                let ni = parse_double(tokens[1])?;
                mtl.eta = ni;
            }
            /* roughness */
            "Ns" => {
                let ns = parse_double(tokens[1])?;
                // blender uses this mapping
                mtl.roughness = 1.0 - ns.min(900.0).sqrt() / 30.0;
            }
            /* illumination model */
            "illum" => {
                let illum = parse_double(tokens[1])? as usize;
                match illum {
                    5 => mtl.fresnel_enabled = true,
                    6 => mtl.is_transparent = true,
                    7 => { mtl.fresnel_enabled = true; mtl.is_transparent = true; },
                    _ => (),
                }
            }
            _ => (),
        }
    }

    if !mtl_name.is_empty() {
        materials.push(mtl.build_material());
        material_indices.insert(mtl_name, materials.len() - 1);
    }

    Ok(())
}
