---
title: Fastest yes in the west
author: Troels Hoffmeyer
theme:
  override:
    default:
      margin:
        percent: 10
    slide_title:
      padding_top: 2
    code:
      alignment: left
      background: true
  
---


Background
===
# The start of the quest
 - Was reading hackernews one day
 - Saw uutils/coreutils/yes
 - People were saying that yes is suprisingly fast
 - Ergo, i had to check if it can be done faster

## What is `yes`
A linux command that will print "y\n" to standard out as fast as possible.
It can also print an argument you give it, but we will ignore it for now, to simplify a lot of code

```bash +exec
yes | head -n 10
```
<!-- pause -->
I use 'pv' to test the speed of it

```nushell +exec
yes | pv  -q -v --stop-at-size --size 10737418240 o> /dev/null e>| awk '{split($4, a, "/"); print a[2]/1073741824 " avg Gib/s"}'
```
<!-- end_slide -->

Naive implementation.
===

Let's just println 'y' with the println macro
# The code

```rust +line_numbers
fn main() {
    loop {
        println!("y");
    }
}
```
# Test it

```nushell +exec
~/Projects/yes/target/release/naive_yes |  pv  -q -v --stop-at-size --size 10485760 o> /dev/null e>| awk '{split($4, a, "/"); print a[2]/1073741824 " avg Gib/s"}'
```
<!-- end_slide -->

Using io::stdout directly
===
We can use std::io::stdout directly and lock it before we begin using it.
# The code
```rust +line_numbers
use std::io::{self, Write};

fn main() -> io::Result<()>{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    loop {
        let _ = handle.write_all(b"y\n");
    }
}
```
# Test it

```nushell +exec
~/Projects/yes/target/release/direct_yes |  pv  -q -v --stop-at-size --size 104857600 o> /dev/null e>| awk '{split($4, a, "/"); print a[2]/1073741824 " avg Gib/s"}'
```

<!-- end_slide -->

Buffering
===
Let's print a whole buffer, instead. 1024 bytes is what the default stdout buffer size is

uutils yes is using 16*1024 bytes, with a comment about some systems might be faster with a different size.

I have found 1024 bytes to work best. (While writing this i wonder why this is the number and not 16kb, which is the default pipe size, or 4kb which is the page size)
# The code
```rust +line_numbers
use std::io::{self, Write};

fn main() -> io::Result<()>{
    const BUFFER_SIZE: usize =  1024;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let buffer = b"y\n".repeat(BUFFER_SIZE/2);
    loop {
        handle.write_all(buffer.as_slice())?;
    }
}
```

# Test it

```nushell +exec
~/Projects/yes/target/release/buffer_yes |  pv  -q -v --stop-at-size --size 10737418240 o> /dev/null e>| awk '{split($4, a, "/"); print a[2]/1073741824 " avg Gib/s"}'
```
<!-- end_slide -->

Let's optimize the build
===
# cargo wizard
A "wizard" for generating cargo configs to create optimized builds
```nushell
cargo install cargo-wizard
cargo wizard
```

```nushell
? Select the profile that you want to update/create:
> dev (builtin)
  release (builtin)
  <Create a new profile>
[↑↓ to move, enter to select, type to filter]
```

```nushell
> Select the profile that you want to update/create: release (builtin)
? Select the template that you want to apply:
> FastCompile: minimize compile times
  FastRuntime: maximize runtime performance
  MinSize: minimize binary size
[↑↓ to move, enter to select, type to filter]
```
``` nushell
> Select the profile that you want to update/create: release (builtin)
> Select the template that you want to apply: FastRuntime: maximize runtime performance
? Select items to modify or confirm the template:
> <Confirm>
  Optimization level                   [3]
  Link-time optimizations           [true]
  Number of codegen units (CGUs)       [1]
  Target CPU instruction set      [native]
  Panic handling mechanism         [abort]
  Debug info                       [false]
  Split debug info                       -
  Strip symbols                     [none]
  Incremental compilation          [false]
  Linker ^                               -
v Codegen backend *                      -
[↑↓ to move, enter to select, type to filter. * Requires nightly compiler ^ Requires Unix]
```
<!-- end_slide -->

Cargo config
===
Those optimization flags resulted in
# .cargo/config.toml
```toml
[build]
rustflags = ["-Ctarget-cpu=native"]
```
# Cargo.toml
```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
```
<!-- end_slide -->

