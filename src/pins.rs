//! Pin types to improve type safety

use arduino_hal::{
    hal::port,
    port::{
        mode::{Input, PullUp},
        Pin,
    },
};

pub mod optical_encoder {
    use super::*;
    pub type Sensor = Pin<Input<PullUp>, port::PB4>;
}

pub mod iic {
    use super::*;
    pub type SDA = Pin<Input<PullUp>, port::PC4>;
    pub type SCL = Pin<Input<PullUp>, port::PC5>;
}

pub struct IICPins {
    pub sda: self::iic::SDA,
    pub scl: self::iic::SCL,
}
