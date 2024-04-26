#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(trait_alias)]
#![feature(stmt_expr_attributes)]

use arduino_hal::{default_serial, delay_ms, I2c};
use console::{println, set_console};
use seven_segment_display::{Display as _, RPMDisplay};
use shared::UsbSerial;
use shared_bus::BusManagerSimple;
use state::State;

pub mod console;
pub mod interrupts;
pub mod panic;
pub mod pins;
pub mod seven_segment_display;
pub mod shared;
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
        digits: [0, 0, 0, 0],
    };

    // Set up pin handles
    let optical_encoder_pin = pins.d12.into_pull_up_input() as pins::optical_encoder::Sensor;
    let iic_pins = pins::IICPins {
        sda: pins.a4.into_pull_up_input(),
        scl: pins.a5.into_pull_up_input(),
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

    // IIC initialization
    let iic = I2c::new(peripherals.TWI, iic_pins.sda, iic_pins.scl, 1);
    let iic_bus = BusManagerSimple::new(iic);

    // Display initialization
    let mut rpm_display = RPMDisplay::new(iic_bus.acquire_i2c());

    // Main loop
    loop {
        rpm_display.display(&state);
        delay_ms(1000);
    }
}
