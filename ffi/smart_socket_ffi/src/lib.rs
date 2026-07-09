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

/// Cookie stored as the first field of every live socket.
///
/// Each FFI entry point checks it to reject foreign, corrupted, or
/// already-freed handles. This is a best-effort defence: it cannot protect
/// against completely arbitrary pointers (reading unmapped memory still
/// segfaults), but it catches the common misuse cases — wrong type, zeroed
/// allocation, or a handle whose cookie was stamped out by
/// [`smart_socket_free`].
const COOKIE: u32 = 0x5353_4F4B; // "SSOK" (Smart SOKet)

/// Smart socket: on/off switch with a rated power consumption.
///
/// The layout is `#[repr(C)]` so it has a stable binary representation, but the
/// fields are not part of the public C API — C code treats the type as opaque
/// (`typedef struct SmartSocket SmartSocket;`).
#[repr(C)]
pub struct SmartSocket {
    cookie: u32,
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
        Some(Self {
            cookie: COOKIE,
            is_on,
            power_watts,
        })
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
/// No-op if `ptr` is null or does not point to a live socket (the cookie is
/// checked). After freeing, the cookie is stamped out so a subsequent call
/// with the same pointer is rejected as well — a best-effort defence against
/// double-free.
///
/// # Safety
///
/// `ptr` must be null or a dereferenceable pointer. For full soundness it
/// should be a valid pointer returned by [`smart_socket_new`] that has not yet
/// been freed; the cookie check only guards against *readable* invalid memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_free(ptr: *mut SmartSocket) {
    if ptr.is_null() {
        return;
    }
    // Reject foreign/corrupted handles whose cookie does not match.
    // SAFETY: caller guarantees `ptr` is null (checked) or dereferenceable.
    if unsafe { (*ptr).cookie } != COOKIE {
        return;
    }
    // Stamp out the cookie so a later erroneous call is rejected instead of
    // double-freeing. Best-effort: reading freed memory is UB regardless.
    // SAFETY: same guarantee as above; write happens before deallocation.
    unsafe { (*ptr).cookie = 0 };
    // SAFETY: `ptr` was produced by `Box::into_raw` in `smart_socket_new`.
    unsafe { drop(Box::from_raw(ptr)) };
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

/// Converts a const pointer into a borrowed reference, or `None` if the
/// pointer is null or does not carry the cookie.
///
/// # Safety
///
/// Caller guarantees `ptr` is null or points to dereferenceable memory.
unsafe fn as_ref<'a>(ptr: *const SmartSocket) -> Option<&'a SmartSocket> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: caller guarantees the pointer is null (checked) or dereferenceable.
    let socket = unsafe { &*ptr };
    if socket.cookie != COOKIE {
        return None;
    }
    Some(socket)
}

/// Converts a mut pointer into a mutably borrowed reference, or `None` if the
/// pointer is null or does not carry the cookie.
///
/// # Safety
///
/// Caller guarantees `ptr` is null or points to dereferenceable memory.
unsafe fn as_mut<'a>(ptr: *mut SmartSocket) -> Option<&'a mut SmartSocket> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: caller guarantees the pointer is null (checked) or dereferenceable.
    let socket = unsafe { &mut *ptr };
    if socket.cookie != COOKIE {
        return None;
    }
    Some(socket)
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
    fn invalid_handle_is_rejected() {
        let ptr = socket(true, 100.0);

        // Corrupt the cookie to simulate a foreign / corrupted / freed handle.
        // SAFETY: `ptr` is a valid, live handle; we overwrite only the cookie field.
        unsafe { (*ptr).cookie = 0 };

        // SAFETY: the memory is still readable, but the signature no longer matches,
        // so every entry point must treat the handle as invalid (no-op / default).
        unsafe {
            smart_socket_turn_on(ptr);
            smart_socket_turn_off(ptr);
            assert!(!smart_socket_is_on(ptr));
            assert_eq!(smart_socket_power(ptr), 0.0);
            assert_eq!(smart_socket_rated_power(ptr), 0.0);
            // free must refuse to deallocate foreign/corrupted memory.
            smart_socket_free(ptr);
        }

        // Restore the cookie and free for real to avoid leaking the allocation.
        // SAFETY: memory was not freed above (cookie mismatch); we make it valid again.
        unsafe {
            (*ptr).cookie = COOKIE;
            smart_socket_free(ptr);
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
