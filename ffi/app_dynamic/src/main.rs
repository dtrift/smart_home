//! Demonstrates the smart socket library linked **dynamically at runtime**.
//!
//! Unlike `app_static`, this binary does *not* link `smart_socket_ffi` at
//! compile time. Instead it loads the `cdylib` artifact
//! (`libsmart_socket_ffi.so` / `.dylib` / `.dll`) with [`libloading`] at
//! startup and resolves the C ABI symbols through function pointers.
//!
//! This mirrors how a C program would call `dlopen`/`dlsym` (or `LoadLibrary`).

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::ptr;

use libloading::Library;

/// Opaque smart-socket handle. Matches the `#[repr(C)]` layout in the library
/// but the fields are treated as private — we only ever hold a pointer.
#[repr(C)]
struct SmartSocket {
    _private: [u8; 0],
}

/// Function-pointer types mirroring the C declarations in `smart_socket.h`.
type SmartSocketNewFn = unsafe extern "C" fn(is_on: bool, power_watts: f32) -> *mut SmartSocket;
type SmartSocketFreeFn = unsafe extern "C" fn(ptr: *mut SmartSocket);
type SmartSocketTurnOnFn = unsafe extern "C" fn(ptr: *mut SmartSocket);
type SmartSocketTurnOffFn = unsafe extern "C" fn(ptr: *mut SmartSocket);
type SmartSocketIsOnFn = unsafe extern "C" fn(ptr: *const SmartSocket) -> bool;
type SmartSocketPowerFn = unsafe extern "C" fn(ptr: *const SmartSocket) -> f32;
type SmartSocketRatedPowerFn = unsafe extern "C" fn(ptr: *const SmartSocket) -> f32;
type SmartSocketVersionFn = unsafe extern "C" fn() -> *const c_char;

/// Bundles the resolved symbols into a single owned handle.
///
/// The [`Library`] is kept alive for as long as the function pointers are used:
/// dropping it would unmap the code they point at.
struct SmartSocketApi {
    _lib: Library,
    new_fn: SmartSocketNewFn,
    free_fn: SmartSocketFreeFn,
    turn_on_fn: SmartSocketTurnOnFn,
    turn_off_fn: SmartSocketTurnOffFn,
    is_on_fn: SmartSocketIsOnFn,
    power_fn: SmartSocketPowerFn,
    rated_power_fn: SmartSocketRatedPowerFn,
    version_fn: SmartSocketVersionFn,
}

impl SmartSocketApi {
    /// Loads the cdylib from the workspace `target/` directory and resolves
    /// all required symbols.
    fn load() -> Result<Self, String> {
        let lib_path = resolve_library_path()?;

        // SAFETY: `Library::new` loads a shared object. The library is a trusted
        // build artifact produced by this workspace; loading it is safe in the
        // same sense as linking any native dependency.
        let lib = unsafe { Library::new(&lib_path) }
            .map_err(|e| format!("failed to load {}: {e}", lib_path.display()))?;

        // SAFETY: the resolved symbols have the C ABI declared above and are
        // `#[no_mangle]`, so their names are stable.
        let new_fn = unsafe {
            *lib.get::<SmartSocketNewFn>(b"smart_socket_new\0")
                .map_err(|e| format!("symbol smart_socket_new: {e}"))?
        };
        let free_fn = unsafe {
            *lib.get::<SmartSocketFreeFn>(b"smart_socket_free\0")
                .map_err(|e| format!("symbol smart_socket_free: {e}"))?
        };
        let turn_on_fn = unsafe {
            *lib.get::<SmartSocketTurnOnFn>(b"smart_socket_turn_on\0")
                .map_err(|e| format!("symbol smart_socket_turn_on: {e}"))?
        };
        let turn_off_fn = unsafe {
            *lib.get::<SmartSocketTurnOffFn>(b"smart_socket_turn_off\0")
                .map_err(|e| format!("symbol smart_socket_turn_off: {e}"))?
        };
        let is_on_fn = unsafe {
            *lib.get::<SmartSocketIsOnFn>(b"smart_socket_is_on\0")
                .map_err(|e| format!("symbol smart_socket_is_on: {e}"))?
        };
        let power_fn = unsafe {
            *lib.get::<SmartSocketPowerFn>(b"smart_socket_power\0")
                .map_err(|e| format!("symbol smart_socket_power: {e}"))?
        };
        let rated_power_fn = unsafe {
            *lib.get::<SmartSocketRatedPowerFn>(b"smart_socket_rated_power\0")
                .map_err(|e| format!("symbol smart_socket_rated_power: {e}"))?
        };
        let version_fn = unsafe {
            *lib.get::<SmartSocketVersionFn>(b"smart_socket_version\0")
                .map_err(|e| format!("symbol smart_socket_version: {e}"))?
        };

        Ok(Self {
            _lib: lib,
            new_fn,
            free_fn,
            turn_on_fn,
            turn_off_fn,
            is_on_fn,
            power_fn,
            rated_power_fn,
            version_fn,
        })
    }
}

