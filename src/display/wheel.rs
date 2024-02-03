use embassy_rp::peripherals::PIO1;

use super::{matrix_displayer::MatrixDisplayer, ws2812::{Ws2812, RGB8}};

#[derive(Debug)]
pub struct Wheel(pub usize);

impl<const COLS: usize, const ROWS: usize> MatrixDisplayer<ROWS, COLS> for Wheel {
    fn update(&mut self, buffer: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        (0..ROWS)
            .flat_map(|r| (0..COLS).map(move |c| (r, c)))
            .enumerate()
            .for_each(|(i, (r, c))| {
                buffer[(r, c)] =
                    wheel((((i * 256) as u16 / (ROWS * COLS) as u16 + self.0 as u16) & 255) as u8);
            });
        self.0 += 1;
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
pub fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
