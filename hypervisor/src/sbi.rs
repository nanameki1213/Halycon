#![allow(dead_code)]

use core::sync::atomic::Ordering;

use crate::CNT_FLUSH_NOTIFY;
use crate::CNT_L2_PF_UART;
use crate::CNT_REFLECT_L1_TO_L2;
use crate::CNT_REFLECT_L2_TO_L1;
use crate::mmio::ns16550::putc;
use crate::print;
use crate::println;

pub const SBI_EXT_BASE: usize = 0x10;
pub const SBI_EXT_0_1_CONSOLE_PUTCHAR: usize = 1;
pub const SBI_EXT_DBCN: usize = 0x4442434E;
pub const SBI_EXT_SRST: usize = 0x53525354;

pub const SBI_FID_GET_SBI_SPECIFICATION_VERSION: usize = 0;
pub const SBI_FID_GET_SBI_IMPLEMENTATION_ID: usize = 1;
pub const SBI_FID_GET_SBI_IMPLEMENTATION_VERSION: usize = 2;
pub const SBI_FID_PROBE_SBI_EXT: usize = 3;
pub const SBI_FID_GET_MACHINE_VENDOR_ID: usize = 4;
pub const SBI_FID_GET_MACHINE_ARCHITECTURE_ID: usize = 5;
pub const SBI_FID_GET_MACHINE_IMPLEMENTATION_ID: usize = 6;

pub const SBI_EXT_DBCN_CONSOLE_WRITE: usize = 0;
pub const SBI_EXT_DBCN_CONSOLE_READ: usize = 1;
pub const SBI_EXT_DBCN_CONSOLE_WRITE_BYTE: usize = 2;

const SBI_VERSION_MAJOR_OFFSET: usize = 24;

pub struct Sbiret {
    pub error: u64,
    pub value: u64,
}

pub fn virtual_sbi(
    ext: usize,
    fid: usize,
    _arg0: usize,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
) -> Sbiret {
    match ext {
        SBI_EXT_BASE => match fid {
            SBI_FID_GET_SBI_SPECIFICATION_VERSION => Sbiret {
                error: 0,
                value: 3 << SBI_VERSION_MAJOR_OFFSET,
            },
            SBI_FID_GET_SBI_IMPLEMENTATION_ID => Sbiret { error: 0, value: 0 },
            SBI_FID_GET_SBI_IMPLEMENTATION_VERSION => Sbiret { error: 0, value: 2 },
            SBI_FID_PROBE_SBI_EXT => Sbiret { error: 0, value: 0 },
            SBI_FID_GET_MACHINE_VENDOR_ID => Sbiret { error: 0, value: 0 },
            SBI_FID_GET_MACHINE_ARCHITECTURE_ID => Sbiret { error: 0, value: 0 },
            SBI_FID_GET_MACHINE_IMPLEMENTATION_ID => {
                CNT_L2_PF_UART.store(0, Ordering::Release);
                CNT_REFLECT_L2_TO_L1.store(0, Ordering::Release);
                CNT_REFLECT_L1_TO_L2.store(0, Ordering::Release);
                CNT_FLUSH_NOTIFY.store(0, Ordering::Release);
                println!("\nstart measure.");

                Sbiret { error: 0, value: 0 }
            }
            _ => {
                println!("SBI_EXT_BASE: fid: {}", fid);
                panic!("unrecognized fid");
            }
        },
        SBI_EXT_0_1_CONSOLE_PUTCHAR => {
            if fid == 0 {
                putc(_arg0 as u8);
                if let Some(ch) = char::from_u32(_arg0 as u32) {
                    if ch == '\n' {
                        print!("[L1 VM] ");
                    }
                }
                Sbiret { error: 0, value: 0 }
            } else {
                println!("fid: {}", fid);
                panic!("unrecognized fid");
            }
        }
        SBI_EXT_DBCN => {
            if fid == SBI_EXT_DBCN_CONSOLE_WRITE_BYTE {
                putc(_arg0 as u8);
                Sbiret { error: 0, value: 0 }
            } else {
                println!("SBI_EXT_DBCN_CONSOLE_WRITE_BYTE: fid: {}", fid);
                panic!("unrecognized fid");
            }
        }
        SBI_EXT_SRST => {
            println!("End of measure.");
            println!("cnt_l2_pf_uart: {}", CNT_L2_PF_UART.load(Ordering::Acquire));
            println!(
                "cnt_reflect_l2_to_l1: {}",
                CNT_REFLECT_L2_TO_L1.load(Ordering::Acquire)
            );
            println!(
                "cnt_reflect_l1_to_l2: {}",
                CNT_REFLECT_L1_TO_L2.load(Ordering::Acquire)
            );
            println!(
                "cnt_flush_notify: {}",
                CNT_FLUSH_NOTIFY.load(Ordering::Acquire)
            );
            panic!();
        }
        _ => {
            println!("eid: {}", ext);
            panic!("unrecognized eid")
        }
    }
}
