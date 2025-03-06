use super::*;
use crate::tracer::Spectrum;

/// Holds the properties of a microfacet material
#[allow(non_snake_case)]
pub struct MtlConfig {
    /// Diffuse color of the material
    pub Kd: Spectrum,
    /// Specular color of the material
    pub Ks: Spectrum,
    /// Emittance of the material. If not zero vector, then createas a light
    pub Ke: Spectrum,
    /// How much each light channel passes on transmission
    pub Tf: Spectrum,
    /// Refraction index of the material
    pub Ni: Float,
    /// Roughness of the material
    pub Ns: Float,
    /// Illumination model, see docs.
    /// If 6 or 7 makes transparent, if 5 makes metal, otherwise unused.
    pub illum: usize,
    /// Texture map
    pub map_Kd: Option<Image>,
}

impl Default for MtlConfig {
    fn default() -> Self {
        Self {
            Kd: Spectrum::BLACK,
            Ks: Spectrum::BLACK,
            Ke: Spectrum::BLACK,
            Tf: Spectrum::BLACK,
            Ni: 1.5,
            Ns: 0.0,
            illum: 0,
            map_Kd: None,
        }
    }
}

impl MtlConfig {
    pub fn build_material(&self) -> Material {
        if !self.Ke.is_black() {
            Material::Light(Texture::from(self.Ke.clone()))
        } else {
            let fresnel_enabled = self.illum == 5 || self.illum == 7;
            let is_transparent = self.illum == 6 || self.illum == 7;

            let kd = if let Some(img) = &self.map_Kd {
                Texture::Image(img.clone())
            } else {
                Texture::from(self.Kd.clone())
            };
            let ks = Texture::from(self.Ks.clone());
            let tf = Texture::from(self.Tf.clone());

            // blender uses this mapping
            let roughness = 1.0 - self.Ns.min(900.0).sqrt() / 30.0;

            Material::microfacet(
                roughness,
                self.Ni,
                0.0,
                is_transparent,
                fresnel_enabled,
                kd, ks, tf
            )
        }
    }
}

pub fn load_file(
    file: File,
    zip_file: Option<Vec<u8>>,
    materials: &mut FxHashMap<String, MtlConfig>,
) -> Result<()> {
    let reader = BufReader::new(file);

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
                    materials.insert(mtl_name, mtl);
                }
                mtl = MtlConfig::default();
                mtl_name = tokens[1].to_string();
            }
            /* diffuse color */
            "Kd" => {
                let kd = parse_vec3(&tokens)?;
                mtl.Kd = Spectrum::from_rgb(kd);
            }
            /* texture map */
            "map_Kd" => {
                if let Some(ref zip) = zip_file {
                    let tex_name = tokens[1].replace('\\', "/");
                    let img = super::_img_from_zip(zip.clone(), &tex_name)?;
                    mtl.map_Kd = Some(img);
                }
            }
            /* emission color */
            "Ke" => {
                let ke = parse_vec3(&tokens)?;
                mtl.Ke = Spectrum::from_rgb(ke);
            }
            /* specular color */
            "Ks" => {
                let ks = parse_vec3(&tokens)?;
                mtl.Ks = Spectrum::from_rgb(ks);
            }
            /* transmission filter */
            "Tf" => {
                let tf = parse_vec3(&tokens)?;
                mtl.Tf = Spectrum::from_rgb(tf);
            }
            /* refraction index */
            "Ni" => {
                let ni = parse_double(tokens[1])?;
                mtl.Ni = ni;
            }
            /* roughness */
            "Ns" => {
                let ns = parse_double(tokens[1])?;
                mtl.Ns = ns;
            }
            /* illumination model */
            "illum" => {
                let illum = parse_double(tokens[1])?;
                mtl.illum = illum as usize;
            }
            _ => (),
        }
    }

    materials.insert(mtl_name, mtl);

    Ok(())
}