fn main() {
    println!("=== Dynamic linking demo (cdylib at runtime) ===\n");

    let api = SmartSocketApi::load().unwrap_or_else(|e| panic!("{e}"));

    // SAFETY: returns a static, NUL-terminated C string.
    let version_raw = unsafe { (api.version_fn)() };
    let version = unsafe { CStr::from_ptr(version_raw) }
        .to_str()
        .expect("version is valid UTF-8");
    println!("library version: {version}\n");

    // Create a socket that is initially off, rated at 120 W.
    // SAFETY: valid arguments; pointer released below.
    let socket: *mut SmartSocket = unsafe { (api.new_fn)(false, 120.0) };
    if socket.is_null() {
        panic!("failed to create smart socket");
    }

    // SAFETY: `socket` is a valid handle for the duration of the call.
    unsafe { demo("Floor lamp (dynamic)", &api, socket) };

    // SAFETY: `socket` came from `new_fn` and is freed exactly once.
    unsafe { (api.free_fn)(socket) };

    // SAFETY: passing null is explicitly allowed by the API contract.
    unsafe {
        assert!(!(api.is_on_fn)(ptr::null()));
        assert_eq!((api.power_fn)(ptr::null()), 0.0);
    }
}

/// Exercises the full on/off + power cycle of a smart socket handle.
///
/// # Safety
///
/// `socket` must be a valid (non-freed) [`SmartSocket`] handle obtained from
/// `api.new_fn`, and `api` must stay alive.
unsafe fn demo(label: &str, api: &SmartSocketApi, socket: *const SmartSocket) {
    // SAFETY: `socket` is a valid (non-freed) handle; `api` stays alive.
    unsafe {
        let is_on = (api.is_on_fn)(socket);
        let power = (api.power_fn)(socket);
        let rated = (api.rated_power_fn)(socket);
        println!("[{label}] initial: on={is_on}, power={power:.1} W (rated {rated:.1} W)");

        (api.turn_on_fn)(socket.cast_mut());
        let power = (api.power_fn)(socket);
        println!("[{label}] after turn_on:  on=true, power={power:.1} W");

        (api.turn_off_fn)(socket.cast_mut());
        let power = (api.power_fn)(socket);
        println!("[{label}] after turn_off: on=false, power={power:.1} W\n");
    }
}

/// Locates the compiled `cdylib` in the workspace `target/<profile>/` directory.
///
/// We compute the path relative to `CARGO_MANIFEST_DIR` (this crate's source
/// dir) and pick the platform-specific file name.
fn resolve_library_path() -> Result<PathBuf, String> {
    // `ffi/app_dynamic` -> `../../target` -> workspace `target/`.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let target_dir = manifest_dir.join("../../target").join(profile);

    let lib_name = if cfg!(target_os = "windows") {
        "smart_socket_ffi.dll"
    } else if cfg!(target_os = "macos") {
        "libsmart_socket_ffi.dylib"
    } else {
        "libsmart_socket_ffi.so"
    };

    let path = target_dir.join(lib_name);
    if path.exists() {
        return Ok(path);
    }

    Err(format!(
        "cdylib not found at {}. \
         Run `cargo build -p smart_socket_ffi` first (or `cargo build` from the workspace root).",
        path.display()
    ))
}
