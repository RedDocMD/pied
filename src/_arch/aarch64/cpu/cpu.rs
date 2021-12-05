use cortex_a::asm;

pub use asm::nop;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}

#[cfg(feature = "bsp_rpi3")]
#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}
