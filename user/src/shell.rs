use crate::putchar;

#[unsafe(no_mangle)]
fn main() -> ! {
    for c in b"Hello, World!" {
        putchar(*c as char);
    }

    loop {}
}
