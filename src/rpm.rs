use crate::shared::Milliseconds;

#[repr(C)]
pub struct RpmInfo {
    pub oldest_sensor_hit_time: Milliseconds,
    pub count: usize,
}

pub fn calculate_rpm(current_time: Milliseconds, info: &RpmInfo) -> u32 {
    let deltatime = current_time - info.oldest_sensor_hit_time;
    if deltatime == current_time {
        return 0;
    }
    ((60_000. * <u16 as TryInto<f32>>::try_into(info.count as u16).unwrap())
        / <u16 as TryInto<f32>>::try_into(deltatime as u16).unwrap()) as u32
}
