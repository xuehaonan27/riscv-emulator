/// The mhartid CSR is an MXLEN-bit READ-ONLY register containing the
/// integer ID of the hardware thread running the code. This register
/// must be readable in any implementation. Hart IDs might not necessarily
/// be numbered contiguously in a multiprocessor system, but at least
/// one hart must have a hart ID of zero. Hart IDs must be unique within
/// the execution environment.
#[repr(transparent)]
pub struct Mhartid(u64);

impl Mhartid {
    pub fn new(mhartid: u64) -> Mhartid {
        Mhartid(mhartid)
    }

    pub fn read(&self) -> u64 {
        self.0
    }
}
