//! All time displays

use crate::{
    console::debug,
    pins,
    shared::pin_state::{PinState, HIGH, LOW},
    shared::DIGITS,
    shift_register::ShiftRegister,
    state::State,
};

pub trait Display {
    fn display(&mut self, state: &State);
}

/// A, B, C, D, E, F, G, & DP pin states for a given digit index
const SEVEN_SEGMENT_OUTPUT: [[PinState; 8]; 10 + 1] = [
    [HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, LOW, LOW],  // 0
    [LOW, HIGH, HIGH, LOW, LOW, LOW, LOW, LOW],      // 1
    [HIGH, HIGH, LOW, HIGH, HIGH, LOW, HIGH, LOW],   // 2
    [HIGH, HIGH, HIGH, HIGH, LOW, LOW, HIGH, LOW],   // 3
    [LOW, HIGH, HIGH, LOW, LOW, HIGH, HIGH, LOW],    // 4
    [HIGH, LOW, HIGH, HIGH, LOW, HIGH, HIGH, LOW],   // 5
    [HIGH, LOW, HIGH, HIGH, HIGH, HIGH, HIGH, LOW],  // 6
    [HIGH, HIGH, HIGH, LOW, LOW, LOW, LOW, LOW],     // 7
    [HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, LOW], // 8
    [HIGH, HIGH, HIGH, HIGH, LOW, HIGH, HIGH, LOW],  // 9
    [LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW],        // OFF
];

/// The RPM display is just 4 CL5611AH seven segment digits in linked 8-bit shift registers
/// Each shift register's outputs are in order of A-G and DP
pub struct RPMDisplay {
    shift_register: ShiftRegister<
        { 4 * 8 },
        pins::rpm_display::SerialIn,
        pins::rpm_display::Clock,
        pins::rpm_display::Latch,
    >,
}
impl RPMDisplay {
    pub fn new(
        shift_register: ShiftRegister<
            { DIGITS * 8 },
            pins::rpm_display::SerialIn,
            pins::rpm_display::Clock,
            pins::rpm_display::Latch,
        >,
    ) -> Self {
        Self { shift_register }
    }
}
impl Display for RPMDisplay {
    fn display(&mut self, state: &State) {
        debug!("[DEBUG] [RPM DISPLAY] Displaying!");
        for (idx, bit) in self.shift_register.bit_array.iter_mut().enumerate() {
            *bit = SEVEN_SEGMENT_OUTPUT[state.digits[idx / 8] as usize][idx % 8];
        }
        self.shift_register
            .set_bit_array(self.shift_register.bit_array);
    }
}
