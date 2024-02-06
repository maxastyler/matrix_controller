use core::f32::consts::{PI, TAU};

use embassy_rp::clocks::RoscRng;
use embassy_rp::peripherals::PIO1;
use embassy_rp::rom_data::float_funcs as ff;
use heapless::Vec;
use rand::Rng;

use super::matrix_displayer::MatrixDisplayer;
use super::rgb8::RGB8;
use super::ws2812::Ws2812;

#[derive(Debug)]
pub struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

const SPEED: f32 = 0.001;
const SPEED_INC: f32 = SPEED / 10.0;

impl Ball {
    pub fn new() -> Self {
        let x = ff::fmul(RoscRng.gen(), 16.0);
        let y = ff::fmul(RoscRng.gen(), 16.0);
        let vx = ff::fsub(ff::fmul(RoscRng.gen(), SPEED), ff::fdiv(SPEED, 2.0));
        let vy = ff::fsub(ff::fmul(RoscRng.gen(), SPEED), ff::fdiv(SPEED, 2.0));
        Self { x, y, vx, vy }
    }

    pub fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vx = ff::fadd(
            self.vx,
            ff::fsub(ff::fmul(RoscRng.gen(), SPEED_INC), ff::fdiv(SPEED_INC, 2.0)),
        );
        self.vy = ff::fadd(
            self.vy,
            ff::fsub(ff::fmul(RoscRng.gen(), SPEED_INC), ff::fdiv(SPEED_INC, 2.0)),
        );
        if ff::fcmp(self.x, 16.0) == 1 {
            self.x = 16.0;
            self.vx = -self.vx;
        } else if ff::fcmp(self.x, 0.0) == -1 {
            self.x = 0.0;
            self.vx = -self.vx;
        }
        if ff::fcmp(self.y, 16.0) == 1 {
            self.y = 16.0;
            self.vy = -self.vy;
        } else if ff::fcmp(self.y, 0.0) == -1 {
            self.y = 0.0;
            self.vy = -self.vy;
        }
        let speed_sq = ff::fadd(ff::fmul(self.vx, self.vx), ff::fmul(self.vy, self.vy));
        if ff::fcmp(speed_sq, ff::fmul(SPEED, SPEED)) == 1 {
            let speed = ff::fsqrt(speed_sq);
            self.vx = ff::fdiv(self.vx, speed);
            self.vy = ff::fdiv(self.vy, speed)
        }
    }
}

#[derive(Debug)]
pub struct Metaballs<const N: usize>([Ball; N]);

impl<const N: usize> Metaballs<N> {
    pub fn new() -> Self {
        Self([(); N].map(|_| Ball::new()))
    }
}

impl<const ROWS: usize, const COLS: usize, const N: usize> MatrixDisplayer<ROWS, COLS>
    for Metaballs<N>
{
    fn update(&mut self, buffer: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>) {
        self.0.iter_mut().for_each(|i| i.update());
        for (row, col) in <Metaballs<N> as MatrixDisplayer<ROWS, COLS>>::iterate(self) {
            let (frow, fcol) = (ff::int_to_float(row as i32), ff::int_to_float(col as i32));
            let mut total: f32 = 0.0;
            for Ball { x, y, .. } in self.0.iter() {
                let dx = frow - x;
                let dy = fcol - y;
                let divisor = ff::fsqrt(ff::fadd(ff::fmul(dx, dx), ff::fmul(dy, dy)));
                if ff::fcmp(divisor, 0.0) != 0 {
                    total = ff::fadd(total, ff::fdiv(1.0, divisor));
                }
            }
            buffer[(row, col)] = if total > 1.0 {
                if total > 3.0 {
                    RGB8::new(0, 30, 10)
                } else if total > 2.0 {
                    RGB8::new(
                        0,
                        30,
                        ff::float_to_uint(ff::fmul(ff::fsub(total, 1.0), 10.0)) as u8,
                    )
                } else {
                    RGB8::new(
                        0,
                        ff::float_to_uint(ff::fmul(total, 2.0)) as u8,
                        0,
                    )
                }
            } else {
                RGB8::new(0, 0, 0)
            };
        }
    }
}
