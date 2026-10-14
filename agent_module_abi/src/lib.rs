//! Linux ABI 2 plugin boundary. Only values and borrowed buffers cross it.
//! The plugin retains ownership of its module and task handles; the host
//! retains ownership of its input, output, and callback context.
#![no_std]

use core::ffi::c_void;

pub const ABI_VERSION: u32 = 2;
pub const MODULE_CHANNEL: u32 = 1;

pub const VALUE_MAP: u32 = 1;
pub const VALUE_LIST: u32 = 2;
pub const VALUE_STRING: u32 = 3;
pub const VALUE_BYTES: u32 = 4;
pub const VALUE_BOOL: u32 = 5;
pub const VALUE_UINT: u32 = 6;
pub const VALUE_INT: u32 = 7;
pub const VALUE_FLOAT32: u32 = 8;
pub const VALUE_FLOAT64: u32 = 9;
pub const VALUE_UINT32: u32 = 10;
pub const VALUE_UINT16: u32 = 11;
pub const VALUE_UINT8: u32 = 12;
pub const VALUE_INT32: u32 = 13;
pub const VALUE_INT16: u32 = 14;
pub const VALUE_INT8: u32 = 15;

pub const INPUT_HEADER: usize = 1;
pub const INPUT_BODY: usize = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bytes {
    pub ptr: *const u8,
    pub len: usize,
}

impl Bytes {
    pub const fn from_slice(value: &[u8]) -> Self {
        Self { ptr: value.as_ptr(), len: value.len() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value {
    pub kind: u32,
    pub reserved: u32,
    /// Numeric value or the number of elements for maps and lists.
    pub number: u64,
    /// A borrowed string/byte buffer, map field array, or list value array.
    pub bytes: Bytes,
    /// Opaque host-owned input handle, valid only during the `run` call.
    pub handle: usize,
}

impl Value {
    pub const fn empty() -> Self {
        Self { kind: 0, reserved: 0, number: 0, bytes: Bytes { ptr: core::ptr::null(), len: 0 }, handle: 0 }
    }

    pub const fn string(value: &[u8]) -> Self {
        Self { kind: VALUE_STRING, reserved: 0, number: 0, bytes: Bytes::from_slice(value), handle: 0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Field {
    pub key: Bytes,
    pub value: Value,
}

#[repr(C)]
pub struct HostApi {
    pub size: u32,
    pub version: u32,
    pub context: *mut c_void,
    /// Roots are `INPUT_HEADER` and `INPUT_BODY`; returned handles are opaque.
    pub get_field: unsafe extern "C" fn(*mut c_void, usize, Bytes, *mut Value) -> i32,
    pub get_index: unsafe extern "C" fn(*mut c_void, usize, usize, *mut Value) -> i32,
    /// Returned buffers remain valid for the rest of the `run` call.
    pub info_get: unsafe extern "C" fn(*mut c_void, Bytes, Bytes, *mut Value) -> i32,
    pub info_update: unsafe extern "C" fn(*mut c_void, Bytes, Bytes, *const Value) -> i32,
    pub stop_modules: unsafe extern "C" fn(*mut c_void, *const u64, usize, u64) -> i32,
    pub log: unsafe extern "C" fn(*mut c_void, u32, Bytes),
    /// `fields` and all nested buffers remain borrowed only for this call.
    pub send: unsafe extern "C" fn(*mut c_void, u32, u32, u64, *const Field, usize) -> i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Module {
    pub size: u32,
    pub flags: u32,
    pub id: u64,
    pub context: *mut c_void,
    pub create_task: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    pub run: unsafe extern "C" fn(*mut c_void, *const HostApi) -> i32,
    pub request_stop: unsafe extern "C" fn(*mut c_void, u32) -> i32,
    pub wait_stopped: unsafe extern "C" fn(*mut c_void, u64) -> i32,
    pub clear_cache: unsafe extern "C" fn(*mut c_void, Bytes),
    /// Writes at most `capacity` bytes and returns the full required length.
    pub describe_task: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> usize,
    pub destroy_task: unsafe extern "C" fn(*mut c_void),
}

// An immutable module table may be shared between loader threads. Mutable
// plugin state remains behind its opaque context and function contract.
unsafe impl Sync for Module {}

#[repr(C)]
pub struct Manifest {
    pub size: u32,
    pub version: u32,
    pub modules: *const Module,
    pub module_count: usize,
}

unsafe impl Sync for Manifest {}
