#![no_std]
use core::str;

// allocが使えない環境下での文字列操作関数
pub fn hex_ptr_to_usize(ptr: *const u8) -> Result<usize, ()> {
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(ptr, len);
        match slice_to_usize(slice) {
            Ok(value) => Ok(value),
            Err(_) => Err(()),
        }
    }
}

pub fn slice_to_usize(slice: &[u8]) -> Result<usize, ()> {
    let s = match str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);

    usize::from_str_radix(s, 16).map_err(|_| ())
}
