use core::slice;

#[repr(C)]
#[derive(Debug)]
pub struct VirtioBlkConfig {
    pub capacity: u64,
    pub size_max: u32,
    pub seg_max: u32,
    // TODO: there are more field, buf we aren't supprting now.
}

impl VirtioBlkConfig {
    pub const fn new() -> Self {
        Self {
            capacity: 0,
            size_max: 0,
            seg_max: 0,
        }
    }

    pub fn read_as_bytes(&self, offset: usize) -> u32 {
        let struct_size = core::mem::size_of::<Self>();

        if offset >= struct_size {
            return 0;
        }

        let bytes =
            unsafe { slice::from_raw_parts((self as *const Self) as *const u8, struct_size) };

        let mut result_bytes = [0u8; 4];
        for i in 0..4 {
            if offset + 1 < struct_size {
                result_bytes[i] = bytes[offset + i];
            }
        }

        u32::from_le_bytes(result_bytes)
    }
}
