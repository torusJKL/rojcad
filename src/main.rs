//! rojcad — Headless parametric CAD system with embedded Janet DSL.
//!
//! This binary embeds the Janet interpreter, registers CAD functions
//! (box, sphere, cylinder, cone, torus, cut, common, shape-type, hide, show,
//! visible?, write-step, write-stl), and starts two TCP REPL servers:
//! a raw-text REPL on port 9364 (configurable via --raw-port) and
//! a spork netrepl protocol server on port 9365 (configurable via --spork-port).

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    clippy::missing_safety_doc,
    clippy::too_many_arguments
)]

mod bridge;
mod cad;
mod crash_handler;
mod gui_repl;
mod sketch;
mod text;
mod types;
mod viewer;

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_double, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;
use std::sync::mpsc;

use crate::types::{
    ACTIVE_EDGE_COLOR, EDGE_THICKNESS, HELP_EXAMPLE, INACTIVE_EDGE_COLOR, LAST_EDGE_ACTION,
    LAST_EDGE_INDEX, LAST_EDGE_SHAPE_ID, LAST_SELECTION, LAST_SELECTION_ACTION,
    PROJECTION_PERSPECTIVE, QUIT_REQUESTED, ReplToViewer, SHOW_ACTIVE_EDGES, SHOW_BACK_EDGES,
    SHOW_HELP_OVERLAY, SHOW_INACTIVE_EDGES, SHOW_REPL_PANEL, SHOW_STATS_OVERLAY, ShapeData,
    WINDOW_FULLSCREEN, WINDOW_HEIGHT, WINDOW_MAXIMIZED, WINDOW_WIDTH, global_shape_registry,
    init_edge_color_defaults, pack_color, register_shape_pointer,
};
use crate::viewer::ViewerConfig;

use glam::DVec3;
use opencascade::primitives::{Face, Shape};

// ── Thread-local error buffer for propagating CAD errors to C bridge ─────

std::thread_local! {
    static LAST_CAD_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

pub(crate) fn set_last_error(msg: String) {
    LAST_CAD_ERROR.with(|e| *e.borrow_mut() = msg);
}

pub(crate) fn take_last_error() -> String {
    LAST_CAD_ERROR.with(|e| std::mem::take(&mut *e.borrow_mut()))
}

/// Retrieve the last CAD error message as a C string.
/// The caller (C bridge) owns the returned pointer and must free it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_take_last_error() -> *mut c_char {
    let msg = take_last_error();
    CString::new(msg).unwrap().into_raw()
}

// ── catch_unwind helpers ─────────────────────────────────────────────────────

/// Extract a human-readable message from a panic payload.
fn panic_detail(panic: &Box<dyn std::any::Any + Send>) -> String {
    crash_handler::panic_detail(panic)
}

/// Run a CAD function that produces a ShapeData, catching panics.
/// On success: writes ShapeData to `dest`, registers pointer, returns 0.
/// On expected error: stores message, returns 1.
/// On panic: extracts panic message, stores "name panicked: {msg}", returns 1.
fn catch_cad<F>(name: &str, dest: *mut c_void, f: F) -> c_int
where
    F: FnOnce() -> Result<ShapeData, String>,
{
    let result = catch_unwind(AssertUnwindSafe(f));
    match result {
        Ok(Ok(sd)) => {
            let shape_id = sd.shape_id;
            unsafe {
                ptr::write(dest as *mut ShapeData, sd);
            }
            register_shape_pointer(shape_id, dest);
            0
        }
        Ok(Err(msg)) => {
            set_last_error(msg);
            1
        }
        Err(panic) => {
            let detail = panic_detail(&panic);
            set_last_error(format!("{name} panicked: {detail}"));
            1
        }
    }
}

/// Run a CAD function that produces no value, catching panics.
/// On success: returns 0.
/// On expected error: stores message, returns 1.
/// On panic: extracts panic message, stores "name panicked: {msg}", returns 1.
#[allow(dead_code)]
fn catch_result<F>(name: &str, f: F) -> c_int
where
    F: FnOnce() -> Result<(), String>,
{
    let result = catch_unwind(AssertUnwindSafe(f));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(msg)) => {
            set_last_error(msg);
            1
        }
        Err(panic) => {
            let detail = panic_detail(&panic);
            set_last_error(format!("{name} panicked: {detail}"));
            1
        }
    }
}

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

/// Extract the numeric shape_id from a ShapeData pointer.
/// This lets Janet-side code include a shape's ID in JSON responses.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_get_id(data: *const c_void) -> u64 {
    let shape_data = unsafe { &*(data as *const ShapeData) };
    shape_data.shape_id
}

// ── Primitives — initialize at a pre-allocated destination ───────────────────

/// Initialize a ShapeData as a box at the given destination.
/// Returns 0 on success, 1 on error (error string available via rust_take_last_error).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_box(
    dest: *mut c_void,
    width: c_double,
    depth: c_double,
    height: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_box", dest, || {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        cad::make_box(width, depth, height, center, eager)
    })
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
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_sphere", dest, || {
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
        cad::make_sphere(radius, center, angle_val, eager)
    })
}

/// Initialize a ShapeData as a cube at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cube(
    dest: *mut c_void,
    size: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_cube", dest, || {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        cad::make_cube(size, center, eager)
    })
}

/// Initialize a ShapeData as a box from two opposite corners.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_box_from_corners(
    dest: *mut c_void,
    c1x: c_double,
    c1y: c_double,
    c1z: c_double,
    c2x: c_double,
    c2y: c_double,
    c2z: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_box_from_corners", dest, || {
        cad::make_box_from_corners((c1x, c1y, c1z), (c2x, c2y, c2z), eager)
    })
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
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_cylinder", dest, || {
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            None
        } else {
            unsafe { Some((*cx, *cy, *cz)) }
        };
        cad::make_cylinder(radius, height, center, eager)
    })
}

/// Initialize a ShapeData as a cylinder between two points.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cylinder_from_points(
    dest: *mut c_void,
    p1x: c_double,
    p1y: c_double,
    p1z: c_double,
    p2x: c_double,
    p2y: c_double,
    p2z: c_double,
    radius: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_cylinder_from_points", dest, || {
        cad::make_cylinder_from_points((p1x, p1y, p1z), (p2x, p2y, p2z), radius, eager)
    })
}

