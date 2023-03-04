extern crate png;

use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

pub struct Image {
    pub buffer: Vec<glam::f32::Vec3>,
    pub width: usize,
    pub height: usize
}

impl Image {
    fn rgb(&self) -> Vec<u8> {
        let mut rgb_img: Vec<u8>
            = Vec::with_capacity(self.width * self.height * 3);
        for y in 0..self.height {
            for x in 0..self.width {
                let px = self.buffer[x + y*self.width] * 255.9;
                rgb_img.push(px.x as u8);
                rgb_img.push(px.y as u8);
                rgb_img.push(px.z as u8);
            }
        }

        rgb_img
    }

    pub fn save(&self) {
        let path = Path::new("cover.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.rgb()).unwrap();
    }
}
