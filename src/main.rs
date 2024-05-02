#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(trait_alias)]
#![feature(stmt_expr_attributes)]

use console::{println, set_console};
use interrupts::{millis, rpm};
use rpm::calculate_rpm;
use seven_segment_display::{Display as _, RPMDisplay};
use shared::{UsbSerial, DIGITS, UPDATE_DELTATIME};
use state::State;

use arduino_hal::{default_serial, delay_ms, I2c};
use shared_bus::BusManagerSimple;

pub mod console;
pub mod interrupts;
pub mod panic;
pub mod pins;
pub mod rpm;
pub mod seven_segment_display;
pub mod shared;
pub mod state;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let serial: UsbSerial = default_serial!(peripherals, pins, shared::BAUD_RATE);
    set_console(serial);

    println!("Hello from the Tachometer! (https://github.com/sheepy0125/tachometer)");

    // State initialization
    let mut state = State {
        display_digits: [0; DIGITS],
    };

    // Set up pin handles
    let sensor_pin = pins.d2.into_pull_up_input() as pins::sensor::Sensor;
    let iic_pins = pins::IICPins {
        sda: pins.a4.into_pull_up_input(),
        scl: pins.a5.into_pull_up_input(),
    };

    // Interrupt initializations
    unsafe {
        interrupts::millis_init(peripherals.TC0);
        interrupts::sensor_init(
            &peripherals.EXINT.eicra,
            &peripherals.EXINT.eimsk,
            &sensor_pin,
        );
        avr_device::interrupt::enable();
    };

    // IIC initialization
    let iic = I2c::new(peripherals.TWI, iic_pins.sda, iic_pins.scl, 1);
    let iic_bus = BusManagerSimple::new(iic);

    // Display initialization
    let mut rpm_display = RPMDisplay::new(iic_bus.acquire_i2c());

    // Main loop

    let mut update_time = 0;
    loop {
        delay_ms(UPDATE_DELTATIME as u16);
        let current_time = millis();
        if update_time > current_time {
            continue;
        }
        update_time = current_time + UPDATE_DELTATIME;

        let rpm_info = rpm();
        let rpm = calculate_rpm(current_time, &rpm_info);
        let mut display_count = rpm;
        if display_count > 9999 {
            display_count = 9999;
        }
        console::debug!("RPM: {}, History count: {}", rpm, rpm_info.count);

        for digit in state.display_digits.iter_mut().rev() {
            *digit = (display_count % 10) as u8;
            display_count = display_count / 10;
        }
        rpm_display.display(&state);
    }
}