/// Initialize a ShapeData as a cylinder at a point extending in a direction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cylinder_point_dir(
    dest: *mut c_void,
    px: c_double,
    py: c_double,
    pz: c_double,
    radius: c_double,
    dx: c_double,
    dy: c_double,
    dz: c_double,
    height: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_cylinder_point_dir", dest, || {
        cad::make_cylinder_point_dir((px, py, pz), radius, (dx, dy, dz), height, eager)
    })
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
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_cone", dest, || {
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
        cad::make_cone(bottom_radius, top_radius, height, center, angle_val, eager)
    })
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
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_torus", dest, || {
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
        cad::make_torus(
            ring_radius,
            tube_radius,
            center,
            z_axis,
            angle_val,
            a_start,
            a_end,
            eager,
        )
    })
}

// ── Sketch lifecycle ────────────────────────────────────────────────────────

/// Return the size of SketchData for janet_abstract allocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_sketch_data_size() -> usize {
    std::mem::size_of::<sketch::SketchData>()
}

/// Run destructors on a SketchData without freeing the backing memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_sketch_drop(data: *mut c_void, _len: usize) {
    if !data.is_null() {
        unsafe {
            ptr::drop_in_place(data as *mut sketch::SketchData);
        }
    }
}

/// Create a new sketch on a workplane.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_sketch_new(
    dest: *mut c_void,
    plane: *const c_char,
    at_x: c_double,
    at_y: c_double,
    at_z: c_double,
) {
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };
    let at = if at_x == 0.0 && at_y == 0.0 && at_z == 0.0 {
        None
    } else {
        Some((at_x, at_y, at_z))
    };
    let wp = cad::workplane_from_keyword(&plane_str, at);
    let sk = sketch::SketchData::new(wp);
    unsafe {
        ptr::write(dest as *mut sketch::SketchData, sk);
    }
}

macro_rules! sketch_op {
    ($name:ident, $method:ident, $( $arg:ident ),*) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            dest: *mut c_void,
            src: *mut c_void,
            $( $arg: c_double ),*
        ) {
            let src_sketch = unsafe { &*(src as *const sketch::SketchData) };
            let result = src_sketch.$method( $( $arg ),* );
            unsafe { ptr::write(dest as *mut sketch::SketchData, result); }
        }
    };
}

sketch_op!(rust_sketch_move_to, move_to, x, y);
sketch_op!(rust_sketch_line_to, line_to, x, y);
sketch_op!(rust_sketch_line_dx, line_dx, dx);
sketch_op!(rust_sketch_line_dy, line_dy, dy);
sketch_op!(rust_sketch_line_dx_dy, line_dx_dy, dx, dy);
sketch_op!(rust_sketch_arc_to, arc_to, x2, y2, x3, y3);

/// Close a sketch into a Face.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_sketch_close(shape_dest: *mut c_void, src: *mut c_void) {
    let src_sketch = unsafe { &*(src as *const sketch::SketchData) };
    let wire = src_sketch.close();
    let face = Face::from_wire(&wire);
    let sd = ShapeData::new(Shape::from(face));
    let shape_id = sd.shape_id;
    unsafe {
        ptr::write(shape_dest as *mut ShapeData, sd);
    }
    register_shape_pointer(shape_id, shape_dest);
}

/// Build an unclosed Wire from a sketch.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_sketch_build_wire(shape_dest: *mut c_void, src: *mut c_void) {
    let src_sketch = unsafe { &*(src as *const sketch::SketchData) };
    let wire = src_sketch.build_wire();
    let sd = ShapeData::new(Shape::from(wire));
    let shape_id = sd.shape_id;
    unsafe {
        ptr::write(shape_dest as *mut ShapeData, sd);
    }
    register_shape_pointer(shape_id, shape_dest);
}

// ── 2D Primitives — initialize at a pre-allocated destination ──────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_rect(
    dest: *mut c_void,
    w: c_double,
    d: c_double,
    is_wire: c_int,
    plane: *const c_char,
    at_x: c_double,
    at_y: c_double,
    at_z: c_double,
    eager: c_int,
) -> c_int {
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };
    let at = if at_x == 0.0 && at_y == 0.0 && at_z == 0.0 {
        None
    } else {
        Some((at_x, at_y, at_z))
    };
    let eager = eager != 0;
    catch_cad("rust_init_rect", dest, || {
        cad::make_rect(w, d, is_wire != 0, &plane_str, at, eager)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_circle(
    dest: *mut c_void,
    r: c_double,
    is_wire: c_int,
    plane: *const c_char,
    at_x: c_double,
    at_y: c_double,
    at_z: c_double,
    eager: c_int,
) -> c_int {
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };
    let at = if at_x == 0.0 && at_y == 0.0 && at_z == 0.0 {
        None
    } else {
        Some((at_x, at_y, at_z))
    };
    let eager = eager != 0;
    catch_cad("rust_init_circle", dest, || {
        cad::make_circle(r, is_wire != 0, &plane_str, at, eager)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_polygon(
    dest: *mut c_void,
    pts: *const c_double,
    npts: c_int,
    is_wire: c_int,
    plane: *const c_char,
    at_x: c_double,
    at_y: c_double,
    at_z: c_double,
    eager: c_int,
) -> c_int {
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };
    let at = if at_x == 0.0 && at_y == 0.0 && at_z == 0.0 {
        None
    } else {
        Some((at_x, at_y, at_z))
    };
    let pts_slice = unsafe { std::slice::from_raw_parts(pts, npts as usize) };
    let eager = eager != 0;
    catch_cad("rust_init_polygon", dest, || {
        cad::make_polygon(pts_slice, is_wire != 0, &plane_str, at, eager)
    })
}

// ── Ext/Rev Operations — operates on existing ShapeData ────────────────────

/// Extrude a Face.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_extrude(
    dest: *mut c_void,
    data: *mut c_void,
    height: c_double,
    dx: c_double,
    dy: c_double,
    dz: c_double,
    both: c_int,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_init_extrude", dest, || {
        cad::extrude_shape(shape, height, DVec3::new(dx, dy, dz), both != 0, eager != 0)
    })
}

/// Revolve a Face.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_revolve(
    dest: *mut c_void,
    data: *mut c_void,
    angle: c_double,
    ox: c_double,
    oy: c_double,
    oz: c_double,
    dx: c_double,
    dy: c_double,
    dz: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_init_revolve", dest, || {
        cad::revolve_shape(
            shape,
            angle,
            DVec3::new(ox, oy, oz),
            DVec3::new(dx, dy, dz),
            eager != 0,
        )
    })
}

/// One-shot polygon extrusion.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_extrude_polygon(
    dest: *mut c_void,
    pts: *const c_double,
    npts: c_int,
    height: c_double,
    plane: *const c_char,
    at_x: c_double,
    at_y: c_double,
    at_z: c_double,
    eager: c_int,
) -> c_int {
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };
    let at = if at_x == 0.0 && at_y == 0.0 && at_z == 0.0 {
        None
    } else {
        Some((at_x, at_y, at_z))
    };
    let pts_slice = unsafe { std::slice::from_raw_parts(pts, npts as usize) };
    let eager = eager != 0;
    catch_cad("rust_init_extrude_polygon", dest, || {
        cad::extrude_polygon_raw(pts_slice, height, &plane_str, at, eager)
    })
}

