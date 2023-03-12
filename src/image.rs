use crate::DVec3;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use png::{Encoder, ColorType, BitDepth};

pub struct Image {
    pub buffer: Vec<DVec3>,
    pub width: usize,
    pub height: usize,
    pub fname: String,
}

impl Image {
    fn rgb(&self) -> Vec<u8> {
        let mut rgb_img: Vec<u8> = vec![0; self.width * self.height * 3];

        for y in 0..self.height {
            for x in 0..self.width {
                let px = self.buffer[x + y*self.width];
                let idx = 3*(x + y*self.width);
                rgb_img[idx + 0] = (px.x.sqrt() * 256.0) as u8;
                rgb_img[idx + 1] = (px.y.sqrt() * 256.0) as u8;
                rgb_img[idx + 2] = (px.z.sqrt() * 256.0) as u8;
            }
        }

        rgb_img
    }

    pub fn save(&self) {
        let path = Path::new(&self.fname);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.rgb()).unwrap();
    }
}
