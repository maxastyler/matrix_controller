use embassy_rp::peripherals::PIO1;

use crate::{
    matrix_displayer::MatrixDisplayer,
    ws2812::{Ws2812, RGB8},
};

pub struct Single(pub usize);

impl<const ROWS: usize, const COLS: usize> MatrixDisplayer<ROWS, COLS> for Single {
    fn update(&mut self, buffer: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        let col = self.0.rem_euclid(ROWS);
        let row = self.0.div_euclid(ROWS);
        buffer[(row, col)] = RGB8::new(0, 0, 0);
        self.0 = (self.0 + 1).rem_euclid(256);
        let col = self.0.rem_euclid(ROWS);
        let row = self.0.div_euclid(ROWS);
        buffer[(row, col)] = RGB8::new(30, 30, 30);
    }
}
