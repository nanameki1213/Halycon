use arch::riscv::cpu::*;

pub const TIMER_FRQ: usize = 10000;

pub fn enable_timer_intr() {
    set_sie(get_sie() | XIE_STIE as u64);
}

pub fn disable_timer_intr() {
    set_sie(get_sie() & !(XIE_STIE as u64));
}

pub fn start_timer(value: u64) {
    set_stimecmp(get_time().wrapping_add(value));
    enable_timer_intr();
}
