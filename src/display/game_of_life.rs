use embassy_rp::peripherals::PIO1;

use crate::{matrix_displayer::MatrixDisplayer, ws2812::Ws2812};

pub struct GameOfLife<const ROWS: usize, const COLS: usize>(pub [[bool; COLS]; ROWS]);

impl<const ROWS: usize, const COLS: usize> MatrixDisplayer<ROWS, COLS> for GameOfLife<ROWS, COLS> {
    fn update(&mut self, buffer: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        self.iterate().for_each(|(r, c)| {})
    }
}
