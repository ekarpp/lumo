use super::*;

pub struct RenderTask {
    pub tile: Option<FilmTile>,
    pub samples: u32,
}

impl RenderTask {
    pub fn new(tile: FilmTile, samples: u32) -> Self {
        Self { tile: Some( tile ), samples }
    }

    pub fn null() -> Self {
        Self { tile: None, samples: 0 }
    }
}
