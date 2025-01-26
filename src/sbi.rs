use crate::println;

pub const SBI_EXT_BASE: u64 = 0x10;

pub const SBI_FID_GET_SBI_IMPLEMENTATION_VERSION: u64 = 2;
pub const SBI_FID_PROBE_SBI_EXT: u64 = 3;
pub const SBI_FID_GET_MACHINE_VENDER_ID: u64 = 4;

pub struct Sbiret {
    pub error: u64,
    pub value: u64,
}

pub fn virtual_sbi(sbi_ret: &mut Sbiret, eid: u64, fid: u64) {
    match fid {
        SBI_FID_PROBE_SBI_EXT => {
            if eid == SBI_EXT_BASE {
                sbi_ret.value = 1;
            }
        },
        SBI_FID_GET_SBI_IMPLEMENTATION_VERSION => {
            sbi_ret.value = 2;
        },
        SBI_FID_GET_MACHINE_VENDER_ID => {
            sbi_ret.value= 0;
        },
        _ => {
            println!("fid: {}", fid);
            panic!("unrecognized fid");
        }
    }
}
