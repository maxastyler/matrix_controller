use core::any::Any;

use embassy_rp::{clocks::RoscRng, peripherals::PIO1};
use embedded_graphics_core::pixelcolor::Rgb888;
use heapless::Vec;
use rand::Rng;
use tinybmp::Bmp;

use super::{matrix_displayer::MatrixDisplayer, rgb8::RGB8, ws2812::Ws2812};

const CAKE_IMG: &[u8] = include_bytes!("../../images/cake_bare.bmp");

pub struct Particle {
    age: u8,
    row: u8,
    col: u8,
}

pub struct Cake<const N: usize>(Vec<Particle, N>, Bmp<'static, Rgb888>);

impl<const N: usize> Cake<N> {
    pub fn new() -> Self {
        Self(Vec::new(), Bmp::from_slice(CAKE_IMG).unwrap())
    }
}

impl<const N: usize, const ROWS: usize, const COLS: usize> MatrixDisplayer<ROWS, COLS> for Cake<N> {
    fn update(&mut self, ws2812: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        self.1.pixels().for_each(|p| {
	    let pix: RGB8 = p.1.into();
            ws2812[(p.0.y as usize, p.0.x as usize)] = pix.div();
        });
        // self.0.push()
        if !self.0.is_full() {
            if RoscRng.gen_bool(0.4) {
                self.0.push(Particle {
                    age: 0,
                    row: 3,
                    col: 6,
                });
            }
        }
        self.0.retain_mut(|p| {
            p.age += 1;
            p.row = p.row.saturating_sub(1);
            match RoscRng.gen_range(0..3) {
                0 => p.col = p.col.saturating_sub(1),
                1 => (),
                2 => p.col = p.col.saturating_add(1),
                _ => unreachable!(),
            };
            if p.age > 5 {
                false
            } else {
                true
            }
        });
        self.0.iter().for_each(|p| {
            ws2812[(p.row as usize, p.col as usize)] = match p.age {
                0 => RGB8::new(20, 18, 8),
                1 => RGB8::new(100, 51, 0),
                _ => RGB8::new(100, 32, 0),
            }
        });
    }
}
