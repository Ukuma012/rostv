#[macro_export]
macro_rules! write_csr {
    ($csr:expr, $value:expr) => {
        unsafe {
            use core::arch::asm;
            asm!(
                concat!("csrw ", $csr, ", {r}"), r = in(reg) $value
            )
        }
    }
}
