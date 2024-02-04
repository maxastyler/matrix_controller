use core::cmp::{max, min};

use embassy_rp::rom_data::float_funcs as ff;
use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};

#[derive(Default, Debug, Copy, Clone)]
#[repr(C, align(4))]
pub struct RGB8 {
    pub padding: u8,
    pub b: u8,
    pub r: u8,
    pub g: u8,
}

impl From<Rgb888> for RGB8 {
    fn from(value: Rgb888) -> Self {
        Self {
            padding: 0,
            b: value.b(),
            r: value.r(),
            g: value.g(),
        }
    }
}

// fn fmod(a: f32, b: f32) -> f32 {

// }

impl RGB8 {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            padding: 0,
            g,
            r,
            b,
        }
    }

    // fn hsv(self) -> (f32, f32, f32) {
    //     let (r, g, b) = (
    //         ff::fdiv(self.r as f32, 255.0),
    //         ff::fdiv(self.g as f32, 255.0),
    //         ff::fdiv(self.b as f32, 255.0),
    //     );

    //     let xmax = r.max(g).max(b);
    //     let xmin = r.min(g).min(b);
    //     let c = ff::fsub(xmax, xmin);
    //     let l = ff::fdiv(ff::fadd(xmax, xmin), 2.0);
    //     let v = ff::fadd(l, ff::fdiv(c, 2.0));
    //     let sv = if ff::fcmp(v, 0.0) != 1 {
    //         0.0
    //     } else {
    //         ff::fdiv(c, v)
    //     };
    // }
}

impl From<(u8, u8, u8)> for RGB8 {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        RGB8::new(r, g, b)
    }
}
