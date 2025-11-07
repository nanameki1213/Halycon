use crate::println;
use crate::mmio::ns16550::putc;

pub const SBI_EXT_BASE: usize = 0x10;
pub const SBI_EXT_0_1_CONSOLE_PUTCHAR: usize = 1;
pub const SBI_EXT_DBCN: usize = 0x4442434E;

pub const SBI_FID_GET_SBI_IMPLEMENTATION_VERSION: usize = 2;
pub const SBI_FID_PROBE_SBI_EXT: usize = 3;
pub const SBI_FID_GET_MACHINE_VENDER_ID: usize = 4;

pub const SBI_EXT_DBCN_CONSOLE_WRITE: usize = 0;
pub const SBI_EXT_DBCN_CONSOLE_READ: usize = 1;
pub const SBI_EXT_DBCN_CONSOLE_WRITE_BYTE: usize = 2;


pub struct Sbiret {
    pub error: u64,
    pub value: u64,
}

pub fn virtual_sbi(
    ext: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> Sbiret {
    match ext {
        SBI_EXT_BASE => {
            match fid {
                SBI_FID_PROBE_SBI_EXT => {
                    Sbiret {
                        error: 0,
                        value: 1,
                    }
                }
                SBI_FID_GET_SBI_IMPLEMENTATION_VERSION => {
                    Sbiret {
                        error: 0,
                        value: 2,
                    }
                }
                SBI_FID_GET_MACHINE_VENDER_ID => {
                    Sbiret {
                        error: 0,
                        value: 0,
                    }
                }
                _ => {
                    println!("SBI_EXT_BASE: fid: {}", fid);
                    panic!("unrecognized fid");
                }
            }
        },
        SBI_EXT_0_1_CONSOLE_PUTCHAR => {
            if fid == 0 {
                putc(arg0 as u8);
                Sbiret {
                    error: 0,
                    value: 0,
                }
            } else {
                println!("fid: {}", fid);
                panic!("unrecognized fid");
            }
        },
        SBI_EXT_DBCN => {
            if fid == SBI_EXT_DBCN_CONSOLE_WRITE_BYTE {
                putc(arg0 as u8);
                Sbiret {
                    error: 0,
                    value: 0,
                }
            } else {
                println!("SBI_EXT_DBCN_CONSOLE_WRITE_BYTE: fid: {}", fid);
                panic!("unrecognized fid");
            }
        }
        _ => {
            println!("eid: {}", ext);
            panic!("unrecognized eid")
        }
    }
    
}