// ── Wire Operations ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_wire_to_face(
    dest: *mut c_void,
    data: *mut c_void,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_init_wire_to_face", dest, || {
        cad::wire_to_face(shape, eager != 0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_wire_fillet(
    dest: *mut c_void,
    data: *mut c_void,
    radius: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_init_wire_fillet", dest, || {
        cad::wire_fillet(shape, radius, eager != 0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_wire_chamfer(
    dest: *mut c_void,
    data: *mut c_void,
    distance: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_init_wire_chamfer", dest, || {
        cad::wire_chamfer(shape, distance, eager != 0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_wire_offset(
    dest: *mut c_void,
    data: *mut c_void,
    distance: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_init_wire_offset", dest, || {
        cad::wire_offset(shape, distance, eager != 0)
    })
}

// ── Helper Queries ────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_is_wire(data: *mut c_void) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    c_int::from(cad::is_wire(shape))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_is_face(data: *mut c_void) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    c_int::from(cad::is_face(shape))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_is_solid(data: *mut c_void) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    c_int::from(cad::is_solid(shape))
}

// ── Boolean operations — initialize at a pre-allocated destination ──────────

/// Subtract shape b from shape a, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_cut(
    dest: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_cut", dest, || {
        let shape_a = unsafe { &*(a as *const ShapeData) };
        let shape_b = unsafe { &*(b as *const ShapeData) };
        cad::cut(shape_a, shape_b, eager)
    })
}

/// Intersect shape a with shape b, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_common(
    dest: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_common", dest, || {
        let shape_a = unsafe { &*(a as *const ShapeData) };
        let shape_b = unsafe { &*(b as *const ShapeData) };
        cad::common(shape_a, shape_b, eager)
    })
}

/// Union shape a with shape b, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_fuse(
    dest: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_fuse", dest, || {
        let shape_a = unsafe { &*(a as *const ShapeData) };
        let shape_b = unsafe { &*(b as *const ShapeData) };
        cad::fuse(shape_a, shape_b, eager)
    })
}

/// Create a compound from multiple shapes, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_compound(
    dest: *mut c_void,
    shapes: *mut *mut c_void,
    num_shapes: c_int,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    let num = num_shapes as usize;
    catch_cad("rust_init_compound", dest, || {
        let shapes_slice = unsafe { std::slice::from_raw_parts(shapes as *const *mut c_void, num) };
        let shape_refs: Vec<&ShapeData> = shapes_slice
            .iter()
            .map(|p| unsafe { &*(*p as *const ShapeData) })
            .collect();
        cad::make_compound(&shape_refs, eager)
    })
}

/// Set a shape's render color (in-place mutation, no new shape).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_set_color(data: *mut c_void, r: c_double, g: c_double, b: c_double) {
    let shape_data = unsafe { &mut *(data as *mut ShapeData) };
    cad::set_color(shape_data, r, g, b);
}

/// Get a shape's render color. Returns 1 if set, 0 if nil.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_color(
    data: *mut c_void,
    r: *mut c_double,
    g: *mut c_double,
    b: *mut c_double,
) -> c_int {
    let shape_data = unsafe { &*(data as *const ShapeData) };
    match cad::get_color(shape_data) {
        Some(c) => {
            unsafe {
                *r = c[0];
                *g = c[1];
                *b = c[2];
            }
            1
        }
        None => 0,
    }
}

/// Translate a shape, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_translate(
    dest: *mut c_void,
    data: *mut c_void,
    dx: c_double,
    dy: c_double,
    dz: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_translate", dest, || {
        let shape = unsafe { &*(data as *const ShapeData) };
        cad::translate(shape, dx, dy, dz, eager)
    })
}

/// Rotate a shape, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_rotate(
    dest: *mut c_void,
    data: *mut c_void,
    ax: c_double,
    ay: c_double,
    az: c_double,
    angle: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_rotate", dest, || {
        let shape = unsafe { &*(data as *const ShapeData) };
        cad::rotate(shape, DVec3::new(ax, ay, az), angle, eager)
    })
}

/// Scale a shape, storing the result at dest.
/// Center pointer may be NULL (defaults to origin).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_scale(
    dest: *mut c_void,
    data: *mut c_void,
    factor: c_double,
    cx: *const c_double,
    cy: *const c_double,
    cz: *const c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_scale", dest, || {
        let shape = unsafe { &*(data as *const ShapeData) };
        let center = if cx.is_null() || cy.is_null() || cz.is_null() {
            DVec3::ZERO
        } else {
            unsafe { DVec3::new(*cx, *cy, *cz) }
        };
        cad::scale(shape, factor, center, eager)
    })
}

/// Mirror a shape, storing the result at dest.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_mirror(
    dest: *mut c_void,
    data: *mut c_void,
    ox: c_double,
    oy: c_double,
    oz: c_double,
    dx: c_double,
    dy: c_double,
    dz: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    catch_cad("rust_init_mirror", dest, || {
        let shape = unsafe { &*(data as *const ShapeData) };
        cad::mirror(shape, DVec3::new(ox, oy, oz), DVec3::new(dx, dy, dz), eager)
    })
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

/// Show a shape: tessellate if needed, register if not, set visible.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_show(data: *mut c_void) {
    let shape_data = unsafe { &mut *(data as *mut ShapeData) };
    shape_data.show();
}

/// Hide a shape: set visible flag to false.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_hide(data: *mut c_void) {
    let shape_data = unsafe { &mut *(data as *mut ShapeData) };
    shape_data.hide();
}

/// Remove a shape from the registry and mark it purged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_remove_from_registry(data: *mut c_void) {
    let shape_data = unsafe { &mut *(data as *mut ShapeData) };
    shape_data.remove_from_registry();
}

/// Get the visible flag from a shape.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_shape_get_visible(data: *mut c_void) -> c_int {
    let shape_data = unsafe { &*(data as *const ShapeData) };
    shape_data.visible as c_int
}

// ── Import ───────────────────────────────────────────────────────────────────

/// Read a shape from a STEP file, initializing at the given destination.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_read_step(
    dest: *mut c_void,
    path: *const c_char,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .to_string();
    catch_cad("rust_init_read_step", dest, || {
        cad::read_step(&path_str, eager)
    })
}

// ── Text ───────────────────────────────────────────────────────────────────

