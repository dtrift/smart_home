//! Demonstrates the smart socket library linked **statically**.
//!
//! `smart_socket_ffi` is a Cargo path dependency, so its `rlib` artifact is
//! linked into this binary at compile time — the library code lives inside the
//! executable, with no runtime dependency on a `.so`/`.dll`.
//!
//! The functions called below are the crate's C ABI entry points
//! (`#[no_mangle] extern "C"`), the same symbols a C program would use against
//! the `staticlib` (`.a`) artifact.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use smart_socket_ffi::SmartSocket;

fn main() {
    println!("=== Static linking demo (rlib) ===\n");

    // Library version (static C string, do not free).
    let version_raw: *const c_char = smart_socket_ffi::smart_socket_version();
    // SAFETY: `smart_socket_version` returns a valid, NUL-terminated static string.
    let version = unsafe { CStr::from_ptr(version_raw) }
        .to_str()
        .expect("version is valid UTF-8");
    println!("library version: {version}\n");

    // Create a socket that is initially off, rated at 1500 W.
    let socket: *mut SmartSocket = smart_socket_ffi::smart_socket_new(false, 1500.0);
    if socket.is_null() {
        panic!("failed to create smart socket");
    }

    // SAFETY: `socket` is a valid handle from `smart_socket_new`.
    unsafe { demo("Kettle (static)", socket) };

    // SAFETY: `socket` was returned by `smart_socket_new` and is freed exactly once.
    unsafe { smart_socket_ffi::smart_socket_free(socket) };

    // Null handles are safe to pass.
    // SAFETY: passing null is explicitly allowed by the API contract.
    unsafe {
        assert!(!smart_socket_ffi::smart_socket_is_on(ptr::null()));
        assert_eq!(smart_socket_ffi::smart_socket_power(ptr::null()), 0.0);
    }
}

/// Exercises the full on/off + power cycle of a smart socket handle.
///
/// # Safety
///
/// `socket` must be a valid (non-freed) [`SmartSocket`] handle.
unsafe fn demo(label: &str, socket: *const SmartSocket) {
    // SAFETY: `socket` is a valid (non-freed) handle.
    unsafe {
        let is_on = smart_socket_ffi::smart_socket_is_on(socket);
        let power = smart_socket_ffi::smart_socket_power(socket);
        let rated = smart_socket_ffi::smart_socket_rated_power(socket);
        println!("[{label}] initial: on={is_on}, power={power:.1} W (rated {rated:.1} W)");

        smart_socket_ffi::smart_socket_turn_on(socket.cast_mut());
        let power = smart_socket_ffi::smart_socket_power(socket);
        println!("[{label}] after turn_on:  on=true, power={power:.1} W");

        smart_socket_ffi::smart_socket_turn_off(socket.cast_mut());
        let power = smart_socket_ffi::smart_socket_power(socket);
        println!("[{label}] after turn_off: on=false, power={power:.1} W\n");
    }
}
