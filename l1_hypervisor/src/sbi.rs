use core::arch::asm;

const SBI_EXT_0_1_CONSOLE_PUTCHAR: usize = 0x01;
const SBI_EXT_0_1_CONSOLE_GETCHAR: usize = 0x02;

#[repr(C)]
pub struct Sbiret {
    pub error: usize,
    pub value: usize,
}

pub fn sbi_ecall(
    ext: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> Sbiret {
    let mut a0 = arg0;
    let mut a1 = arg1;

    unsafe {
        asm!(
            "ecall",

            inout("a0") a0,
            inout("a1") a1,

            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") ext,

            options(nostack, preserves_flags, nomem)
         );
    }

    Sbiret {
        error: a0,
        value: a1,
    }
}

pub fn sbi_console_putchar(ch: u8) {
    sbi_ecall(SBI_EXT_0_1_CONSOLE_PUTCHAR, 0, ch as usize, 0, 0, 0, 0, 0);
}

pub fn sbi_console_getchar() -> usize {
    let ret = sbi_ecall(SBI_EXT_0_1_CONSOLE_GETCHAR, 0, 0, 0, 0, 0, 0, 0);

    ret.error
}