/// Create a 2D text shape (Face) from a string and font file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_text(
    dest: *mut c_void,
    text: *const c_char,
    font_path: *const c_char,
    size: c_double,
    plane: *const c_char,
    ax: c_double,
    ay: c_double,
    az: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    let text_str = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .to_string();
    let font_str = unsafe { CStr::from_ptr(font_path) }
        .to_string_lossy()
        .to_string();
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };

    catch_cad("rust_init_text", dest, || {
        let font = text::FontData::from_path(&font_str)?;
        let wp = cad::workplane_from_keyword(&plane_str, Some((ax, ay, az)));
        let shape = text::text_to_shape(&text_str, &font, size, &wp)?;
        let mut sd = ShapeData::new(shape);
        if eager {
            sd.tessellate_if_needed();
        }
        Ok(sd)
    })
}

/// Create an extruded 3D text shape (Solid) from a string and font file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_text_extruded(
    dest: *mut c_void,
    text: *const c_char,
    font_path: *const c_char,
    size: c_double,
    depth: c_double,
    both: c_int,
    plane: *const c_char,
    ax: c_double,
    ay: c_double,
    az: c_double,
    eager: c_int,
) -> c_int {
    let eager = eager != 0;
    let both = both != 0;
    let text_str = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .to_string();
    let font_str = unsafe { CStr::from_ptr(font_path) }
        .to_string_lossy()
        .to_string();
    let plane_str = if plane.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(plane) }
            .to_string_lossy()
            .to_string()
    };

    catch_cad("rust_init_text_extruded", dest, || {
        let font = text::FontData::from_path(&font_str)?;
        let wp = cad::workplane_from_keyword(&plane_str, Some((ax, ay, az)));
        let shape = text::text_to_solid(&text_str, &font, size, depth, both, &wp)?;
        let mut sd = ShapeData::new(shape);
        if eager {
            sd.tessellate_if_needed();
        }
        Ok(sd)
    })
}

/// List system fonts. Returns an array of "name|/path|:aspect" C strings.
/// Sets count_out to the number of entries. Caller must free with rust_free_fonts_list.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_list_fonts(count_out: *mut c_int) -> *mut *mut c_char {
    let fonts = text::list_system_fonts();
    let count = fonts.len();
    let mut entries: Vec<*mut c_char> = Vec::with_capacity(count);

    for (name, path, aspect) in fonts {
        let entry = format!("{}|{}|{}", name, path, aspect.as_str());
        if let Ok(cs) = CString::new(entry) {
            entries.push(cs.into_raw());
        }
    }

    let boxed = entries.into_boxed_slice();
    let ptr = boxed.as_ptr() as *mut *mut c_char;
    std::mem::forget(boxed);
    unsafe {
        ptr::write(count_out, count as c_int);
    }
    ptr
}

/// Free the font list allocated by rust_list_fonts.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_free_fonts_list(ptr: *mut *mut c_char, count: c_int) {
    if ptr.is_null() {
        return;
    }
    let count = count as usize;
    for i in 0..count {
        let p = unsafe { *ptr.add(i) };
        if !p.is_null() {
            unsafe {
                drop(CString::from_raw(p));
            }
        }
    }
    unsafe {
        let sl = std::ptr::slice_from_raw_parts_mut(ptr, count);
        drop(Box::<[*mut c_char]>::from_raw(sl));
    }
}

// ── Export ──────────────────────────────────────────────────────────────────

/// Write one or more shapes to a STEP file. Returns 0 on success, 1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_write_all_step(
    shapes: *mut *mut c_void,
    num_shapes: c_int,
    path: *const c_char,
) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .to_string();
    let num = num_shapes as usize;
    let shapes_slice = unsafe { std::slice::from_raw_parts(shapes as *const *mut c_void, num) };
    let shape_refs: Vec<&ShapeData> = shapes_slice
        .iter()
        .map(|p| unsafe { &*(*p as *const ShapeData) })
        .collect();
    match cad::write_all_step(&shape_refs, &path_str) {
        Ok(()) => 0,
        Err(msg) => {
            set_last_error(msg);
            1
        }
    }
}

/// Write one or more shapes to an STL file. Returns 0 on success, 1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_write_all_stl(
    shapes: *mut *mut c_void,
    num_shapes: c_int,
    path: *const c_char,
) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .to_string();
    let num = num_shapes as usize;
    let shapes_slice = unsafe { std::slice::from_raw_parts(shapes as *const *mut c_void, num) };
    let shape_refs: Vec<&ShapeData> = shapes_slice
        .iter()
        .map(|p| unsafe { &*(*p as *const ShapeData) })
        .collect();
    match cad::write_all_stl(&shape_refs, &path_str) {
        Ok(()) => 0,
        Err(msg) => {
            set_last_error(msg);
            1
        }
    }
}

// ── Shape Query FFI ───────────────────────────────────────────────────────────

/// Return an array of selected shape IDs and their count.
/// Caller must free the returned array with rust_free_u64_array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_selected_shape_ids(count_out: *mut usize) -> *mut u64 {
    let ids: Vec<u64> = types::get_selected_ids().into_iter().collect();
    let count = ids.len();
    let ptr = ids.as_ptr() as *mut u64;
    std::mem::forget(ids);
    if !count_out.is_null() {
        unsafe {
            *count_out = count;
        }
    }
    ptr
}

/// Return an array of registered shape IDs matching the given filter.
/// filter: 0 = all, 1 = visible only, 2 = hidden only.
/// Caller must free the returned array with rust_free_u64_array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_registered_shape_ids(
    filter: u8,
    count_out: *mut usize,
) -> *mut u64 {
    let registry = global_shape_registry();
    let entries = match filter {
        1 => registry.visible_shapes(),
        2 => registry.hidden_shapes(),
        _ => registry.all_shapes(),
    };
    let ids: Vec<u64> = entries.iter().map(|e| e.shape_id).collect();
    let count = ids.len();
    let ptr = ids.as_ptr() as *mut u64;
    std::mem::forget(ids);
    if !count_out.is_null() {
        unsafe {
            *count_out = count;
        }
    }
    ptr
}

/// Look up a ShapeData pointer by shape ID. Returns null if not found.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_shape_pointer(id: u64) -> *mut c_void {
    types::get_shape_pointer(id)
}

/// Free an array of u64 allocated by rust_get_selected_shape_ids
/// or rust_get_registered_shape_ids.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_free_u64_array(ptr: *mut u64, count: usize) {
    if !ptr.is_null() {
        unsafe {
            drop(Vec::from_raw_parts(ptr, count, count));
        }
    }
}

// ── Selection callback ───────────────────────────────────────────────────────

