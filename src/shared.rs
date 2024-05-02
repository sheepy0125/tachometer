use arduino_hal::{
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin,
    },
    Usart,
};

pub const DEBUG: bool = true;
pub const TRACE: bool = true;
pub const BAUD_RATE: u32 = 57_600;
pub const UPDATE_DELTATIME: u32 = 250;
pub const DIGITS: usize = 4;
pub const HISTORY_LEN: usize = 64;
pub const MAX_DELTATIME: u32 = 10_000;

pub type UsbSerial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;
pub type Milliseconds = u32;

pub mod pin_state {
    pub type PinState = bool;
    pub const LOW: bool = false;
    pub const HIGH: bool = true;
}
