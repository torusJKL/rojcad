//! rojcad — Headless parametric CAD system with embedded Janet DSL.
//!
//! This binary embeds the Janet interpreter, registers CAD functions
//! (box, sphere, cylinder, cone, torus, cut, common, shape-type, hide, show,
//! visible?, write-step, write-stl), and starts a TCP REPL server on port 9365
//! (configurable via --port).

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

mod bridge;
mod cad;
mod types;
mod viewer;

use std::ffi::{c_char, c_double, c_int, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::Ordering;

use types::{
    global_shape_registry, init_edge_color_defaults, pack_color, ShapeData, ACTIVE_EDGE_COLOR,
    EDGE_THICKNESS, INACTIVE_EDGE_COLOR, LAST_SELECTION, SHOW_ACTIVE_EDGES, SHOW_INACTIVE_EDGES,
};

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
    angle: *const c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        let angle_val = if angle.is_null() {
            None
        } else {
            unsafe { Some(*angle) }
        };
        cad::make_sphere(radius, center, angle_val)
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

/// Initialize a ShapeData as a cube at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cube(
    dest: *mut c_void,
    size: c_double,
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
        cad::make_cube(size, center)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_cube failed");
        }
    }
}

/// Initialize a ShapeData as a box from two opposite corners.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_box_from_corners(
    dest: *mut c_void,
    c1x: c_double, c1y: c_double, c1z: c_double,
    c2x: c_double, c2y: c_double, c2z: c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        cad::make_box_from_corners((c1x, c1y, c1z), (c2x, c2y, c2z))
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_box_from_corners failed");
        }
    }
}

/// Initialize a ShapeData as a cylinder at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cylinder(
    dest: *mut c_void,
    radius: c_double,
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
        cad::make_cylinder(radius, height, center)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_cylinder failed");
        }
    }
}

/// Initialize a ShapeData as a cylinder between two points.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cylinder_from_points(
    dest: *mut c_void,
    p1x: c_double, p1y: c_double, p1z: c_double,
    p2x: c_double, p2y: c_double, p2z: c_double,
    radius: c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        cad::make_cylinder_from_points((p1x, p1y, p1z), (p2x, p2y, p2z), radius)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_cylinder_from_points failed");
        }
    }
}

/// Initialize a ShapeData as a cylinder at a point extending in a direction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cylinder_point_dir(
    dest: *mut c_void,
    px: c_double, py: c_double, pz: c_double,
    radius: c_double,
    dx: c_double, dy: c_double, dz: c_double,
    height: c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        cad::make_cylinder_point_dir((px, py, pz), radius, (dx, dy, dz), height)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_cylinder_point_dir failed");
        }
    }
}

/// Initialize a ShapeData as a cone at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cone(
    dest: *mut c_void,
    bottom_radius: c_double,
    top_radius: c_double,
    height: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
    angle: *const c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        let angle_val = if angle.is_null() {
            None
        } else {
            unsafe { Some(*angle) }
        };
        cad::make_cone(bottom_radius, top_radius, height, center, angle_val)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_cone failed");
        }
    }
}

/// Initialize a ShapeData as a torus at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_torus(
    dest: *mut c_void,
    ring_radius: c_double,
    tube_radius: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
    zx: *const c_double,
    zy: *const c_double,
    zz: *const c_double,
    angle: *const c_double,
    angle_start: *const c_double,
    angle_end: *const c_double,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        let z_axis = if zx.is_null() || zy.is_null() || zz.is_null() {
            None
        } else {
            unsafe { Some((*zx, *zy, *zz)) }
        };
        let angle_val = if angle.is_null() {
            None
        } else {
            unsafe { Some(*angle) }
        };
        let a_start = if angle_start.is_null() {
            None
        } else {
            unsafe { Some(*angle_start) }
        };
        let a_end = if angle_end.is_null() {
            None
        } else {
            unsafe { Some(*angle_end) }
        };
        cad::make_torus(ring_radius, tube_radius, center, z_axis, angle_val, a_start, a_end)
    }));
    match result {
        Ok(shape_data) => {
            unsafe { ptr::write(dest as *mut ShapeData, shape_data); }
        }
        Err(_) => {
            panic!("rust_init_torus failed");
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
    let is_visible = visible != 0;
    shape_data.visible = is_visible;
    global_shape_registry().set_visible(shape_data.shape_id, is_visible);
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

// ── Selection callback ───────────────────────────────────────────────────────

/// Poll for a pending selection event.
/// Returns 0 if no event, u64::MAX for deselected, or the selected shape ID.
/// Resets the event to 0 after reading.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_poll_selection() -> u64 {
    LAST_SELECTION.swap(0, Ordering::SeqCst)
}

// ── Edge visibility toggles ────────────────────────────────────────────────────

/// Toggle inactive edge visibility. Returns new state (1 = showing, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_toggle_inactive() -> c_int {
    let old = SHOW_INACTIVE_EDGES.fetch_xor(true, Ordering::SeqCst);
    c_int::from(!old)
}