/// Poll for a pending selection event.
/// Returns 0 if no event, u64::MAX for deselected, or the selected shape ID.
/// Writes the action type (0=none, 1=toggled_on, 2=toggled_off, 3=cleared)
/// to the out-parameter pointed to by `action` if non-null.
/// Resets both atomics to 0 after reading.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_poll_selection(action: *mut u8) -> u64 {
    let id = LAST_SELECTION.swap(0, Ordering::SeqCst);
    if !action.is_null() {
        unsafe {
            *action = LAST_SELECTION_ACTION.swap(0, Ordering::SeqCst);
        }
    }
    id
}

/// Poll for a pending edge selection event.
/// Returns the edge index (-1 if no event pending).
/// Writes the shape ID to `shape_id_out` if non-null.
/// Writes the action (0=none, 4=selected, 5=deselected) to `action_out` if non-null.
/// Resets all three edge atomics after reading.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_poll_edge_selection(
    shape_id_out: *mut u64,
    action_out: *mut u8,
) -> i32 {
    let idx = LAST_EDGE_INDEX.swap(-1, Ordering::SeqCst);
    if !shape_id_out.is_null() {
        unsafe {
            *shape_id_out = LAST_EDGE_SHAPE_ID.swap(0, Ordering::SeqCst);
        }
    }
    if !action_out.is_null() {
        unsafe {
            *action_out = LAST_EDGE_ACTION.swap(0, Ordering::SeqCst);
        }
    }
    idx
}

/// Return a flat array of selected edge pairs [shape_id, edge_idx, shape_id, edge_idx, ...].
/// Caller must free the returned array with rust_free_u64_array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_selected_edge_ids(count_out: *mut usize) -> *mut u64 {
    let edges = types::get_selected_edges();
    let mut flat: Vec<u64> = Vec::new();
    for (&sid, indices) in &edges {
        for &idx in indices {
            flat.push(sid);
            flat.push(idx as u64);
        }
    }
    let count = flat.len();
    let ptr = flat.as_ptr() as *mut u64;
    std::mem::forget(flat);
    if !count_out.is_null() {
        unsafe {
            *count_out = count;
        }
    }
    ptr
}

/// Check if the application should quit (Ctrl+Q or window close).
/// Returns 1 if quit was requested, 0 otherwise. One-shot — resets the flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_quit_requested() -> c_int {
    c_int::from(QUIT_REQUESTED.swap(false, Ordering::SeqCst))
}

/// Sender for REPL→Viewer commands (fit-to-bounds, etc.).
static REPL_TO_VIEWER: OnceLock<mpsc::Sender<ReplToViewer>> = OnceLock::new();

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

// ── Back (hidden) edges visibility ────────────────────────────────────────────

/// Toggle back edge visibility. Returns new state (1 = showing, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_back_edges_toggle() -> c_int {
    let old = SHOW_BACK_EDGES.fetch_xor(true, Ordering::SeqCst);
    c_int::from(!old)
}

/// Query back edge visibility state (1 = showing, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_back_edges_showing() -> c_int {
    c_int::from(SHOW_BACK_EDGES.load(Ordering::SeqCst))
}

/// Set back edge visibility (0 = hidden, non-zero = showing).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_back_edges_set(value: c_int) {
    SHOW_BACK_EDGES.store(value != 0, Ordering::SeqCst);
}

// ── Projection mode toggle ────────────────────────────────────────────────────

/// Toggle projection mode. Returns new state (1 = perspective, 0 = orthographic).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_projection_perspective_toggle() -> c_int {
    let old = PROJECTION_PERSPECTIVE.fetch_xor(true, Ordering::SeqCst);
    c_int::from(!old)
}

/// Query projection mode (1 = perspective, 0 = orthographic).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_projection_perspective_showing() -> c_int {
    c_int::from(PROJECTION_PERSPECTIVE.load(Ordering::SeqCst))
}

/// Set projection mode (0 = orthographic, non-zero = perspective).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_projection_perspective_set(value: c_int) {
    PROJECTION_PERSPECTIVE.store(value != 0, Ordering::SeqCst);
}

// ── View angle ──────────────────────────────────────────────────────────────

/// Set camera to specific yaw/pitch angles, optionally with a distance.
/// If has_distance is false, the current camera radius is preserved.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_view_set_angles(
    yaw: f64,
    pitch: f64,
    has_distance: bool,
    distance: f64,
) {
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let dist = if has_distance { Some(distance) } else { None };
        let _ = tx.send(ReplToViewer::SetViewAngles {
            yaw,
            pitch,
            distance: dist,
        });
    }
}

// ── Stats overlay toggle ───────────────────────────────────────────────────

/// Toggle stats overlay. Returns new state (1 = visible, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_stats_overlay_toggle() -> c_int {
    let old = SHOW_STATS_OVERLAY.fetch_xor(true, Ordering::SeqCst);
    c_int::from(!old)
}

/// Query stats overlay visibility (1 = visible, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_stats_overlay_showing() -> c_int {
    c_int::from(SHOW_STATS_OVERLAY.load(Ordering::SeqCst))
}

/// Set stats overlay visibility (0 = hidden, non-zero = visible).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_stats_overlay_set(value: c_int) {
    SHOW_STATS_OVERLAY.store(value != 0, Ordering::SeqCst);
}

// ── Help overlay toggle ────────────────────────────────────────────────────

/// Toggle help window. Returns new state (1 = visible, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_help_overlay_toggle() -> c_int {
    let old = SHOW_HELP_OVERLAY.fetch_xor(true, Ordering::SeqCst);
    c_int::from(!old)
}

/// Query help window visibility (1 = visible, 0 = hidden).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_help_overlay_showing() -> c_int {
    c_int::from(SHOW_HELP_OVERLAY.load(Ordering::SeqCst))
}

/// Set help window visibility (0 = hidden, non-zero = visible).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_help_overlay_set(value: c_int) {
    SHOW_HELP_OVERLAY.store(value != 0, Ordering::SeqCst);
}

/// Set the Quick Example expression string from Janet at boot time.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_help_set_example(path: *const c_char) {
    let s = unsafe { CStr::from_ptr(path) }
        .to_str()
        .expect("help-set-example: invalid UTF-8");
    HELP_EXAMPLE.set(s.to_string()).ok();
}

// ── View fit ───────────────────────────────────────────────────────────────────

/// Fit camera to bounding box union of explicitly provided shapes.
/// `reset` — if true, reset to default isometric angle; otherwise keep current angle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_view_fit_shapes(shapes: *mut *mut c_void, count: c_int, reset: bool) {
    let shape_ptrs = unsafe { std::slice::from_raw_parts(shapes, count as usize) };
    let shape_refs: Vec<&ShapeData> = shape_ptrs
        .iter()
        .map(|&p| unsafe { &*(p as *const ShapeData) })
        .collect();

    if let Some((center, radius)) = cad::compute_union_bounds(&shape_refs)
        && let Some(tx) = REPL_TO_VIEWER.get()
    {
        let _ = tx.send(ReplToViewer::FitToBounds {
            center,
            radius,
            keep_angle: !reset,
        });
    }
}

