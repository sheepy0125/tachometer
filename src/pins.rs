//! Pin types to improve type safety

use arduino_hal::{
    hal::port,
    port::{
        mode::{Input, Output, PullUp},
        Pin,
    },
};
use embedded_hal::digital::v2::OutputPin;

pub mod character_lcd {
    use super::*;
    pub type SerialIn = Pin<Output, port::PB0>;
    pub type Clock = Pin<Output, port::PB1>;
    pub type Latch = Pin<Output, port::PB2>;
}

pub mod optical_encoder {
    use super::*;
    pub type Sensor = Pin<Input<PullUp>, port::PB4>;
}

pub struct ShiftRegisterPins<SerialInput, Clock, Latch>
where
    SerialInput: OutputPin,
    Clock: OutputPin,
    Latch: OutputPin,
{
    pub serial_input: SerialInput,
    pub clock: Clock,
    pub latch: Latch,
}