/// Toggle active edge visibility. Returns new state (1 = showing, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_toggle_active() -> c_int {
    let old = SHOW_ACTIVE_EDGES.fetch_xor(true, Ordering::SeqCst);
    c_int::from(!old)
}

/// Query inactive edge visibility state (1 = showing, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_inactive_showing() -> c_int {
    c_int::from(SHOW_INACTIVE_EDGES.load(Ordering::SeqCst))
}

/// Query active edge visibility state (1 = showing, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_active_showing() -> c_int {
    c_int::from(SHOW_ACTIVE_EDGES.load(Ordering::SeqCst))
}

// ── Edge style (thickness / color) ─────────────────────────────────────────────

/// Set edge thickness in NDC units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_set_thickness(value: c_double) {
    EDGE_THICKNESS.store(value.to_bits(), Ordering::SeqCst);
}

/// Query edge thickness in NDC units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_get_thickness() -> c_double {
    f64::from_bits(EDGE_THICKNESS.load(Ordering::SeqCst))
}

/// Set inactive edge color (r, g, b as doubles in [0, 1]).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_set_color_inactive(r: c_double, g: c_double, b: c_double) {
    INACTIVE_EDGE_COLOR.store(pack_color(r, g, b), Ordering::SeqCst);
}

/// Set active (selected) edge color (r, g, b as doubles in [0, 1]).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_set_color_active(r: c_double, g: c_double, b: c_double) {
    ACTIVE_EDGE_COLOR.store(pack_color(r, g, b), Ordering::SeqCst);
}

// ── C bridge registration forward declaration ────────────────────────────────

unsafe extern "C" {
    fn cad_register_functions(env: *mut bridge::JanetTable);
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn parse_port_arg() -> Option<u16> {
    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        if let Some(port_str) = arg.strip_prefix("--port=") {
            let p: u16 = port_str.parse().unwrap_or_else(|_| {
                eprintln!("rojcad: invalid port '{}'", port_str);
                std::process::exit(1);
            });
            if !(1..=65535).contains(&p) {
                eprintln!("rojcad: port must be between 1 and 65535, got {}", p);
                std::process::exit(1);
            }
            return Some(p);
        }
        if arg == "--port" {
            let next = args.next().unwrap_or_else(|| {
                eprintln!("rojcad: --port requires a value");
                std::process::exit(1);
            });
            let p: u16 = next.parse().unwrap_or_else(|_| {
                eprintln!("rojcad: invalid port '{}'", next);
                std::process::exit(1);
            });
            if !(1..=65535).contains(&p) {
                eprintln!("rojcad: port must be between 1 and 65535, got {}", p);
                std::process::exit(1);
            }
            return Some(p);
        }
    }
    None
}

fn main() {
    // Parse CLI arguments
    let headless: bool = std::env::args().any(|arg| arg == "--headless");
    let port: u16 = parse_port_arg().unwrap_or(9365);

    // Initialize edge style defaults
    init_edge_color_defaults();

    // Initialize Janet
    unsafe {
        bridge::janet_init();
    }

    // Get the core environment
    let env: *mut bridge::JanetTable;
    unsafe {
        env = bridge::janet_core_env(ptr::null());
    }

    // Register Janet core library modules.
    // Under JANET_BOOTSTRAP these aren't auto-registered, so we do it manually.
    unsafe {
        bridge::janet_lib_io(env);
        bridge::janet_lib_math(env);
        bridge::janet_lib_array(env);
        bridge::janet_lib_tuple(env);
        bridge::janet_lib_buffer(env);
        bridge::janet_lib_table(env);
        bridge::janet_lib_struct(env);
        bridge::janet_lib_fiber(env);
        bridge::janet_lib_os(env);
        bridge::janet_lib_parse(env);
        bridge::janet_lib_compile(env);
        bridge::janet_lib_debug(env);
        bridge::janet_lib_string(env);
        bridge::janet_lib_marsh(env);
        bridge::janet_lib_ev(env);
        bridge::janet_lib_net(env);
    }

    // Register CAD functions
    unsafe {
        cad_register_functions(env);
    }

    // Set the netrepl port as a Janet dynamic variable so boot.janet
    // can read it via (dyn '*netrepl-port*') instead of hardcoding.
    // Only set when explicitly provided; boot.janet falls back to 9365.
    if 9365 != port {
        unsafe {
            let port_name = CString::new("*netrepl-port*").unwrap();
            bridge::janet_setdyn(port_name.as_ptr(), bridge::janet_wrap_number(port as f64));
        }
    }

    // Start viewer thread unless --headless flag is present
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    let _viewer_handle = if !headless {
        Some(viewer::spawn_viewer())
    } else {
        None
    };

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