/// Fit camera to bounding box union of all shapes (visible by default,
/// or all including hidden if `include_hidden` is true).
/// `reset` — if true, reset to default isometric angle; otherwise keep current angle.
/// If no shapes are found, always reset to default position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_view_fit_all(include_hidden: bool, reset: bool) {
    let registry = global_shape_registry();
    let entries = if include_hidden {
        registry.all_shapes()
    } else {
        registry.visible_shapes()
    };

    let mut min = DVec3::splat(f64::MAX);
    let mut max = DVec3::splat(f64::MIN);
    let mut has_vertices = false;

    for entry in &entries {
        if let Some(ref mesh) = entry.mesh {
            for v in &mesh.vertices {
                let p = DVec3::new(v[0] as f64, v[1] as f64, v[2] as f64);
                min = min.min(p);
                max = max.max(p);
                has_vertices = true;
            }
        }
    }

    let (center, radius, final_keep) = if has_vertices {
        (
            (min + max) * 0.5,
            (max - min).length() * 0.5 * 1.3,
            !reset, // invert: reset=false → keep angle (default)
        )
    } else {
        // No shapes found: always reset to default camera position
        (DVec3::ZERO, 50.0, false)
    };

    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::FitToBounds {
            center,
            radius,
            keep_angle: final_keep,
        });
    }
}

// ── Window size / fullscreen FFI ─────────────────────────────────────────────

/// Set viewer window size from Janet. Sets atomics optimistically then sends
/// a command via the REPL→Viewer channel.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_window_set_size(width: u32, height: u32) {
    WINDOW_WIDTH.store(width, Ordering::SeqCst);
    WINDOW_HEIGHT.store(height, Ordering::SeqCst);
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::SetWindowSize { width, height });
    }
}

/// Query current viewer window size. Returns [width, height] written to
/// the provided out-parameters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_window_size_query(out_width: *mut u32, out_height: *mut u32) {
    unsafe {
        *out_width = WINDOW_WIDTH.load(Ordering::SeqCst);
        *out_height = WINDOW_HEIGHT.load(Ordering::SeqCst);
    }
}

/// Set fullscreen mode from Janet. Sets the atomic then sends a command.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_window_set_fullscreen(fs: bool) {
    WINDOW_FULLSCREEN.store(fs, Ordering::SeqCst);
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::SetFullscreen(fs));
    }
}

/// Query current fullscreen state. Returns 1 if fullscreen, 0 if not.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_window_fullscreen_query() -> c_int {
    c_int::from(WINDOW_FULLSCREEN.load(Ordering::SeqCst))
}

/// Set maximized state from Janet. Sets the atomic then sends a command.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_window_set_maximized(mx: bool) {
    WINDOW_MAXIMIZED.store(mx, Ordering::SeqCst);
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::SetMaximized(mx));
    }
}

/// Query current maximized state. Returns 1 if maximized, 0 if not.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_window_maximized_query() -> c_int {
    c_int::from(WINDOW_MAXIMIZED.load(Ordering::SeqCst))
}

// ── Fillet / Chamfer FFI ──────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_fillet(
    dest: *mut c_void,
    data: *mut c_void,
    radius: c_double,
    idxs: *const c_int,
    count: c_int,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    let edge_indices: Option<Vec<usize>> = if idxs.is_null() || count <= 0 {
        None
    } else {
        let raw = unsafe { std::slice::from_raw_parts(idxs, count as usize) };
        Some(raw.iter().map(|&i| i as usize).collect())
    };
    catch_cad("rust_init_fillet", dest, || {
        cad::shape_fillet(shape, radius, edge_indices.as_deref(), eager != 0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_init_chamfer(
    dest: *mut c_void,
    data: *mut c_void,
    distance: c_double,
    idxs: *const c_int,
    count: c_int,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    let edge_indices: Option<Vec<usize>> = if idxs.is_null() || count <= 0 {
        None
    } else {
        let raw = unsafe { std::slice::from_raw_parts(idxs, count as usize) };
        Some(raw.iter().map(|&i| i as usize).collect())
    };
    catch_cad("rust_init_chamfer", dest, || {
        cad::shape_chamfer(shape, distance, edge_indices.as_deref(), eager != 0)
    })
}

// ── Face Info FFI ──────────────────────────────────────────────────────────────

/// Get face info JSON for a shape. Caller must free with rust_face_info_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_face_info(data: *mut c_void) -> *mut c_char {
    if data.is_null() {
        return CString::new("[]").unwrap().into_raw();
    }
    let shape = unsafe { &*(data as *const ShapeData) };
    let json = cad::face_info_json(shape);
    CString::new(json)
        .unwrap_or_else(|_| CString::new("[]").unwrap())
        .into_raw()
}

/// Free a string returned by rust_face_info.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_face_info_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

// ── Get Face / Edge FFI ────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_face(
    dest: *mut c_void,
    data: *mut c_void,
    index: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_get_face", dest, || {
        cad::get_nth_face(shape, index as usize)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_get_edge(
    dest: *mut c_void,
    data: *mut c_void,
    index: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_get_edge", dest, || {
        cad::get_nth_edge(shape, index as usize)
    })
}

// ── Face Operation FFI ─────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_face_offset(
    dest: *mut c_void,
    data: *mut c_void,
    distance: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_face_offset", dest, || {
        cad::face_offset(shape, distance, eager != 0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_face_fillet(
    dest: *mut c_void,
    data: *mut c_void,
    radius: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_face_fillet", dest, || {
        cad::face_fillet(shape, radius, eager != 0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_face_chamfer(
    dest: *mut c_void,
    data: *mut c_void,
    distance: c_double,
    eager: c_int,
) -> c_int {
    let shape = unsafe { &*(data as *const ShapeData) };
    catch_cad("rust_face_chamfer", dest, || {
        cad::face_chamfer(shape, distance, eager != 0)
    })
}

/// Create a sketch from a face's workplane. Returns a rojcad/sketch abstract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_face_workplane(dest: *mut c_void, data: *mut c_void) -> c_int {
    if data.is_null() {
        return 1;
    }
    let shape = unsafe { &*(data as *const ShapeData) };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let face = shape.shape.expect_face();
        let wp = face.workplane();
        let sk = sketch::SketchData::new(wp);
        unsafe {
            ptr::write(dest as *mut sketch::SketchData, sk);
        }
    }));
    match result {
        Ok(()) => 0,
        Err(_) => {
            set_last_error("face-workplane: expected a Face shape".into());
            1
        }
    }
}

