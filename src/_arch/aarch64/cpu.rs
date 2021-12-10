use cortex_a::asm;

pub use asm::nop;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}
