#[derive(Default, Debug)]
pub struct Reg {
    pub v: [u8; 16],
    pub i: u16,
    pub delay_timer: u8,
    pub sound_time: u8,
    pub pc: u16,
    pub sp: u8,
}
