#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(trait_alias)]
#![feature(stmt_expr_attributes)]

use ag_lcd::{Cursor, Display as LcdDisplayMode, LcdDisplay, Lines};
use arduino_hal::default_serial;
use console::{debug, println, set_console};
use pins::ShiftRegisterPins;
use shared::UsbSerial;
use shift_register_driver::sipo::ShiftRegister8 as DecomposableShiftRegister;

pub mod console;
pub mod interrupts;
pub mod panic;
pub mod pins;
pub mod shared;
pub mod shift_register;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let serial: UsbSerial = default_serial!(peripherals, pins, shared::BAUD_RATE);
    set_console(serial);

    println!("Hello from the Tachometer!");

    // Set up pin handles
    let optical_encoder_pin = pins.d12.into_pull_up_input() as pins::optical_encoder::Sensor;
    let character_lcd_shift_register_pins = ShiftRegisterPins {
        serial_input: pins.d8.into_output() as pins::character_lcd::SerialIn,
        clock: pins.d9.into_output() as pins::character_lcd::Clock,
        latch: pins.d10.into_output() as pins::character_lcd::Latch,
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

    // Character LCD initialization
    debug!("[DEBUG] Character LCD shift register initialization");
    let character_lcd_shift_register = DecomposableShiftRegister::new(
        character_lcd_shift_register_pins.clock,
        character_lcd_shift_register_pins.latch,
        character_lcd_shift_register_pins.serial_input,
    );
    let character_lcd_pins = character_lcd_shift_register.decompose();
    let mut character_lcd: LcdDisplay<_, _> = match character_lcd_pins {
        // Refer to KiCad schematic for pin layout
        [_, rs, _, enabled, db4, db5, db6, db7] => {
            LcdDisplay::new(rs, enabled, arduino_hal::Delay::new())
                .with_half_bus(db4, db5, db6, db7)
                .with_display(LcdDisplayMode::On)
                .with_cursor(Cursor::On)
                .with_lines(Lines::TwoLines)
                .build()
        }
    };

    character_lcd.print("Tachometer!!!");

    // Main loop
    loop {}
}
