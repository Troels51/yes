
use core::alloc::{self, Layout};
use std::{os::raw::c_void, ptr::slice_from_raw_parts, slice::{from_raw_parts, from_raw_parts_mut}};

use libc::{F_SETPIPE_SZ, MADV_HUGEPAGE};
extern crate libc;

const N: usize = 2097152; // Huge page
const YES: [u8; 2] = *b"y\n";

fn main() {
    unsafe {
        let layout = Layout::from_size_align(N, N).unwrap();
        let ptr = std::alloc::alloc(layout);
        libc::madvise(ptr as *mut c_void, N, MADV_HUGEPAGE);
        let yes_array = from_raw_parts_mut(ptr, N);
        let mut i = 0;
        while i < yes_array.len() {
            yes_array[i] = YES[0];
            yes_array[i + 1] = YES[1];
            i += 2;
        }

        libc::fcntl(1, F_SETPIPE_SZ, N);
        let iovec = libc::iovec {
            iov_base: ptr as *mut c_void,
            iov_len: N,
        };
        loop {
            libc::vmsplice(1, &iovec, 1, 0);
        }
    }
}
