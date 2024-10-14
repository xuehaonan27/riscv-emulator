/// The misa CSR is a WARL read-write register reporting the ISA supported by the hart. This register must be readable in any implementation, but a value of zero can be returned to indicate the misa register has not been implemented, requiring that CPU capabilities be determined through a separate non-standard mechanism.
#[repr(transparent)]
pub struct Misa(u64);

impl Misa {
    pub fn new_with_mxl(mxl: u64) -> Misa {
        todo!()
    }
}