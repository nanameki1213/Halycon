use crate::mmio::ns16550::putc;
use core::fmt;

pub struct Console();

pub static mut DEFAULT_CONSOLE: Console = Console::new();

impl Console {
    pub const fn new() -> Self {
        Console()
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            putc(*c);
        }
        return Ok(());
    }
}

pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    let result = unsafe { DEFAULT_CONSOLE.write_fmt(args) };
    if result.is_err() {
        panic!("write_fmt was failed.");
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::console::print(format_args!("{}\n", format_args!($fmt))));
    ($fmt:expr, $($arg:tt)*) => ($crate::console::print(format_args!("{}\n", format_args!($fmt, $($arg)*))));
}
