
#![no_main]
use core::arch::asm;


const N: usize = 1024;
const YES: [u8; 2] = *b"y\n";
const fn create_yes_array() -> [u8; N] {
    let mut res = [0; N];
    let mut i = 0;
    while i < res.len() {
        res[i] = YES[0];
        res[i + 1] = YES[1];
        i += 2;
    }
    res
}
const YES_ARRAY: [u8; N] = create_yes_array();

// From https://github.com/jasonwhite/syscalls/blob/main/src/syscall/x86_64.rs
// On x86-64, the following registers are used for args 1-6:
// arg1: %rdi
// arg2: %rsi
// arg3: %rdx
// arg4: %r10
// arg5: %r8
// arg6: %r9
//
// rax is used for both the syscall number and the syscall return value.

#[inline]
pub unsafe fn write(fd: usize, buffer: &[u8; N], count: usize) -> usize {
    let mut ret: usize;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") 1usize => ret,
            in("rdi") fd,
            in("rsi") buffer.as_ptr(),
            in("rdx") count,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
    ret
}


#[unsafe(no_mangle)]
pub fn main(_argc: i32, _argv: *const *const u8) {
    unsafe {
        loop {
            let _ = write(1, &YES_ARRAY, N);
        }
    }
}
