use super::memory::*;

const PM_WDOG_MAGIC: u32 = 0x5A00_0000;
const PM_RSTC_FULLRST: u32 = 0x0000_0020;

#[allow(dead_code)]
pub fn board_reset() -> ! {
    let mut r = unsafe { core::ptr::read(board_pm_rsts()) };
    r &= !0xFFFF_FAAA;
    unsafe {
        core::ptr::write_volatile(board_pm_rsts() as *mut _, PM_WDOG_MAGIC | r);
        core::ptr::write_volatile(board_pm_wdog() as *mut _, PM_WDOG_MAGIC | 10);
        core::ptr::write_volatile(board_pm_rstc() as *mut _, PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    };
    // Should never be executed
    loop {}
}
