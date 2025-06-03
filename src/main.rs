
#![no_main]
#![no_std]

use libc::F_SETPIPE_SZ;
extern crate libc;

const N: usize = 1024*1024;
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


#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    unsafe {
        libc::fcntl(1, F_SETPIPE_SZ, N);
        let iovec = libc::iovec {
            iov_base: YES_ARRAY.as_mut_ptr() as *mut core::ffi::c_void,
            iov_len: N,
        };
        loop {
            libc::vmsplice(1, &iovec, 1, libc::SPLICE_F_GIFT & libc::SPLICE_F_NONBLOCK);
        }
    }
}


#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}