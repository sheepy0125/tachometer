use arduino_hal::{
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin,
    },
    Usart,
};

pub const DEBUG: bool = false;
pub const TRACE: bool = false;
pub const BAUD_RATE: u32 = 57_600_u32;
pub const UPDATE_DELTATIME: u16 = 100_u16;
pub type UsbSerial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

pub mod pin_state {
    pub type PinState = bool;
    pub const LOW: bool = false;
    pub const HIGH: bool = true;
}
