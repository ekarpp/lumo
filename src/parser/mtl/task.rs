use super::*;

#[derive(Clone)]
pub struct MtlTaskExecutor {
    zip_bytes: Arc<Vec<u8>>,
    map_ks: bool,
}

impl MtlTaskExecutor {
    pub fn new(zip_bytes: Arc<Vec<u8>>, map_ks: bool) -> Self {
        Self { zip_bytes, map_ks }
    }
}

impl Executor<MtlTask, MtlTaskResult> for MtlTaskExecutor {
    fn exec(&mut self, task: MtlTask) -> MtlTaskResult {
        let mut mtl = MtlConfig::default();
        let mut mtl_name = String::default();
        for line in task.lines {
            let tokens: Vec<&str> = line.split_ascii_whitespace().collect();
            match tokens[0] {
                "newmtl" => mtl_name = tokens[1].to_string(),
                /* diffuse color */
                "Kd" => {
                    let kd = parse_vec3(&tokens)
                        .expect("Couldn't parse vec3");
                    mtl.Kd = Spectrum::from_rgb(RGB::from(kd));
                }
                /* texture map */
                "map_Kd" => {
                    let tex_name = tokens[1..].join(" ").replace('\\', "/");
                    let img = super::_img_from_zip(&self.zip_bytes, &tex_name)
                        .expect("Couldn't extract image");
                    mtl.map_Kd = Some(img);
                }
                /* emission color */
                "Ke" => {
                    let ke = parse_vec3(&tokens)
                        .expect("Couldn't parse vec3");
                    mtl.Ke = Spectrum::from_rgb(RGB::from(ke));
                }
                "map_Ke" => {
                    let tex_name = tokens[1..].join(" ").replace('\\', "/");
                    let img = super::_img_from_zip(&self.zip_bytes, &tex_name)
                        .expect("Couldn't extract image");
                    mtl.map_Ke = Some(img);
                }
                /* specular color */
                "Ks" => {
                    let ks = parse_vec3(&tokens)
                        .expect("Couldn't parse vec3");
                    mtl.Ks = Spectrum::from_rgb(RGB::from(ks));
                }
                "map_Ks" => {
                    let tex_name = tokens[1..].join(" ").replace('\\', "/");
                    if self.map_ks {
                        let img = super::_img_from_zip(&self.zip_bytes, &tex_name)
                            .expect("Couldn't extract image");
                        mtl.map_Ks = Some(img);
                    } else {
                        let bytes = super::_extract_zip(&self.zip_bytes, &tex_name)
                            .expect("Couldn't extract image");
                        // occlusion, roughness, metalness
                        let orm = Image::<Vec3>::mean_vec3_from_file(bytes.as_slice())
                            .expect("Couldn't decode image");
                        mtl.roughness = orm.y;
                        mtl.k = orm.z;
                        mtl.Ks = Spectrum::WHITE;
                    }
                }
                /* bump map */
                "map_Bump" => {
                    let map_name = tokens[1..].join(" ").replace('\\', "/");
                    let bytes = super::_extract_zip(&self.zip_bytes, &map_name)
                        .expect("Couldn't extract image");
                    mtl.map_Bump = Some(Image::bump_from_file(bytes.as_slice())
                                        .expect("Couldn't decode image"));
                }
                /* transmission filter */
                "Tf" => {
                    let tf = parse_vec3(&tokens)
                        .expect("Couldn't parse vec3");
                    mtl.Tf = Spectrum::from_rgb(RGB::from(tf));
                }
                /* refraction index */
                "Ni" => {
                    let ni = parse_double(tokens[1])
                        .expect("Couldn't parse double");
                    mtl.eta = ni;
                }
                /* roughness */
                "Ns" => {
                    let ns = parse_double(tokens[1])
                        .expect("Couldn't parse double");
                    // blender uses this mapping
                    mtl.roughness = 1.0 - ns.min(900.0).sqrt() / 30.0;
                }
                /* illumination model */
                "illum" => {
                    let illum = parse_double(tokens[1])
                        .expect("Couldn't parse double")as usize;
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

        MtlTaskResult::new(mtl, mtl_name)
    }
}

pub struct MtlTask {
    pub lines: Vec<String>,
}

impl MtlTask {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

pub struct MtlTaskResult {
    pub mtl_cfg: MtlConfig,
    pub mtl_name: String,
}

impl MtlTaskResult {
    pub fn new(mtl_cfg: MtlConfig, mtl_name: String) -> Self {
        Self { mtl_cfg, mtl_name }
    }
}
