#[repr(transparent)]
pub struct Sepc(u64);

impl Sepc {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
