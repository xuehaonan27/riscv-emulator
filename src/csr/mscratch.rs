#[repr(transparent)]
pub struct Mscratch(u64);

impl Mscratch {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
