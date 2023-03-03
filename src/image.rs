extern crate png;

pub struct Image {
    pub buffer: Vec<glam::f32::Vec3>,
    pub width: usize,
    pub height: usize
}

pub mod saver {
    use std::path::Path;
    use std::fs::File;
    use std::io::BufWriter;

    pub fn save(img: &super::Image) {
        let path = Path::new("image.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, img.width as u32, img.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let mut rgb_img: Vec<u8> = Vec::with_capacity(img.width * img.height * 3);
        for y in 0..img.height {
            for x in 0..img.width {
                let px = img.buffer[x + y*img.width] * 255.9;
                rgb_img.push(px.x as u8);
                rgb_img.push(px.y as u8);
                rgb_img.push(px.z as u8);
            }
        }

        writer.write_image_data(&rgb_img).unwrap();
    }
}
