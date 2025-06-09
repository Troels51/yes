
use core::alloc::{self, Layout};
use std::{ptr::slice_from_raw_parts, slice::{from_raw_parts, from_raw_parts_mut}};

use libc::F_SETPIPE_SZ;
extern crate libc;

const N: usize = 1024*1024; //times 2 bytes = 2 mb
const YES: [u8; 2] = *b"y\n";
unsafe fn create_yes_array() -> [u8; N] {
    let layout = Layout::from_size_align(N, N).unwrap();
    let ptr = std::alloc::alloc(layout);
    let res = from_raw_parts_mut(ptr, N);
    let mut i = 0;
    while i < res.len() {
        res[i] = YES[0];
        res[i + 1] = YES[1];
        i += 2;
    }
    res.try_into().unwrap()
}

fn main() {
    unsafe {
        let mut yes_array: [u8; N] = create_yes_array();

        libc::fcntl(1, F_SETPIPE_SZ, N);
        let iovec = libc::iovec {
            iov_base: yes_array.as_mut_ptr() as *mut core::ffi::c_void,
            iov_len: N,
        };
        loop {
            libc::vmsplice(1, &iovec, 1, libc::SPLICE_F_GIFT & libc::SPLICE_F_NONBLOCK);
        }
    }
}
