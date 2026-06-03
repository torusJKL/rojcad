//! rojcad — Headless parametric CAD system with embedded Janet DSL.
//!
//! This binary embeds the Janet interpreter, registers CAD functions
//! (make-box, make-sphere, cut, common, shape-type, hide, show, visible?,
//! write-step, write-stl), and starts a TCP REPL server on port 9000.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

mod bridge;
mod cad;
mod types;

use std::ffi::{c_char, c_double, c_int, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use types::ShapeData;

// ── Size helper for Janet GC allocation ─────────────────────────────────────

/// Return the size of ShapeData. Used by janet_abstract in C to allocate
/// the right amount of memory via the GC.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_data_size() -> usize {
    std::mem::size_of::<ShapeData>()
}

// ── Shape lifecycle ─────────────────────────────────────────────────────────

/// Run destructors on a ShapeData without freeing the backing memory
/// (Janet GC owns the memory via janet_abstract).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_drop(data: *mut c_void, _len: usize) {
    if !data.is_null() {
        unsafe {
            ptr::drop_in_place(data as *mut ShapeData);
        }
    }
}

/// Return the shape type string (e.g., "SOLID") for display.
/// Returns a leaked CString pointer — Janet reads it during tostring and
/// does not take ownership, so we leak intentionally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_type_string(data: *mut c_void) -> *const c_char {
    let shape_data = unsafe { &*(data as *const ShapeData) };
    let s = CString::new(shape_data.type_string()).unwrap();
    s.into_raw()
}

// ── Primitives — initialize at a pre-allocated destination ───────────────────

/// Initialize a ShapeData as a box at the given destination.
/// dest must point to sizeof(ShapeData) bytes allocated via janet_abstract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_box(
    dest: *mut c_void,
    width: c_double,
    depth: c_double,
    height: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        cad::make_box(width, depth, height, center)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown error".to_string()
            };
            panic!("rust_init_box failed: {}", msg);
        }
    }
}

/// Initialize a ShapeData as a sphere at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_sphere(
    dest: *mut c_void,
    radius: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        cad::make_sphere(radius, center)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_sphere failed");
        }
    }
}

// ── Boolean operations — initialize at a pre-allocated destination ──────────

/// Subtract shape b from shape a, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cut(
    dest: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let shape_a = unsafe { &*(a as *const ShapeData) };
        let shape_b = unsafe { &*(b as *const ShapeData) };
        cad::cut(shape_a, shape_b)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_cut failed");
        }
    }
}

/// Intersect shape a with shape b, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_common(
    dest: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let shape_a = unsafe { &*(a as *const ShapeData) };
        let shape_b = unsafe { &*(b as *const ShapeData) };
        cad::common(shape_a, shape_b)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_common failed");
        }
    }
}

// ── Inspection ──────────────────────────────────────────────────────────────

/// Return the shape type as a lowercased C string (e.g., "solid").
/// Returns a leaked CString pointer — the caller reads it immediately.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_type(data: *mut c_void) -> *const c_char {
    let shape_data = unsafe { &*(data as *const ShapeData) };
    let s = CString::new(shape_data.type_string().to_lowercase()).unwrap();
    s.into_raw()
}

// ── Visibility ──────────────────────────────────────────────────────────────

/// Set the visible flag on a shape.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_set_visible(data: *mut c_void, visible: c_int) {
    let shape_data = unsafe { &mut *(data as *mut ShapeData) };
    shape_data.visible = visible != 0;
}

/// Get the visible flag from a shape.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_get_visible(data: *mut c_void) -> c_int {
    let shape_data = unsafe { &*(data as *const ShapeData) };
    shape_data.visible as c_int
}

// ── Export ──────────────────────────────────────────────────────────────────

/// Write a shape to a STEP file. Returns 0 on success, 1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_write_step(data: *mut c_void, path: *const c_char) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy().to_string();
    let shape_data = unsafe { &*(data as *const ShapeData) };
    match cad::write_step(shape_data, &path_str) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

/// Write a shape to an STL file. Returns 0 on success, 1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_write_stl(data: *mut c_void, path: *const c_char) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy().to_string();
    let shape_data = unsafe { &*(data as *const ShapeData) };
    match cad::write_stl(shape_data, &path_str) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

// ── C bridge registration forward declaration ────────────────────────────────

unsafe extern "C" {
    fn cad_register_functions(env: *mut bridge::JanetTable);
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    // Initialize Janet
    unsafe {
        bridge::janet_init();
    }

    // Get the core environment
    let env: *mut bridge::JanetTable;
    unsafe {
        env = bridge::janet_core_env(ptr::null());
    }

    // Register CAD functions
    unsafe {
        cad_register_functions(env);
    }

    // Embed and run boot.janet
    let boot_code = include_str!("../boot.janet");
    let boot_c = CString::new(boot_code).unwrap_or_else(|_| CString::new("").unwrap());
    let name_c = CString::new("boot.janet").unwrap();

    let mut result = bridge::Janet(0);
    let status = unsafe {
        bridge::janet_dostring(env, boot_c.as_ptr(), name_c.as_ptr(), &mut result)
    };

    if status != 0 {
        eprintln!("rojcad: failed to load boot.janet");
        unsafe {
            bridge::janet_deinit();
        }
        std::process::exit(1);
    }

    // The event loop runs automatically via the Janet VM.
    // boot.janet has a (forever ...) loop that blocks indefinitely.
    // If we reach here (shouldn't under normal operation), clean up.
    unsafe {
        bridge::janet_deinit();
    }
}
