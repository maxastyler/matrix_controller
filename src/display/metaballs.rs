use core::f32::consts::{PI, TAU};

use embassy_rp::peripherals::PIO1;
use embassy_rp::rom_data::float_funcs as ff;

use super::matrix_displayer::MatrixDisplayer;
use super::ws2812::{Ws2812, RGB8};

const BALLS: [(f32, f32, f32, f32); 5] = [
    (1.0, 0.0, 2.0, 1.0),
    (1.0, PI, 1.0, 0.0),
    (4.0, 2.0, 3.4, 0.0),
    (0.1, 2.0, 3.4, 3.0),
    (2.0, 1.0, 0.1, 0.0),
];

pub struct Metaballs(f32);

impl Metaballs {
    pub fn new() -> Self {
        Self(0.0)
    }
}

impl<const ROWS: usize, const COLS: usize> MatrixDisplayer<ROWS, COLS> for Metaballs {
    fn update(&mut self, buffer: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        for (row, col) in <Metaballs as MatrixDisplayer<ROWS, COLS>>::iterate(self) {
            let (frow, fcol) = (ff::int_to_float(row as i32), ff::int_to_float(col as i32));
            let mut total: f32 = 0.0;
            for (row_freq, row_offset, col_freq, col_offset) in BALLS {
                let brow = ff::fmul(ff::fadd(ff::fsin(self.0 * row_freq + row_offset), 1.0), 8.0);
                let bcol = ff::fmul(ff::fadd(ff::fsin(self.0 * col_freq + col_offset), 1.0), 8.0);
                let x = frow - brow;
                let y = fcol - bcol;
                let divisor = ff::fsqrt(ff::fadd(ff::fmul(x, x), ff::fmul(y, y)));
                if ff::fcmp(divisor, 0.0) != 0 {
                    total = ff::fadd(total, ff::fdiv(1.0, divisor));
                }
            }
            buffer[(row, col)] = if total > 1.0 {
                RGB8::new(30, 0, 0)
            } else {
                RGB8::new(0, 0, 0)
            };
        }
        self.0 += 0.01;
        while self.0 > PI {
            self.0 -= TAU;
        }
    }
}
