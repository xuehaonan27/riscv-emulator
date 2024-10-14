#[repr(transparent)]
pub struct Mepc(u64);

impl Mepc {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
