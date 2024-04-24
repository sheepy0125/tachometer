#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(trait_alias)]
#![feature(stmt_expr_attributes)]

use arduino_hal::{default_serial, delay_ms};
use console::{println, set_console};
use pins::ShiftRegisterPins;
use seven_segment_display::{Display as _, RPMDisplay};
use shared::UsbSerial;
use state::State;

pub mod console;
pub mod interrupts;
pub mod panic;
pub mod pins;
pub mod seven_segment_display;
pub mod shared;
pub mod shift_register;
pub mod state;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let serial: UsbSerial = default_serial!(peripherals, pins, shared::BAUD_RATE);
    set_console(serial);

    println!("Hello from the Tachometer!");

    // State initialization
    let mut state = State {
        digits: [1, 2, 3, 4],
    };

    // Set up pin handles
    let optical_encoder_pin = pins.d12.into_pull_up_input() as pins::optical_encoder::Sensor;
    let rpm_display_shift_register_pins = ShiftRegisterPins {
        latch: pins.d7.into_output() as pins::rpm_display::Latch,
        serial_input: pins.d5.into_output() as pins::rpm_display::SerialIn,
        clock: pins.d6.into_output() as pins::rpm_display::Clock,
    };
    // Interrupt initializations
    unsafe {
        interrupts::millis_init(peripherals.TC0);
        interrupts::optical_encoder_init(
            &peripherals.EXINT.pcicr,
            &peripherals.EXINT.pcmsk0,
            &optical_encoder_pin,
        );
        avr_device::interrupt::enable();
    };

    // Display initialization
    let mut rpm_display = RPMDisplay::new(shift_register::ShiftRegister::from_pins(
        rpm_display_shift_register_pins,
    ));

    // Main loop
    loop {
        rpm_display.display(&state);
        for digit in state.digits.iter_mut() {
            *digit += 1;
            if *digit > 9 {
                *digit = 0;
            }
        }
        delay_ms(1000);
    }
}
