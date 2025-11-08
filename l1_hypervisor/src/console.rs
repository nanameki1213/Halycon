use crate::sbi::sbi_console_putchar;
use core::fmt;
use spin::Mutex;

pub struct Console();

pub static DEFAULT_CONSOLE: Mutex<Console> = Mutex::new(Console::new());

impl Console {
    pub const fn new() -> Self {
        Console()
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            sbi_console_putchar(*c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    let result = DEFAULT_CONSOLE.lock().write_fmt(args);
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
