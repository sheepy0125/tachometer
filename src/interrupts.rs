//! Interrupts

use crate::pins;
use arduino_hal::{pac::TC0, pins, Peripherals};
use avr_device::{
    atmega328p::exint::{pcicr::PCICR_SPEC, pcmsk0::PCMSK0_SPEC},
    generic::Reg,
    interrupt::{self, Mutex},
};
use core::cell::Cell;

pub use millis::{millis, millis_init};
pub use optical_encoder::{optical_encoder_init, OPTICAL_ENCODER_HITS};

/// This millisecond interrupt was usurped from Rahix's amazing blog:
/// https://blog.rahix.de/005-avr-hal-millis/
mod millis {
    use super::*;

    const PRESCALER: u32 = 1024_u32;
    const TIMER_COUNTS: u32 = 250_u32;
    const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16_000_u32; // 16MHz

    static MILLIS_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0_u32));

    pub unsafe fn millis_init(tc0: TC0) {
        // Configure the timer for the above interval (in CTC mode)
        // and enable its interrupt
        tc0.tccr0a.write(|w| w.wgm0().ctc());
        tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
        tc0.tccr0b.write(|w| match PRESCALER {
            8_u32 => w.cs0().prescale_8(),
            64_u32 => w.cs0().prescale_64(),
            256_u32 => w.cs0().prescale_256(),
            1024_u32 => w.cs0().prescale_1024(),
            _ => panic!(),
        });
        tc0.timsk0.write(|w| w.ocie0a().set_bit());

        // Reset the global millisecond counter
        interrupt::free(|critical_section| {
            MILLIS_COUNTER.borrow(critical_section).set(0_u32);
        });
    }

    #[avr_device::interrupt(atmega328p)]
    #[allow(non_snake_case)]
    fn TIMER0_COMPA() {
        interrupt::free(|critical_section| {
            let counter_cell = MILLIS_COUNTER.borrow(critical_section);
            let counter = counter_cell.get();
            counter_cell.set(counter + MILLIS_INCREMENT);
        })
    }

    /// Milliseconds since the interrupt timer was configured for all times that interrupts were allowed
    pub fn millis() -> u32 {
        interrupt::free(|critical_section| MILLIS_COUNTER.borrow(critical_section).get())
    }
}

mod optical_encoder {
    use super::*;

    pub static OPTICAL_ENCODER_HITS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

    /// Safety note: The caller must ensure that the sensor pin is
    /// pin change interrupt 4 of mask 0!
    pub unsafe fn optical_encoder_init(
        pcicr: &Reg<PCICR_SPEC>,
        pcmsk0: &Reg<PCMSK0_SPEC>,
        _: &pins::optical_encoder::Sensor,
    ) {
        // Enable mask 0 interrupt
        let mut enabled_interrupts = pcicr.read().bits() as u8;
        enabled_interrupts |= 0b1 << 0;
        pcicr.write(|w| unsafe { w.bits(enabled_interrupts) });

        // Configure mask 0
        let mut mask_0_bits = pcmsk0.read().bits() as u8;
        mask_0_bits |= 0b1_u8 << 4; // PCINT4
        pcmsk0.write(|w| w.bits(mask_0_bits));
    }

    #[avr_device::interrupt(atmega328p)]
    #[allow(non_snake_case)]
    fn PCINT2() {
        let peripherals = unsafe { Peripherals::steal() };
        let pins = pins!(peripherals);
        let optical_encoder_pin = pins.d12.into_pull_up_input() as pins::optical_encoder::Sensor;
        if optical_encoder_pin.is_low() {
            return;
        }
        interrupt::free(|critical_section| {
            let counter_cell = OPTICAL_ENCODER_HITS.borrow(critical_section);
            let counter = counter_cell.get();
            counter_cell.set(counter + 1);
        })
    }
}
