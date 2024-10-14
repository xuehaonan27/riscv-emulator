#[repr(transparent)]
pub struct Sscratch(u64);

impl Sscratch {
    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
