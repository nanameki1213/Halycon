use core::str;

// allocが使えない環境下での文字列操作関数
pub unsafe fn hex_ptr_to_usize(ptr: *const u8) -> Result<usize, ()> {

    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }

    let slice = core::slice::from_raw_parts(ptr, len);
    let s = match str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);

    usize::from_str_radix(s, 16).map_err(|_| ())
}

pub unsafe fn hex_ptr_to_usize_length(ptr: *const u8, len: usize) -> Result<usize, ()> {
    let slice = core::slice::from_raw_parts(ptr, len);
    let s = match str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);

    usize::from_str_radix(s, 16).map_err(|_| ())
}
