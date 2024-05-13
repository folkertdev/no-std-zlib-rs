#![no_main]
#![no_std]

use core::{
    cell::UnsafeCell,
    ffi::{c_uint, c_void},
    sync::atomic::{AtomicUsize, Ordering},
};

use defmt_rtt as _; // global logger

const INPUT: &str = "hello world";

static USED: AtomicUsize = AtomicUsize::new(0);

struct Buffer<const N: usize>(UnsafeCell<[u8; N]>);
static BUFFER: Buffer<{ 32 * 1024 }> = Buffer::new();

impl<const N: usize> Buffer<N> {
    const fn new() -> Self {
        Self(UnsafeCell::new([0; N]))
    }
}

unsafe impl<const N: usize> Sync for Buffer<N> {}

pub unsafe extern "C" fn zalloc_c(opaque: *mut c_void, items: c_uint, size: c_uint) -> *mut c_void {
    use core::sync::atomic::Ordering;

    let _ = opaque;

    let start = USED.fetch_add((items * size) as usize, Ordering::Relaxed);

    let ptr = unsafe { BUFFER.0.get().cast::<u8>().add(start) as *mut c_void };

    ptr
}

/// # Safety
///
/// The `ptr` must be allocated with the allocator that is used internally by `zcfree`
pub unsafe extern "C" fn zfree_c(opaque: *mut c_void, ptr: *mut c_void) {
    let _ = opaque;

    // no-op
    let _ = ptr;
}

#[cortex_m_rt::entry]
fn main() -> ! {
    do_the_thing();
    exit();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    exit()
}

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

fn do_the_thing() {
    use libz_rs_sys::*;

    let mut stream = libz_rs_sys::z_stream {
        next_in: core::ptr::null_mut(),
        avail_in: 0,
        total_in: 0,
        next_out: core::ptr::null_mut(),
        avail_out: 0,
        total_out: 0,
        msg: core::ptr::null_mut(),
        state: core::ptr::null_mut(),
        zalloc: Some(zalloc_c),
        zfree: Some(zfree_c),
        opaque: core::ptr::null_mut(),
        data_type: 0,
        adler: 0,
        reserved: 0,
    };

    let err = unsafe {
        deflateInit2_(
            &mut stream,
            1,
            Z_DEFLATED,
            10,
            1,
            Z_FILTERED,
            zlibVersion(),
            core::mem::size_of::<z_stream>() as _,
        )
    };

    assert_eq!(err, Z_OK);
    defmt::println!("done with init");

    stream.next_in = INPUT.as_ptr() as *const u8 as *mut u8;
    let mut next_out = [0u8; 1236];
    stream.next_out = next_out.as_mut_ptr();

    stream.avail_in = INPUT.len() as _;
    stream.avail_out = next_out.len() as _;

    let err = unsafe { deflate(&mut stream, Z_FINISH) };
    assert_eq!(err, Z_STREAM_END);

    let err = unsafe { deflateEnd(&mut stream) };
    assert_eq!(err, Z_OK);

    let deflated = &next_out[..stream.total_out as usize];

    defmt::println!("deflated {}", deflated);

    // reset the allocator
    USED.store(0, Ordering::Relaxed);

    let mut stream = libz_rs_sys::z_stream {
        next_in: core::ptr::null_mut(),
        avail_in: 0,
        total_in: 0,
        next_out: core::ptr::null_mut(),
        avail_out: 0,
        total_out: 0,
        msg: core::ptr::null_mut(),
        state: core::ptr::null_mut(),
        zalloc: Some(zalloc_c),
        zfree: Some(zfree_c),
        opaque: core::ptr::null_mut(),
        data_type: 0,
        adler: 0,
        reserved: 0,
    };

    let err = unsafe {
        inflateInit2_(
            &mut stream,
            10,
            zlibVersion(),
            core::mem::size_of::<z_stream>() as _,
        )
    };
    assert_eq!(err, Z_OK);

    stream.next_in = deflated.as_ptr() as *mut u8;
    stream.avail_in = deflated.len() as _;

    let mut output = [0; 32];

    stream.next_out = output.as_mut_ptr();
    stream.avail_out = output.len() as _;

    let err = unsafe { inflate(&mut stream, Z_FINISH) };
    assert_eq!(err, Z_STREAM_END);

    let err = unsafe { inflateEnd(&mut stream) };
    assert_eq!(err, Z_OK);

    let inflated = &output[..stream.total_out as usize];

    assert_eq!(inflated, INPUT.as_bytes());

    defmt::println!("It worked, we got our input back out!");
}
