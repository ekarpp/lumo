use super::*;

pub mod srgb {
    use super::*;

    const TABLE: &[u8; 9437448] = include_bytes!("srgb.coeff");
    const RESOLUTION: usize = fetch_u32(4, 0) as usize;
    const SCALE_OFFSET: usize = 8;
    const DATA_OFFSET: usize = SCALE_OFFSET + 4 * RESOLUTION;

    const fn fetch_u32(offset: usize, idx: usize) -> u32 {
        ((TABLE[offset + 4 * idx + 3] as u32) << 24)
            | ((TABLE[offset + 4 * idx + 2] as u32) << 16)
            | ((TABLE[offset + 4 * idx + 1] as u32) << 8)
            | ((TABLE[offset + 4 * idx + 0] as u32) << 0)
    }

    const fn fetch_f32(offset: usize, idx: usize) -> TexFloat {
        f32::from_bits(fetch_u32(offset, idx)) as TexFloat
    }

    const fn fetch_data(idx: usize) -> TexFloat {
        fetch_f32(DATA_OFFSET, idx)
    }

    const fn min(a: usize, b: usize) -> usize {
        if a < b { a } else { b }
    }

    pub const fn eval(maxc: usize, xn: TexFloat, yn: TexFloat, zn: TexFloat)
                      -> (TexFloat, TexFloat, TexFloat) {
        let x = xn * (RESOLUTION as TexFloat - 1.0);
        let y = yn * (RESOLUTION as TexFloat - 1.0);
        let xi = min(x as usize, RESOLUTION - 2);
        let yi = min(y as usize, RESOLUTION - 2);
        let mut left = 0;
        let mut right = RESOLUTION - 1;
        while left < right {
            let mid = (left + right) / 2;
            let v = fetch_f32(SCALE_OFFSET, mid);
            if v <= zn {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        let zi = (left + right) / 2 - 1;
        let x1 = x - xi as TexFloat; let x0 = 1.0 - x1;
        let y1 = y - yi as TexFloat; let y0 = 1.0 - y1;
        let scale_zi = fetch_f32(SCALE_OFFSET, zi);
        let scale_zi1 = fetch_f32(SCALE_OFFSET, zi + 1);
        let z1 = (zn - scale_zi) / (scale_zi1 - scale_zi);
        let z0 = 1.0 - z1;

        let dx = 3; let dy = RESOLUTION * dx; let dz = RESOLUTION * dy;

        let offset = (((maxc * RESOLUTION + zi) * RESOLUTION + yi) * RESOLUTION + xi) * 3;

        let mut cs = [0.0; 3];
        let mut i = 0;
        loop {
            let ofst = offset + i;
            let idx = ofst + 0;
            let x00 = fetch_data(idx) * x0 + fetch_data(idx + dx) * x1;
            let idx = ofst + dy;
            let x10 = fetch_data(idx) * x0 + fetch_data(idx + dx) * x1;
            let idx = ofst + dz;
            let x01 = fetch_data(idx) * x0 + fetch_data(idx + dx) * x1;
            let idx = ofst + dz + dy;
            let x11 = fetch_data(idx) * x0 + fetch_data(idx + dx) * x1;

            let y00 = x00 * y0 + x10 * y1;
            let y01 = x01 * y0 + x11 * y1;

            let z00 = y00 * z0 + y01 * z1;

            cs[i] = z00;

            i += 1;
            if i >= 3 { break; }
        }

        (cs[0], cs[1], cs[2])
    }
}
