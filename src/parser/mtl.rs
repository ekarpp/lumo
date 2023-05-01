use super::*;

/// Holds the properties of a material
pub struct MtlConfig {
    pub diffuse_color: DVec3,
    pub specular_color: DVec3,
    pub emission_color: DVec3,
    pub refraction_idx: f64,
    pub roughness: f64,
    pub illumination_model: usize,
}

impl Default for MtlConfig {
    fn default() -> Self {
        Self {
            diffuse_color: DVec3::ZERO,
            specular_color: DVec3::ZERO,
            emission_color: DVec3::ZERO,
            refraction_idx: 1.5,
            roughness: 1.0,
            illumination_model: 0,
        }
    }
}

impl MtlConfig {
    pub fn build_material(&self) -> Material {
        if self.emission_color.length_squared() != 0.0 {
            Material::Light(Texture::Solid(self.emission_color))
        } else {
            let texture = Texture::Solid(self.diffuse_color);
            let is_transparent = self.illumination_model == 6
                || self.illumination_model == 7;
            Material::microfacet(
                texture,
                self.roughness,
                self.refraction_idx,
                0.0,
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
                mtl.diffuse_color = kd;
            }
            "Ke" => {
                let ke = parse_vec3(&tokens)?;
                mtl.emission_color = ke;
            }
            "Ks" => {
                let ks = parse_vec3(&tokens)?;
                mtl.specular_color = ks;
            }
            "Ni" => {
                let ni = parse_double(&tokens[1])?;
                mtl.refraction_idx = ni;
            }
            "Ns" => {
                let ns = parse_double(&tokens[1])?;
                // blender uses this mapping
                let roughness = 1.0 - ns.min(900.0).sqrt() / 30.0;
                mtl.roughness = roughness;
            }
            "illum" => {
                let illum = parse_double(&tokens[1])?;
                mtl.illumination_model = illum as usize;
            }
            _ => (),
        }
    }

    materials.insert(mtl_name, mtl);

    Ok(())
}
