#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod display;
mod network;
mod web;

use crate::display::matrix_displayer::matrix_task;
use crate::web::start_server;
use crate::web::WEB_TASK_POOL_SIZE;

use defmt as _;
use defmt_rtt as _;
use display::matrix_displayer::Displays;
use display::metaballs::Metaballs;
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use panic_probe as _;

use crate::network::set_up_network_stack;

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
    PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO1>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::peripherals::USB) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

static MATRIX_DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, Displays> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    MATRIX_DISPLAY_SIGNAL.signal(Displays::Metaballs(Metaballs::new()));
    let p = embassy_rp::init(Default::default());

    spawner.must_spawn(logger_task(p.USB));

    let (_, stack) = set_up_network_stack(
        &spawner, p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0,
    )
    .await;

    start_server(&spawner, stack).await;

    let pio_led = Pio::new(p.PIO1, Irqs);
    spawner.must_spawn(matrix_task(
        pio_led,
        p.DMA_CH1,
        p.PIN_16,
        &MATRIX_DISPLAY_SIGNAL,
    ));
}
