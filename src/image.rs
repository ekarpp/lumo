use crate::DVec3;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use png::{Encoder, ColorType, BitDepth};

/// Contains the necessary data to write buffer to file.
pub struct Image {
    /// Image buffer storing RGB-channels in range \[0,1\].
    pub buffer: Vec<DVec3>,
    /// Width of rendered image.
    pub width: usize,
    /// Height of rendered image.
    pub height: usize,
    /// Filename of the output file.
    pub fname: String,
}

impl Image {
    pub fn new(buffer: Vec<DVec3>, width: usize, height: usize, fname: String)
               -> Self {
        Self {
            buffer,
            width,
            height,
            fname,
        }
    }
    /// Translates the image buffer of RGB values in range \[0,1\]
    /// to discrete range \[0,256\]. Applies gamma correction.
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

    /// Creates the PNG file
    pub fn save(&self) {
        let path = Path::new(&self.fname);
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.rgb()).unwrap();
    }
}
