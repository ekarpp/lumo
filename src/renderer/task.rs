use super::*;

pub type RenderTaskExecutor =
    dyn Fn(RenderTask, &RefCell<Xorshift>, &Camera, &Scene)
           -> RenderTaskResult + Send + Sync + 'static;

pub struct RenderTask {
    pub tile: Option<FilmTile>,
    pub samples: u64,
}

impl RenderTask {
    pub fn new(tile: FilmTile, samples: u64) -> Self {
        Self { tile: Some( tile ), samples }
    }

    pub fn null() -> Self {
        Self { tile: None, samples: 0 }
    }
}

pub struct RenderTaskResult {
    pub tile: Option<FilmTile>,
    pub num_rays: u64,
}

impl RenderTaskResult {
    pub fn new(tile: FilmTile, num_rays: u64) -> Self {
        Self { tile: Some(tile), num_rays, }
    }

    pub fn null() -> Self {
        Self { tile: None, num_rays: 0, }
    }

    pub fn is_null(&self) -> bool {
        self.tile.is_none() && self.num_rays == 0
    }
}