// ── Edge Info FFI ──────────────────────────────────────────────────────────────

/// Get edge info JSON for a shape. Caller must free with rust_edge_info_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_info(data: *mut c_void) -> *mut c_char {
    if data.is_null() {
        return CString::new("[]").unwrap().into_raw();
    }
    let shape = unsafe { &*(data as *const ShapeData) };
    let json = cad::edge_info_json(shape);
    CString::new(json)
        .unwrap_or_else(|_| CString::new("[]").unwrap())
        .into_raw()
}

/// Free a string returned by rust_edge_info.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_edge_info_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

// ── Highlight FFI ────────────────────────────────────────────────────────────

/// Send a highlight command to the viewer for the given shape.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_highlight_shape(data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let shape_data = unsafe { &*(data as *const ShapeData) };
    let id = shape_data.shape_id;
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::HighlightShape { id });
    }
}

/// Send a clear-highlight command to the viewer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_highlight_clear() {
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::ClearHighlight);
    }
}

/// Highlight specific edges of a shape in the viewer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_highlight_edges(data: *mut c_void, idxs: *const c_int, count: c_int) {
    if data.is_null() {
        return;
    }
    let shape_data = unsafe { &*(data as *const ShapeData) };
    let id = shape_data.shape_id;
    let indices = if count > 0 && !idxs.is_null() {
        let raw = unsafe { std::slice::from_raw_parts(idxs, count as usize) };
        raw.iter().map(|&i| i as usize).collect()
    } else {
        Vec::new()
    };
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::HighlightEdges { id, indices });
    }
}

/// Clear all edge highlights in the viewer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_highlight_edges_clear() {
    if let Some(tx) = REPL_TO_VIEWER.get() {
        let _ = tx.send(ReplToViewer::ClearEdgeHighlight);
    }
}

// ── C bridge registration forward declaration ────────────────────────────────

unsafe extern "C" {
    fn cad_register_functions(env: *mut bridge::JanetTable);
}

// ── Main ─────────────────────────────────────────────────────────────────────

macro_rules! make_port_parser {
    ($name:ident, $flag:expr) => {
        fn $name() -> Option<u16> {
            let eq_flag = format!("{}=", $flag);
            let mut args = std::env::args().peekable();
            while let Some(arg) = args.next() {
                if let Some(port_str) = arg.strip_prefix(&eq_flag) {
                    let p: u16 = port_str.parse().unwrap_or_else(|_| {
                        eprintln!("rojcad: invalid {} '{}'", $flag, port_str);
                        std::process::exit(1);
                    });
                    if !(1..=65535).contains(&p) {
                        eprintln!("rojcad: {} must be between 1 and 65535, got {}", $flag, p);
                        std::process::exit(1);
                    }
                    return Some(p);
                }
                if arg.as_str() == $flag {
                    let next = args.next().unwrap_or_else(|| {
                        eprintln!("rojcad: {} requires a value", $flag);
                        std::process::exit(1);
                    });
                    let p: u16 = next.parse().unwrap_or_else(|_| {
                        eprintln!("rojcad: invalid {} '{}'", $flag, next);
                        std::process::exit(1);
                    });
                    if !(1..=65535).contains(&p) {
                        eprintln!("rojcad: {} must be between 1 and 65535, got {}", $flag, p);
                        std::process::exit(1);
                    }
                    return Some(p);
                }
            }
            None
        }
    };
}

make_port_parser!(parse_spork_port_arg, "--spork-port");
make_port_parser!(parse_raw_port_arg, "--raw-port");

fn parse_eval_args() -> Vec<String> {
    let mut exprs = Vec::new();
    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        if let Some(e) = arg.strip_prefix("--eval=") {
            exprs.push(e.to_string());
        }
        if arg == "--eval" {
            let next = args.next().unwrap_or_else(|| {
                eprintln!("rojcad: --eval requires an argument");
                std::process::exit(1);
            });
            exprs.push(next);
        }
    }
    exprs
}

fn parse_size_args() -> (Option<u32>, Option<u32>) {
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        if let Some(w) = arg.strip_prefix("--width=") {
            width = Some(w.parse().unwrap_or_else(|_| {
                eprintln!("rojcad: invalid width '{}'", w);
                std::process::exit(1);
            }));
        }
        if arg == "--width" {
            let next = args.next().unwrap_or_else(|| {
                eprintln!("rojcad: --width requires a value");
                std::process::exit(1);
            });
            width = Some(next.parse().unwrap_or_else(|_| {
                eprintln!("rojcad: invalid width '{}'", next);
                std::process::exit(1);
            }));
        }
        if let Some(h) = arg.strip_prefix("--height=") {
            height = Some(h.parse().unwrap_or_else(|_| {
                eprintln!("rojcad: invalid height '{}'", h);
                std::process::exit(1);
            }));
        }
        if arg == "--height" {
            let next = args.next().unwrap_or_else(|| {
                eprintln!("rojcad: --height requires a value");
                std::process::exit(1);
            });
            height = Some(next.parse().unwrap_or_else(|_| {
                eprintln!("rojcad: invalid height '{}'", next);
                std::process::exit(1);
            }));
        }
    }
    (width, height)
}

fn parse_repl_visibility() -> Option<bool> {
    for arg in std::env::args() {
        if arg == "--repl" {
            return Some(true);
        }
        if arg == "--no-repl" {
            return Some(false);
        }
    }
    None
}

