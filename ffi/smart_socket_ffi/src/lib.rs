//! Smart socket library exposing a C ABI.
//!
//! The socket has a rated power (watts) and an on/off switch. When the socket
//! is off its current power consumption is reported as `0`; when it is on the
//! current consumption equals the rated power.
//!
//! # Build artifacts
//!
//! The crate is compiled into three artifact types (see `Cargo.toml`):
//!
//! - `rlib` — Rust static archive, consumable by other Rust crates.
//! - `staticlib` — C-compatible static archive (`libsmart_socket_ffi.a`).
//! - `cdylib` — C-compatible dynamic library (`libsmart_socket_ffi.so` /
//!   `.dylib` / `.dll`), loadable at runtime via `dlopen`/`libloading`.
//!
//! # C API
//!
//! See [`smart_socket.h`](../smart_socket.h) for the C header. All functions
//! use `extern "C"`, are `#[no_mangle]`, and operate on an opaque
//! [`SmartSocket`] handle obtained from [`smart_socket_new`].

use std::os::raw::c_char;

/// Smart socket: on/off switch with a rated power consumption.
///
/// The layout is `#[repr(C)]` so it has a stable binary representation, but the
/// fields are not part of the public C API — C code treats the type as opaque
/// (`typedef struct SmartSocket SmartSocket;`).
#[repr(C)]
pub struct SmartSocket {
    is_on: bool,
    power_watts: f32,
}

impl SmartSocket {
    /// Creates a new local smart socket.
    ///
    /// Returns `None` if `power_watts` is negative or NaN.
    fn new(is_on: bool, power_watts: f32) -> Option<Self> {
        if power_watts.is_nan() || power_watts < 0.0 {
            return None;
        }
        Some(Self { is_on, power_watts })
    }

    fn turn_on(&mut self) {
        self.is_on = true;
    }

    fn turn_off(&mut self) {
        self.is_on = false;
    }

    fn is_on(&self) -> bool {
        self.is_on
    }

    /// Current power consumption: `power_watts` when on, `0` when off.
    fn power(&self) -> f32 {
        if self.is_on { self.power_watts } else { 0.0 }
    }

    fn rated_power(&self) -> f32 {
        self.power_watts
    }
}

// ---------------------------------------------------------------------------
// C ABI
// ---------------------------------------------------------------------------

/// Creates a new smart socket.
///
/// # Returns
///
/// A heap-allocated [`SmartSocket`] handle, or a null pointer if `power_watts`
/// is negative or NaN. The caller owns the handle and must release it with
/// [`smart_socket_free`].
///
/// This function is safe to call: it takes no pointer arguments and cannot
/// cause undefined behaviour by itself.
#[unsafe(no_mangle)]
pub extern "C" fn smart_socket_new(is_on: bool, power_watts: f32) -> *mut SmartSocket {
    match SmartSocket::new(is_on, power_watts) {
        Some(socket) => Box::into_raw(Box::new(socket)),
        None => std::ptr::null_mut(),
    }
}

/// Frees a smart socket previously created with [`smart_socket_new`].
///
/// No-op if `ptr` is null. Passing a pointer not returned by [`smart_socket_new`],
/// or freeing the same pointer twice, is undefined behaviour.
///
/// # Safety
///
/// `ptr` must be either null or a valid pointer returned by [`smart_socket_new`]
/// that has not yet been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_free(ptr: *mut SmartSocket) {
    if !ptr.is_null() {
        // SAFETY: caller guarantees `ptr` came from `Box::into_raw` and is used once.
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

/// Turns the socket on. No-op if `ptr` is null.
///
/// # Safety
///
/// `ptr` must be null or a valid (non-freed) [`SmartSocket`] handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_turn_on(ptr: *mut SmartSocket) {
    // SAFETY: caller guarantees `ptr` is null or a valid handle.
    if let Some(socket) = unsafe { as_mut(ptr) } {
        socket.turn_on();
    }
}

/// Turns the socket off. No-op if `ptr` is null.
///
/// # Safety
///
/// `ptr` must be null or a valid (non-freed) [`SmartSocket`] handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_turn_off(ptr: *mut SmartSocket) {
    // SAFETY: caller guarantees `ptr` is null or a valid handle.
    if let Some(socket) = unsafe { as_mut(ptr) } {
        socket.turn_off();
    }
}

/// Returns `true` if the socket is on, `false` if it is off or `ptr` is null.
///
/// # Safety
///
/// `ptr` must be null or a valid (non-freed) [`SmartSocket`] handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_is_on(ptr: *const SmartSocket) -> bool {
    // SAFETY: caller guarantees `ptr` is null or a valid handle.
    unsafe { as_ref(ptr) }.is_some_and(SmartSocket::is_on)
}

