use crate::state::State;

use adafruit_7segment::{Index, SevenSegment};
use embedded_hal::blocking::i2c;
use ht16k33::{Dimming, Display as DisplayState, HT16K33};

pub const RPM_CONTROLLER_ADDRESS: u8 = 0x70;

pub trait Display {
    fn display(&mut self, state: &State);
}

/// This is a KW4-12041CUYA 4-digit 7-segment display behind the HT16K33 IIC backpack
pub struct RPMDisplay<I> {
    controller: HT16K33<I>,
}
impl<I, E> RPMDisplay<I>
where
    I: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(iic: I) -> Self {
        let mut controller = HT16K33::new(iic, RPM_CONTROLLER_ADDRESS);
        controller
            .initialize()
            .expect("Failed to initialize RPM display");
        controller
            .set_display(DisplayState::ON)
            .expect("Failed to turn on the RPM display");
        controller
            .set_dimming(Dimming::BRIGHTNESS_MAX)
            .expect("Failed to set dimming on RPM display");
        controller.update_buffer_with_colon(false);
        Self { controller }
    }
}
impl<I, E> Display for RPMDisplay<I>
where
    I: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn display(&mut self, state: &State) {
        self.controller
            .update_buffer_with_digit(Index::One, state.display_digits[0]);
        self.controller
            .update_buffer_with_digit(Index::Two, state.display_digits[1]);
        self.controller
            .update_buffer_with_digit(Index::Three, state.display_digits[2]);
        self.controller
            .update_buffer_with_digit(Index::Four, state.display_digits[3]);
        let _ = self.controller.write_display_buffer();
    }
}
