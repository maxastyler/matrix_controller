use core::{
    ops::{Index, IndexMut},
    slice,
};

use fixed::types::U24F8;
use fixed_macro::fixed;

use embassy_rp::{
    clocks,
    dma::{AnyChannel, Channel},
    into_ref,
    pio::{Common, Config, FifoJoin, Instance, PioPin, ShiftConfig, StateMachine},
    Peripheral, PeripheralRef,
};
use picoserve::response::ws;

#[derive(Default, Debug, Copy, Clone)]
#[repr(C, align(4))]
pub struct RGB8 {
    pub padding: u8,
    pub b: u8,
    pub r: u8,
    pub g: u8,
}

impl RGB8 {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            padding: 0,
            g,
            r,
            b,
        }
    }

    pub fn half(self) -> Self {
        Self {
            r: self.r / 10,
            g: self.g / 10,
            b: self.b / 10,
            padding: 0,
        }
    }
}

impl From<(u8, u8, u8)> for RGB8 {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        RGB8::new(r, g, b)
    }
}

pub struct Ws2812<'d, P: Instance, const S: usize, const ROWS: usize, const COLS: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
    colours: [[RGB8; COLS]; ROWS],
}

impl<'d, P: Instance, const S: usize, const ROWS: usize, const COLS: usize>
    Ws2812<'d, P, S, ROWS, COLS>
{
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        pin: impl PioPin,
    ) -> Self {
        into_ref!(dma);

        // Setup sm0

        // prepare the PIO program
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        const T1: u8 = 2; // start bit
        const T2: u8 = 5; // data bit
        const T3: u8 = 3; // stop bit
        const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        let mut do_zero = a.label();
        a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
        a.bind(&mut wrap_target);
        // Do stop bit
        a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
        // Do start bit
        a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
        // Do data bit = 1
        a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
        a.bind(&mut do_zero);
        // Do data bit = 0
        a.nop_with_delay_and_side_set(T2 - 1, 0);
        a.bind(&mut wrap_source);

        let prg = a.assemble_with_wrap(wrap_source, wrap_target);
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        cfg.use_program(&pio.load_program(&prg), &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        // TODO CLOCK_FREQ should come from embassy_rp
        let clock_freq = U24F8::from_num(clocks::clk_sys_freq() / 1000);
        let ws2812_freq = fixed!(800: U24F8);
        let bit_freq = ws2812_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 24,
            direction: embassy_rp::pio::ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.map_into(),
            sm,
            colours: [[RGB8::default(); COLS]; ROWS],
        }
    }

    pub async fn write(&mut self) {
        // DMA transfer
        self.sm
            .tx()
            .dma_push(self.dma.reborrow(), unsafe {
                let d: &[u32] =
                    slice::from_raw_parts(self.colours.as_ptr() as *const u32, ROWS * COLS);
                d
            })
            .await;
    }
}

impl<'d, P: Instance, const S: usize, const ROWS: usize, const COLS: usize> Index<(usize, usize)>
    for Ws2812<'d, P, S, ROWS, COLS>
{
    type Output = RGB8;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        if row & 1 == 0 {
            // we are on an even row, reverse direction of column
            &self.colours[row][COLS - 1 - col]
        } else {
            // we are on an odd row, keep direction of column
            &self.colours[row][col]
        }
    }
}

impl<'d, P: Instance, const S: usize, const ROWS: usize, const COLS: usize> IndexMut<(usize, usize)>
    for Ws2812<'d, P, S, ROWS, COLS>
{
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        if row & 1 == 0 {
            // we are on an even row, reverse direction of column
            self.colours
                .get_mut(row)
                .unwrap()
                .get_mut(COLS - 1 - col)
                .unwrap()
        } else {
            // we are on an odd row, keep direction of column
            self.colours.get_mut(row).unwrap().get_mut(col).unwrap()
        }
    }
}
