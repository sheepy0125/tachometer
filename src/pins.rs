//! Pin types to improve type safety

use arduino_hal::{
    hal::port,
    port::{
        mode::{Input, Output, PullUp},
        Pin,
    },
};
use embedded_hal::digital::v2::OutputPin;

pub mod rpm_display {
    use super::*;
    pub type SerialIn = Pin<Output, port::PD5>;
    pub type Clock = Pin<Output, port::PD6>;
    pub type Latch = Pin<Output, port::PD7>;
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
