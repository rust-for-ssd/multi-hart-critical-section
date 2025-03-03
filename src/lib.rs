#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};
use critical_section::{Impl, RawRestoreState, set_impl};

const UNLOCKED: usize = usize::max_value();
static SPINLOCK: AtomicUsize = AtomicUsize::new(UNLOCKED);

struct MultiHartCriticalSection;
set_impl!(MultiHartCriticalSection);

unsafe impl Impl for MultiHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        // Disable interrupts on the current hart by clearing MIE in mstatus.
        let mut mstatus: usize;
        unsafe {
            core::arch::asm!("csrrci {}, mstatus, 0b1010", out(reg) mstatus);
        }
        let mstatus =
            unsafe { core::mem::transmute::<_, riscv::register::mstatus::Mstatus>(mstatus) }.bits();
        let hart_id = riscv::register::mhartid::read();
        if hart_id != SPINLOCK.load(Ordering::SeqCst) {
            while let Err(_) =
                SPINLOCK.compare_exchange(UNLOCKED, hart_id, Ordering::SeqCst, Ordering::Relaxed)
            {
                // core::hint::spin_loop();
            }
        }

        mstatus
    }

    unsafe fn release(mstatus: RawRestoreState) {
        let hart_id = riscv::register::mhartid::read();
        let _ = SPINLOCK.compare_exchange(hart_id, UNLOCKED, Ordering::SeqCst, Ordering::Relaxed);

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