Minimize binary
===
Now for a detour, to see how `small` we can make yes.
# Normal build size
``` nushell +exec
cargo build --release
ls -la ~/Projects/yes/target/release/buffer_yes  | select name size
```
<!-- pause -->
# minimized build size build size
With a bunch of rust and cargo flags we
- Remove fmt::Debug
- Remove location detail for panics
- Remove panic string formatting with immediate_abort
```nushell +exec
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release

ls -la ~/Projects/yes/target/release/buffer_yes  | select name size
```
<!-- new_lines: 5 -->
ref:
- https://github.com/johnthagen/min-sized-rust
<!-- end_slide -->

Use no_main
===
By using #![no_main] we can remove core::fmt
```rust +line_numbers

#![no_main]
use std::io::{self, Write};
use std::fs::File;
use std::os::unix::io::FromRawFd;

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

fn stdout() -> File {
    unsafe { File::from_raw_fd(1) }
}

#[unsafe(no_mangle)]
pub fn main(_argc: i32, _argv: *const *const u8) {
    let array = create_yes_array();
    let mut stdout = stdout();
    loop {
        let _ = stdout.write(&array);
    }
}
```

```nushell +exec
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release

ls -la ~/Projects/yes/target/release/no_main_yes  | select name size
```

<!-- end_slide -->

Direct asm syscall
===
To shave a couple of bytes off, we can directly do the write syscall in asm
```rust +line_numbers
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
            "syscall", // syscall x86_64 instruction
            inlateout("rax") 1usize => ret, // "1" is the write command, and this register has the result
            in("rdi") fd, // filedescriptor, with 1 being stdout
            in("rsi") buffer.as_ptr(),
            in("rdx") count,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[unsafe(no_mangle)]
pub fn main(_argc: i32, _argv: *const *const u8) {
    let yes_array = create_yes_array();
    unsafe {
        loop {
            let _ = write(1, &yes_array, N);
        }
    }
}
```

```nushell +exec
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release

ls -la ~/Projects/yes/target/release/asm_yes  | select name size
```
<!-- end_slide -->

Use no_std
===
Let's try to remove the use of the standard library, that should yield a couple of bytes.
Differences are:
- #![no_std]
- extern crate libc
- extern "C" main
- A panic handler

```rust +line_numbers

#![no_main]
#![no_std]
use core::arch::asm;
extern crate libc;

// removed create_yes_array() and write()

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let yes_array = create_yes_array();
    unsafe {
        loop {
            let _ = write(1, &yes_array, N);
        }
    }
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

```nushell +exec
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release

ls -la ~/Projects/yes/target/release/no_std_asm_yes  | select name size
```
<!-- end_slide -->

Using vmsplice
===
Next approach uses improvements based on: < [How fast are linux pipes anyway?](https://mazzo.li/posts/fast-pipes.html) >
It is a bit of "Volkswagen" situation, as `yes` will not actually output to standard out anymore
- Allocate a 1Mib aligned piece of memory
- Set the pipe size to be 1Mib
- Use vmsplice to allow the kernel to directly use our buffers
```rust +line_numbers

use core::alloc::Layout;
use std::{os::raw::c_void, slice::from_raw_parts_mut};

use libc::{F_SETPIPE_SZ};
extern crate libc;

const N: usize = 1024*1024; // 1Mib
const YES: [u8; 2] = *b"y\n";

fn main() {
    unsafe {
        let layout = Layout::from_size_align(N, N).unwrap();
        let ptr = std::alloc::alloc(layout);
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
        while libc::vmsplice(1, &iovec, 1, 0) != -1 {}

    }
}
```
# Test it

```nushell +exec
~/Projects/yes/target/release/vmsplice_yes |  pv  -q -v --stop-at-size --size 107374182400 o> /dev/null e>| awk '{split($4, a, "/"); print a[2]/1073741824 " avg Gib/s"}'
```
<!-- end_slide -->
Huge Pages
===
How do i get this to work???
# Code
```rust +line_numbers {6, 13}
use core::alloc::Layout;
use std::{os::raw::c_void, slice::from_raw_parts_mut};
use libc::{F_SETPIPE_SZ, MADV_HUGEPAGE};
extern crate libc;

const N: usize = 2*1024*1024; // 2Mib
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
        while libc::vmsplice(1, &iovec, 1, 0) != -1 {}

    }
}
```
# Test
```nushell +exec
~/Projects/yes/target/release/huge_page_yes |  pv  -q -v --stop-at-size --size 10737418240 o> /dev/null e>| awk '{split($4, a, "/"); print a[2]/1073741824 " avg Gib/s"}'
```
<!-- pause -->
# Not enabled on my linux
```nushell +exec
cat /sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages

cat /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages
```

<!-- end_slide -->

Ideas for improvements
===
- Custom linker script to reduce size even more
- 1Gib huge page
- io_uring????
- a yes syscall?