/// Returns the current power consumption in watts (`0` when off).
/// Returns `0` if `ptr` is null.
///
/// # Safety
///
/// `ptr` must be null or a valid (non-freed) [`SmartSocket`] handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_power(ptr: *const SmartSocket) -> f32 {
    // SAFETY: caller guarantees `ptr` is null or a valid handle.
    unsafe { as_ref(ptr) }.map_or(0.0, SmartSocket::power)
}

/// Returns the rated power in watts. Returns `0` if `ptr` is null.
///
/// # Safety
///
/// `ptr` must be null or a valid (non-freed) [`SmartSocket`] handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_rated_power(ptr: *const SmartSocket) -> f32 {
    // SAFETY: caller guarantees `ptr` is null or a valid handle.
    unsafe { as_ref(ptr) }.map_or(0.0, SmartSocket::rated_power)
}

/// Returns the library version as a UTF-8 C string.
///
/// The returned pointer is statically allocated and must **not** be freed.
///
/// This function is safe to call: it takes no arguments and returns a pointer
/// to static data.
#[unsafe(no_mangle)]
pub extern "C" fn smart_socket_version() -> *const c_char {
    static VERSION: &[u8] = b"smart_socket_ffi 0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Converts a const pointer into a borrowed reference, or `None` if null.
///
/// # Safety
///
/// Caller guarantees `ptr` is null or points to a valid, non-freed `SmartSocket`.
unsafe fn as_ref<'a>(ptr: *const SmartSocket) -> Option<&'a SmartSocket> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: caller guarantees the pointer is valid and non-freed.
        Some(unsafe { &*ptr })
    }
}

/// Converts a mut pointer into a mutably borrowed reference, or `None` if null.
///
/// # Safety
///
/// Caller guarantees `ptr` is null or points to a valid, non-freed, uniquely
/// owned `SmartSocket`.
unsafe fn as_mut<'a>(ptr: *mut SmartSocket) -> Option<&'a mut SmartSocket> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: caller guarantees the pointer is valid, non-freed, and unique.
        Some(unsafe { &mut *ptr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn socket(is_on: bool, power: f32) -> *mut SmartSocket {
        let ptr = smart_socket_new(is_on, power);
        assert!(!ptr.is_null(), "socket should be created");
        ptr
    }

    #[test]
    fn new_rejects_negative_power() {
        assert!(smart_socket_new(false, -1.0).is_null());
        assert!(smart_socket_new(true, f32::NAN).is_null());
    }

    #[test]
    fn on_off_cycle() {
        let ptr = socket(false, 100.0);
        // SAFETY: `ptr` is a valid handle from `smart_socket_new`.
        unsafe {
            assert!(!smart_socket_is_on(ptr));
            assert_eq!(smart_socket_power(ptr), 0.0);

            smart_socket_turn_on(ptr);
            assert!(smart_socket_is_on(ptr));
            assert_eq!(smart_socket_power(ptr), 100.0);

            smart_socket_turn_off(ptr);
            assert!(!smart_socket_is_on(ptr));
            assert_eq!(smart_socket_power(ptr), 0.0);

            smart_socket_free(ptr);
        }
    }

    #[test]
    fn rated_power() {
        let ptr = socket(true, 250.0);
        // SAFETY: `ptr` is a valid handle from `smart_socket_new`.
        unsafe {
            assert_eq!(smart_socket_rated_power(ptr), 250.0);
            smart_socket_turn_off(ptr);
            // Rated power is independent of the on/off state.
            assert_eq!(smart_socket_rated_power(ptr), 250.0);
            smart_socket_free(ptr);
        }
    }

    #[test]
    fn null_handles_are_safe() {
        // SAFETY: passing null is explicitly allowed by the API contract.
        unsafe {
            smart_socket_free(std::ptr::null_mut());
            smart_socket_turn_on(std::ptr::null_mut());
            smart_socket_turn_off(std::ptr::null_mut());
            assert!(!smart_socket_is_on(std::ptr::null()));
            assert_eq!(smart_socket_power(std::ptr::null()), 0.0);
            assert_eq!(smart_socket_rated_power(std::ptr::null()), 0.0);
        }
    }

    #[test]
    fn version_is_non_null() {
        let v = smart_socket_version();
        assert!(!v.is_null());
        // SAFETY: `smart_socket_version` returns a valid, NUL-terminated static C string.
        let bytes = unsafe { std::ffi::CStr::from_ptr(v).to_bytes() };
        let s = std::str::from_utf8(bytes).unwrap();
        assert!(s.starts_with("smart_socket_ffi"));
    }
}
