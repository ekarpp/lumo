use super::*;
use crate::tracer::Color;

/// Holds the properties of a microfacet material
pub struct MtlConfig {
    /// Base color of the material
    pub diffuse_color: Color,
    /// Specular color of the material. Currently material color = kd + ks
    pub specular_color: Color,
    /// Emittance of the material. If not zero vector, then createas a light
    pub emission_color: Color,
    /// How much each light channel passes on transmission. Unused ATM
    pub transmission_filter: Vec3,
    /// Refraction index of the material
    pub refraction_idx: Float,
    /// Roughness of the material
    pub roughness: Float,
    /// Illumination model, see docs.
    /// If 6 or 7 makes transparent, if 5 makes metal, otherwise unused.
    pub illumination_model: usize,
}

impl Default for MtlConfig {
    fn default() -> Self {
        Self {
            diffuse_color: Color::BLACK,
            specular_color: Color::BLACK,
            emission_color: Color::BLACK,
            transmission_filter: Vec3::ZERO,
            refraction_idx: 1.5,
            roughness: 1.0,
            illumination_model: 0,
        }
    }
}

impl MtlConfig {
    pub fn build_material(&self) -> Material {
        if !self.emission_color.is_black() {
            Material::Light(Texture::Solid(self.emission_color))
        } else {
            let texture = Texture::Solid(self.diffuse_color + self.specular_color);

            let metallicity = if self.illumination_model == 5 { 1.0 } else { 0.0 };
            let is_transparent = self.illumination_model == 6
                || self.illumination_model == 7;
            Material::microfacet(
                texture,
                self.roughness,
                self.refraction_idx,
                metallicity,
                is_transparent,
            )
        }
    }
}

pub fn load_file(file: File, materials: &mut HashMap<String, MtlConfig>) -> Result<()> {
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
            "Kd" => {
                let kd = parse_vec3(&tokens)?;
                mtl.diffuse_color = Color::from(kd);
            }
            "Ke" => {
                let ke = parse_vec3(&tokens)?;
                mtl.emission_color = Color::from(ke);
            }
            "Ks" => {
                let ks = parse_vec3(&tokens)?;
                mtl.specular_color = Color::from(ks);
            }
            "Tf" => {
                let tf = parse_vec3(&tokens)?;
                mtl.transmission_filter = tf;
            }
            "Ni" => {
                let ni = parse_double(tokens[1])?;
                mtl.refraction_idx = ni;
            }
            "Ns" => {
                let ns = parse_double(tokens[1])?;
                // blender uses this mapping
                let roughness = 1.0 - ns.min(900.0).sqrt() / 30.0;
                mtl.roughness = roughness;
            }
            "illum" => {
                let illum = parse_double(tokens[1])?;
                mtl.illumination_model = illum as usize;
            }
            _ => (),
        }
    }

    materials.insert(mtl_name, mtl);

    Ok(())
}