fn main() {
    // Initialize crash diagnostics (signal handler, panic hook, crash log file)
    crash_handler::init();

    // Parse CLI arguments
    let headless: bool = std::env::args().any(|arg| arg == "--headless");
    let spork_port: u16 = parse_spork_port_arg().unwrap_or(9365);
    let raw_port: u16 = parse_raw_port_arg().unwrap_or(9364);

    let eval_exprs: Vec<String> = parse_eval_args();
    let (cli_width, cli_height) = parse_size_args();
    let maximized = cli_width.is_none() && cli_height.is_none();

    // Apply --repl/--no-repl CLI flags to the visibility atomic
    if let Some(visible) = parse_repl_visibility() {
        SHOW_REPL_PANEL.store(visible, Ordering::SeqCst);
    }

    let viewer_config = ViewerConfig {
        width: cli_width.unwrap_or(1024),
        height: cli_height.unwrap_or(768),
        maximized,
    };

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
        bridge::janet_lib_asm(env);
        bridge::janet_lib_spork_json(env);
    }

    // Register CAD functions
    unsafe {
        cad_register_functions(env);
    }

    // GUI REPL FFI functions are now registered via bridge.c JANET_FN wrappers
    // (gui-repl-poll-request and gui-repl-send-response), registered in
    // cad_register_functions().

    // Port values are injected directly into boot.janet via a prefix string
    // (see boot code assembly below).

    // Create channel for REPL→Viewer commands
    let (repl_tx, repl_rx) = mpsc::channel::<ReplToViewer>();
    let _ = REPL_TO_VIEWER.set(repl_tx);

    // Create channels for GUI REPL panel (viewer ↔ REPL thread)
    let (gui_req_tx, gui_resp_rx) = gui_repl::init();

    // Start viewer thread unless --headless flag is present
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    let _viewer_handle = if !headless {
        Some(viewer::spawn_viewer(
            repl_rx,
            gui_req_tx,
            gui_resp_rx,
            viewer_config,
        ))
    } else {
        None
    };

    // Embed and run upstream.janet (standard Janet macros) before boot.janet,
    // so that rojcad boot code can optionally use macros like defn, each, ->, etc.
    // Pre-define boot/args and boot/config symbols (referenced by upstream.janet for
    // CLI arg parsing and core image generation — not used in rojcad's embedded build).
    let upstream_base = include_str!("../upstream.janet");
    let upstream_code = format!("(def boot/args @[\"rojcad\" \"\" \"\"])\n{}", upstream_base);
    let upstream_c = CString::new(upstream_code).unwrap_or_else(|_| CString::new("").unwrap());
    let upstream_name_c = CString::new("upstream.janet").unwrap();

    let mut result = bridge::Janet(0);
    let status = unsafe {
        bridge::janet_dostring(
            env,
            upstream_c.as_ptr(),
            upstream_name_c.as_ptr(),
            &mut result,
        )
    };

    if status != 0 {
        eprintln!("rojcad: failed to load upstream.janet");
        unsafe {
            bridge::janet_deinit();
        }
        std::process::exit(1);
    }

    // Load vendored spork modules — concatenated into one source to avoid
    // Janet module-resolution issues (bootstrap mode + janet_dostring).
    // msg.janet and ev-utils.janet provide the low-level protocol helpers;
    // netrepl-server.janet provides the server entry points.
    // The client/getline/rawterm parts of spork are excluded (not needed server-side).
    let spork_source = concat!(
        include_str!("../vendor/spork/msg.janet"),
        "\n",
        include_str!("../vendor/spork/ev-utils.janet"),
        "\n",
        include_str!("../vendor/spork/netrepl-server.janet"),
        "\n(def netrepl/server server)\n(def netrepl/server-single server-single)\n(def netrepl/run-server run-server)\n(def netrepl/run-server-single run-server-single)",
    );
    let spork_c = CString::new(spork_source).unwrap_or_else(|_| CString::new("").unwrap());
    let spork_name_c = CString::new("vendor/spork/spork.janet").unwrap();
    let mut spork_result = bridge::Janet(0);
    let spork_status = unsafe {
        bridge::janet_dostring(
            env,
            spork_c.as_ptr(),
            spork_name_c.as_ptr(),
            &mut spork_result,
        )
    };
    if spork_status != 0 {
        eprintln!("rojcad: warning: failed to load vendored spork");
    }

    // Embed and run boot.janet (rojcad REPL server)
    // Prefix the port values as global defs so boot.janet can read them.
    let boot_base = include_str!("../boot.janet");

    // Embed and append model.janet (parametric model runtime)
    let model_base = include_str!("../boot/model.janet");
    let version = env!("CARGO_PKG_VERSION");
    let version_suffix = if !std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| o.stdout.is_empty())
        .unwrap_or(false)
    {
        "-dirty"
    } else {
        ""
    };
    let boot_prefix = format!(
        "(def *rojcad-version* {:?})\n(def *rojcad-os* {:?})\n(def *raw-repl-port* {})\n(def *spork-repl-port* {})\n",
        format!("{}{}", version, version_suffix),
        std::env::consts::OS,
        raw_port,
        spork_port
    );
    let boot_with_model = format!("{}{}\n\n{}\n", boot_prefix, boot_base, model_base);

    let boot_code = if !eval_exprs.is_empty() {
        // Append --eval expression(s) as raw Janet code at end of boot.
        format!("{}\n\n{}\n", boot_with_model, eval_exprs.join("\n"))
    } else {
        boot_with_model
    };
    let boot_c = CString::new(boot_code).unwrap_or_else(|_| CString::new("").unwrap());
    let boot_name_c = CString::new("boot.janet").unwrap();

    let mut result = bridge::Janet(0);
    let status =
        unsafe { bridge::janet_dostring(env, boot_c.as_ptr(), boot_name_c.as_ptr(), &mut result) };

    if status != 0 {
        eprintln!("rojcad: failed to load boot.janet");
        unsafe {
            bridge::janet_deinit();
        }
        std::process::exit(1);
    }

    // The event loop runs automatically via the Janet VM.
    // The two-phase boot loads upstream macros first, then rojcad boot code.
    // boot.janet has a (forever ...) loop that blocks indefinitely.
    // If we reach here (shouldn't under normal operation), clean up.
    unsafe {
        bridge::janet_deinit();
    }
    crash_handler::cleanup();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catch_cad_error() {
        let result = catch_cad("test_fn", std::ptr::null_mut(), || {
            Err("invalid dimension".to_string())
        });
        assert_eq!(result, 1);
        assert_eq!(take_last_error(), "invalid dimension");
    }

    #[test]
    fn test_catch_cad_panic_str() {
        let result = catch_cad("test_fn", std::ptr::null_mut(), || {
            panic!("oops");
        });
        assert_eq!(result, 1);
        let err = take_last_error();
        assert!(err.contains("test_fn panicked: oops"), "got: {err}");
    }

    #[test]
    fn test_catch_cad_panic_unknown() {
        let result = catch_cad("test_fn", std::ptr::null_mut(), || {
            std::panic::panic_any(42);
        });
        assert_eq!(result, 1);
        let err = take_last_error();
        assert!(err.contains("test_fn panicked: (unknown)"), "got: {err}");
    }

    #[test]
    fn test_catch_result_success() {
        let result = catch_result("test_fn", || Ok(()));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_catch_result_error() {
        let result = catch_result("test_fn", || Err("write failed".to_string()));
        assert_eq!(result, 1);
        assert_eq!(take_last_error(), "write failed");
    }

    #[test]
    fn test_catch_result_panic() {
        let result = catch_result("test_fn", || {
            panic!("result panic");
        });
        assert_eq!(result, 1);
        let err = take_last_error();
        assert!(err.contains("test_fn panicked: result panic"), "got: {err}");
    }
}
