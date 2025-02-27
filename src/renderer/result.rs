use super::*;

pub struct RenderResult {
    pub tile: Option<FilmTile>,
    pub num_rays: i32,
}

impl RenderResult {
    pub fn new(tile: FilmTile, num_rays: i32) -> Self {
        Self { tile: Some(tile), num_rays, }
    }

    pub fn null() -> Self {
        Self { tile: None, num_rays: 0, }
    }
}
