#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::{Impl, RawRestoreState, set_impl};

const LOCKED: bool = true;
const UNLOCKED: bool = false;
static SPINLOCK: AtomicBool = AtomicBool::new(UNLOCKED);

struct MultiHartCriticalSection;
set_impl!(MultiHartCriticalSection);

unsafe impl Impl for MultiHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        // Disable interrupts on the current hart by clearing MIE in mstatus.
        let mut mstatus: usize;
        unsafe {
            core::arch::asm!("csrrci {}, mstatus, 0b1000", out(reg) mstatus);
        }
        let mie_set =
            unsafe { core::mem::transmute::<_, riscv::register::mstatus::Mstatus>(mstatus) }.mie();

        while let Err(_) =
            SPINLOCK.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
        {
            core::hint::spin_loop();
        }

        mie_set
    }

    unsafe fn release(was_active: RawRestoreState) {
        SPINLOCK.store(UNLOCKED, Ordering::Release);

        // Re-enable interrupts only if they were enabled before the critical section.
        if was_active {
            unsafe {
                riscv::interrupt::enable();
            }
        }
    }
}
