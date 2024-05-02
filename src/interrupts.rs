pub use millis::{millis, millis_init};
pub use sensor::{rpm, sensor_init};

use crate::{pins, shared::HISTORY_LEN};

use arduino_hal::pac::{
    exint::{eicra::EICRA_SPEC, eimsk::EIMSK_SPEC},
    TC0,
};
use avr_device::{
    generic::Reg,
    interrupt::{self, Mutex},
};
use core::cell::{Cell, RefCell};
use heapless::HistoryBuffer;

/// This millisecond interrupt was usurped from Rahix's amazing blog:
/// https://blog.rahix.de/005-avr-hal-millis/
mod millis {
    use super::*;

    const PRESCALER: u32 = 1024_u32;
    const TIMER_COUNTS: u32 = 250_u32;
    const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16_000_u32; // 16MHz

    static MILLIS_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0_u32));

    pub unsafe fn millis_init(tc0: TC0) {
        // Configure the timer for the above interval (in CTC mode)
        // and enable its interrupt
        tc0.tccr0a.write(|w| w.wgm0().ctc());
        tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
        tc0.tccr0b.write(|w| match PRESCALER {
            8_u32 => w.cs0().prescale_8(),
            64_u32 => w.cs0().prescale_64(),
            256_u32 => w.cs0().prescale_256(),
            1024_u32 => w.cs0().prescale_1024(),
            _ => panic!(),
        });
        tc0.timsk0.write(|w| w.ocie0a().set_bit());

        // Reset the global millisecond counter
        interrupt::free(|critical_section| {
            MILLIS_COUNTER.borrow(critical_section).set(0_u32);
        });
    }

    #[avr_device::interrupt(atmega328p)]
    #[allow(non_snake_case)]
    fn TIMER0_COMPA() {
        interrupt::free(|critical_section| {
            let counter_cell = MILLIS_COUNTER.borrow(critical_section);
            let counter = counter_cell.get();
            counter_cell.set(counter + MILLIS_INCREMENT);
        })
    }

    pub fn millis() -> u32 {
        interrupt::free(|critical_section| MILLIS_COUNTER.borrow(critical_section).get())
    }
}

mod sensor {
    use crate::{
        rpm::RpmInfo,
        shared::{Milliseconds, MAX_DELTATIME},
    };

    use super::*;

    static SENSOR_HISTORY: Mutex<RefCell<HistoryBuffer<u32, HISTORY_LEN>>> =
        Mutex::new(RefCell::new(HistoryBuffer::new()));
    static RPM: Mutex<RefCell<RpmInfo>> = Mutex::new(RefCell::new(RpmInfo {
        oldest_sensor_hit_time: 0,
        count: 0,
    }));

    /// Safety note: The caller must ensure that the sensor pin is INT 0 / PCINT 18!
    pub unsafe fn sensor_init(
        eicra: &Reg<EICRA_SPEC>,
        eimsk: &Reg<EIMSK_SPEC>,
        _: &pins::sensor::Sensor,
    ) {
        // Enable mask 0 interrupt
        eicra.modify(|_, w| w.isc0().bits(0x02));
        eimsk.modify(|_, w| w.int0().set_bit());

        // Clear the sensor history (currently filled with garbage).
        // `new_with` isn't constant, so we need to do this at runtime
        interrupt::free(|critical_section| {
            let history_ref = SENSOR_HISTORY.borrow(critical_section);
            let mut history = history_ref.borrow_mut();
            for _ in 0..HISTORY_LEN {
                history.write(0);
            }
        })
    }

    #[avr_device::interrupt(atmega328p)]
    fn INT0() {
        let time = millis();
        crate::console::debug!("INT");
        let (count, oldest_sensor_hit_time) = interrupt::free(
            |critical_section: interrupt::CriticalSection<'_>| -> (usize, Milliseconds) {
                let history_ref = SENSOR_HISTORY.borrow(critical_section);
                let mut history = history_ref.borrow_mut();
                history.write(time);
                history
                    .oldest_ordered()
                    .enumerate()
                    .find(|(_, past)| {
                        **past != 0 && ((time - **past) <= MAX_DELTATIME) && (time != **past)
                    })
                    .map_or((0, 0), |x| (HISTORY_LEN - x.0, *x.1))
            },
        );
        interrupt::free(|critical_section| {
            let rpm_cell = RPM.borrow(critical_section);
            let mut rpm_info = rpm_cell.borrow_mut();
            rpm_info.oldest_sensor_hit_time = oldest_sensor_hit_time;
            rpm_info.count = count;
        })
    }

    pub fn rpm() -> RpmInfo {
        interrupt::free(|critical_section| {
            let rpm_cell = RPM.borrow(critical_section);
            let RpmInfo {
                ref oldest_sensor_hit_time,
                ref count,
            } = &*rpm_cell.borrow();
            RpmInfo {
                oldest_sensor_hit_time: *oldest_sensor_hit_time,
                count: *count,
            }
        })
    }
}
