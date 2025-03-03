#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::{Impl, RawRestoreState, set_impl};

const UNLOCKED: bool = false;
const LOCKED: bool = true;
static SPINLOCK: AtomicBool = AtomicBool::new(UNLOCKED);

#[cfg(debug_assertions)]
static COUNTER: core::sync::atomic::AtomicIsize = core::sync::atomic::AtomicIsize::new(0);

struct MultiHartCriticalSection;
set_impl!(MultiHartCriticalSection);

unsafe impl Impl for MultiHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        // Disable interrupts on the current hart by clearing MIE in mstatus.
        let mut mstatus: usize;
        unsafe {
            core::arch::asm!("csrrci {}, mstatus, 0b1010", out(reg) mstatus);
        }

        while let Err(_) =
            SPINLOCK.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
        {
            core::hint::spin_loop();
        }

        #[cfg(debug_assertions)]
        {
            debug_assert!(COUNTER.load(Ordering::SeqCst) == 0);
            COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        mstatus
    }

    unsafe fn release(mstatus: RawRestoreState) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(COUNTER.load(Ordering::SeqCst) == 1);
            COUNTER.fetch_sub(1, Ordering::SeqCst);
        }

        SPINLOCK.store(UNLOCKED, Ordering::Release);
        let imm = mstatus & 0b1111;

        match imm {
            0b1010 => unsafe {
                core::arch::asm!("csrrsi {0}, mstatus, 0b1010", in(reg) mstatus);
            },
            0b1000 => unsafe {
                core::arch::asm!("csrrsi {0}, mstatus, 0b1000", in(reg) mstatus);
            },
            0b0010 => unsafe {
                core::arch::asm!("csrrsi {0}, mstatus, 0b0010", in(reg) mstatus);
            },
            0b0000 => unsafe {
                core::arch::asm!("csrrsi {0}, mstatus, 0b0000", in(reg) mstatus);
            },
            _ => panic!("Didn't match any in critical section!"),
        }
    }
}
