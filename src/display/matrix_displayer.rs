use embassy_futures::select::select;
use embassy_rp::{
    peripherals::{DMA_CH1, PIN_16, PIO1},
    pio::Pio,
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use log::info;

use super::{cake::Cake, metaballs::Metaballs, wheel::Wheel, ws2812::Ws2812};

pub trait MatrixDisplayer<const ROWS: usize, const COLS: usize> {
    fn update(&mut self, ws2812: &mut Ws2812<'_, PIO1, 0, ROWS, COLS>);
    fn iterate(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..ROWS).flat_map(|r| (0..COLS).map(move |c| (r, c)))
    }
}

pub enum Displays {
    Wheel(Wheel),
    // Wrap(Wrap),
    // Single(Single),
    Metaballs(Metaballs),
    Cake(Cake<10>),
}

impl TryFrom<usize> for Displays {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Wheel(Wheel(0)),
            1 => Self::Metaballs(Metaballs::new()),
            _ => return Err(()),
        })
    }
}

async fn change_on_signal(
    state: &mut Displays,
    signal: &'static Signal<CriticalSectionRawMutex, Displays>,
) {
    *state = signal.wait().await;
}

#[embassy_executor::task]
pub async fn matrix_task(
    mut pio: Pio<'static, PIO1>,
    dma: DMA_CH1,
    pin: PIN_16,
    signal: &'static Signal<CriticalSectionRawMutex, Displays>,
) {
    let mut ws2812: Ws2812<'_, embassy_rp::peripherals::PIO1, 0, 16, 16> =
        Ws2812::new(&mut pio.common, pio.sm0, dma, pin);
    let mut state = Displays::try_from(0).unwrap();
    loop {
        match state {
            Displays::Wheel(ref mut w) => {
                w.update(&mut ws2812);
            }
            // Displays::Wrap(ref mut w) => {
            //     w.update(&mut ws2812);
            // }
            // Displays::Single(ref mut w) => {
            //     w.update(&mut ws2812);
            // }
            Displays::Metaballs(ref mut w) => {
                w.update(&mut ws2812);
            }
            Displays::Cake(ref mut w) => {
                w.update(&mut ws2812);
            }
        }
        ws2812.write().await;
        select(
            Timer::after_millis(10),
            change_on_signal(&mut state, signal),
        )
        .await;
    }
}
