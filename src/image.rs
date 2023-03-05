use glam::f64::DVec3;

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
                let px = self.buffer[x + y*self.width] * 255.9;
                let idx = 3*(x + y*self.width);
                rgb_img[idx + 0] = px.x as u8;
                rgb_img[idx + 1] = px.y as u8;
                rgb_img[idx + 2] = px.z as u8;
            }
        }

        rgb_img
    }

    pub fn save(&self) {
        let path = std::path::Path::new(&self.fname);
        let file = std::fs::File::create(path).unwrap();
        let ref mut w = std::io::BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.rgb()).unwrap();
    }
}
