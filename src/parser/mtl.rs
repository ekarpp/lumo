use super::*;
use crate::pool::{Executor, ThreadPool};
use crate::tracer::{RGB, Spectrum};

use task::{MtlTask, MtlTaskExecutor};

mod task;

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
    zip_file: Arc<Vec<u8>>,
    materials: &mut Vec<Material>,
    material_indices: &mut FxHashMap<String, usize>,
) -> Result<()> {
    let reader = BufReader::new(bytes);
    let mut block = Vec::new();

    let executor = MtlTaskExecutor::new(zip_file, map_ks);
    let threads = 4;
    let pool = ThreadPool::new(
        threads,
        executor,
    );

    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cmd = line.split_ascii_whitespace().nth(0).unwrap();

        if cmd == "newmtl" && !block.is_empty() {
            pool.publish(MtlTask::new(block));
            block = Vec::new();
        }

        block.push(line);
    }

    if !block.is_empty() {
        pool.publish(MtlTask::new(block));
    }

    pool.all_published();

    let mut finished = 0;
    while finished < threads {
        let result = pool.pop_result();
        if let Some(result) = result {
            if !material_indices.contains_key(&result.mtl_name) {
                println!("{}", result.mtl_name);
                materials.push(result.mtl_cfg.build_material());
                material_indices.insert(result.mtl_name, materials.len() - 1);
            }
        } else {
            finished += 1;
        }
    }

    Ok(())
}
