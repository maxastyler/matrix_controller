use embassy_rp::{
    peripherals::PIO1,
    rom_data::float_funcs::{float_to_uint64, fsin},
};

use super::{
    matrix_displayer::MatrixDisplayer,
    ws2812::{Ws2812, RGB8},
};

pub struct Wrap(pub usize);

impl<const ROWS: usize, const COLS: usize> MatrixDisplayer<ROWS, COLS> for Wrap {
    fn update(&mut self, buffer: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        (0..ROWS)
            .flat_map(|r| (0..COLS).map(move |c| (r, c)))
            .enumerate()
            .for_each(|(i, (r, c))| {
                // let offset = (self.0 + i).rem_euclid(256) as u8;
                let offset =
                    float_to_uint64(((fsin((self.0 + i) as f32) + 1.0) * 255.0 / 2.0)) as u8;
                buffer[(r, c)] = if self.0 < 256 {
                    RGB8::new(offset, 0, 0)
                } else if self.0 < 256 * 2 {
                    RGB8::new(0, offset, 0)
                } else {
                    RGB8::new(0, 0, offset)
                }
                .half();
            });
        self.0 = (self.0 + 1).rem_euclid(3 * 256);
    }
}
