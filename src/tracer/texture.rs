use crate::DVec3;

pub enum Texture {
    Solid(DVec3),
}

impl Texture {
    pub fn color_at(&self, p: DVec3) -> DVec3 {
        match self {
            Texture::Solid(c) => c.clone(),
        }
    }
}
