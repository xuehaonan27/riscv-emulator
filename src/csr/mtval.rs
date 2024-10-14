#[repr(transparent)]
pub struct Mtval(u64);

impl Mtval {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
