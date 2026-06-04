//! Rust `extern "C"` bridge layer — Janet C API function declarations
//! and Rust callbacks invoked from bridge/bridge.c.
//!
//! Most items here are only referenced by the C linker, not called directly
//! from Rust code, so we suppress false-positive unused warnings.

#![allow(dead_code, non_camel_case_types, non_snake_case)]

use std::ffi::{c_char, c_double, c_int, c_void};

pub type c_size_t = usize;

// ── Janet C API types ─────────────────────────────────────────────────────────

#[repr(C)]
pub struct Janet(pub u64);

#[repr(C)]
pub struct JanetFunction(*mut c_void);

#[repr(C)]
pub struct JanetTable(*mut c_void);

#[repr(C)]
pub struct JanetArray(*mut c_void);

#[repr(C)]
pub struct JanetKV(*mut c_void);

#[repr(C)]
pub struct JanetBuffer(*mut c_void);

#[repr(C)]
pub struct JanetMarshalContext(*mut c_void);

pub type JanetCFunction = unsafe extern "C" fn(
    argc: i32,
    argv: *const Janet,
) -> Janet;

// ── Janet C API externs ───────────────────────────────────────────────────────

unsafe extern "C" {
    pub fn janet_init();
    pub fn janet_deinit();
    pub fn janet_core_env(tail: *const Janet) -> *mut JanetTable;
    pub fn janet_dostring(
        env: *mut JanetTable,
        str: *const c_char,
        name: *const c_char,
        result: *mut Janet,
    ) -> c_int;

    pub fn janet_cfuns(
        env: *mut JanetTable,
        prefix: *const c_char,
        cfuns: *const JanetReg,
    );

    pub fn janet_abstract(abstract_type: *const JanetAbstractType, size: c_size_t) -> *mut c_void;
    pub fn janet_checkabstract(x: Janet, at: *const JanetAbstractType) -> *mut c_void;
    pub fn janet_checktype(x: Janet, t: c_int) -> c_int;
    pub fn janet_type(x: Janet) -> c_int;
    pub fn janet_wrap_number(x: c_double) -> Janet;
    pub fn janet_wrap_string(s: *const u8) -> Janet;
    pub fn janet_wrap_keyword(s: *const u8) -> Janet;
    pub fn janet_wrap_true() -> Janet;
    pub fn janet_wrap_false() -> Janet;
    pub fn janet_wrap_nil() -> Janet;

    pub fn janet_unwrap_number(x: Janet) -> f64;
    pub fn janet_unwrap_keyword(x: Janet) -> *const u8;
    pub fn janet_unwrap_string(x: Janet) -> *const u8;
    pub fn janet_unwrap_abstract(x: Janet) -> *mut c_void;
    pub fn janet_unwrap_tuple(x: Janet) -> *const Janet;

    pub fn janet_ckeyword(s: *const c_char) -> *mut c_void;
    pub fn janet_ckeywordv(cstr: *const c_char) -> Janet;

    pub fn janet_checkint(x: Janet) -> c_int;

    pub fn janet_panic(msg: *const c_char);

    pub fn janet_arity(argc: i32, min: i32, max: i32);

    pub fn janet_tuple_n(parts: *const Janet, n: usize) -> *mut c_void;
    pub fn janet_tuple_length(t: *const Janet) -> usize;

    pub fn janet_buffer_push_cstring(buffer: *mut c_void, s: *const c_char);

    pub fn janet_setdyn(name: *const c_char, value: Janet);
}

// ── Janet library registration functions ──────────────────────────────────────

unsafe extern "C" {
    pub fn janet_lib_io(env: *mut JanetTable);
    pub fn janet_lib_math(env: *mut JanetTable);
    pub fn janet_lib_array(env: *mut JanetTable);
    pub fn janet_lib_tuple(env: *mut JanetTable);
    pub fn janet_lib_buffer(env: *mut JanetTable);
    pub fn janet_lib_table(env: *mut JanetTable);
    pub fn janet_lib_struct(env: *mut JanetTable);
    pub fn janet_lib_fiber(env: *mut JanetTable);
    pub fn janet_lib_os(env: *mut JanetTable);
    pub fn janet_lib_parse(env: *mut JanetTable);
    pub fn janet_lib_compile(env: *mut JanetTable);
    pub fn janet_lib_debug(env: *mut JanetTable);
    pub fn janet_lib_string(env: *mut JanetTable);
    pub fn janet_lib_marsh(env: *mut JanetTable);
    pub fn janet_lib_net(env: *mut JanetTable);
    pub fn janet_lib_ev(env: *mut JanetTable);
}

// ── Rust callback declarations (called from C) ───────────────────────────────

unsafe extern "C" {
    pub fn rust_shape_data_size() -> usize;
    pub fn rust_shape_drop(data: *mut c_void, len: c_size_t);
    pub fn rust_shape_type_string(data: *mut c_void) -> *const c_char;

    pub fn rust_init_box(
        dest: *mut c_void,
        width: c_double,
        depth: c_double,
        height: c_double,
        cx: *const c_double,
        cy: *const c_double,
        cz: *const c_double,
    );

    pub fn rust_init_sphere(
        dest: *mut c_void,
        radius: c_double,
        cx: *const c_double,
        cy: *const c_double,
        cz: *const c_double,
        angle: *const c_double,
    );

    pub fn rust_init_cube(
        dest: *mut c_void,
        size: c_double,
        cx: *const c_double,
        cy: *const c_double,
        cz: *const c_double,
    );

    pub fn rust_init_box_from_corners(
        dest: *mut c_void,
        c1x: c_double, c1y: c_double, c1z: c_double,
        c2x: c_double, c2y: c_double, c2z: c_double,
    );

    pub fn rust_init_cylinder(
        dest: *mut c_void,
        radius: c_double,
        height: c_double,
        cx: *const c_double,
        cy: *const c_double,
        cz: *const c_double,
    );

    pub fn rust_init_cylinder_from_points(
        dest: *mut c_void,
        p1x: c_double, p1y: c_double, p1z: c_double,
        p2x: c_double, p2y: c_double, p2z: c_double,
        radius: c_double,
    );

    pub fn rust_init_cylinder_point_dir(
        dest: *mut c_void,
        px: c_double, py: c_double, pz: c_double,
        radius: c_double,
        dx: c_double, dy: c_double, dz: c_double,
        height: c_double,
    );

    pub fn rust_init_cone(
        dest: *mut c_void,
        bottom_radius: c_double,
        top_radius: c_double,
        height: c_double,
        cx: *const c_double,
        cy: *const c_double,
        cz: *const c_double,
        angle: *const c_double,
    );

    pub fn rust_init_torus(
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
    );

    pub fn rust_init_cut(dest: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn rust_init_common(dest: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn rust_init_fuse(dest: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn rust_init_translate(
        dest: *mut c_void,
        data: *mut c_void,
        dx: c_double,
        dy: c_double,
        dz: c_double,
    );
    pub fn rust_init_rotate(
        dest: *mut c_void,
        data: *mut c_void,
        ax: c_double,
        ay: c_double,
        az: c_double,
        angle: c_double,
    );
    pub fn rust_init_scale(
        dest: *mut c_void,
        data: *mut c_void,
        factor: c_double,
        cx: *const c_double,
        cy: *const c_double,
        cz: *const c_double,
    );
    pub fn rust_init_mirror(
        dest: *mut c_void,
        data: *mut c_void,
        ox: c_double,
        oy: c_double,
        oz: c_double,
        dx: c_double,
        dy: c_double,
        dz: c_double,
    );

    pub fn rust_shape_type(data: *mut c_void) -> *const c_char;

    pub fn rust_write_step(data: *mut c_void, path: *const c_char) -> c_int;
    pub fn rust_write_stl(data: *mut c_void, path: *const c_char) -> c_int;

    pub fn rust_shape_set_visible(data: *mut c_void, visible: c_int);
    pub fn rust_shape_get_visible(data: *mut c_void) -> c_int;
}

// ── JanetReg struct ───────────────────────────────────────────────────────────

#[repr(C)]
pub struct JanetReg {
    pub name: *const c_char,
    pub cfunction: JanetCFunction,
    pub documentation: *const c_char,
}

// ── Abstract type descriptor ──────────────────────────────────────────────────

#[repr(C)]
pub struct JanetAbstractType {
    pub name: *const c_char,
    pub gc: Option<unsafe extern "C" fn(*mut c_void, c_size_t) -> c_int>,
    pub gcmark: Option<unsafe extern "C" fn(*mut c_void, c_size_t) -> c_int>,
    pub get: Option<unsafe extern "C" fn(*mut c_void, Janet, *mut Janet) -> c_int>,
    pub put: Option<unsafe extern "C" fn(*mut c_void, Janet, Janet)>,
    pub marshal: Option<unsafe extern "C" fn(*mut c_void, *mut JanetMarshalContext)>,
    pub unmarshal: Option<unsafe extern "C" fn(*mut JanetMarshalContext) -> *mut c_void>,
    pub tostring: Option<unsafe extern "C" fn(*mut c_void, *mut JanetBuffer)>,
    pub compare: Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int>,
    pub hash: Option<unsafe extern "C" fn(*mut c_void, c_size_t) -> i32>,
    pub next: Option<unsafe extern "C" fn(*mut c_void, Janet) -> Janet>,
    pub call: Option<unsafe extern "C" fn(*mut c_void, i32, *mut Janet) -> Janet>,
    pub length: Option<unsafe extern "C" fn(*mut c_void, c_size_t) -> usize>,
    pub bytes: Option<unsafe extern "C" fn(*mut c_void, c_size_t) -> JanetByteView>,
    pub gcperthread: Option<unsafe extern "C" fn(*mut c_void, c_size_t) -> c_int>,
}

#[repr(C)]
pub struct JanetAbstractHead {
    pub _type: *const JanetAbstractType,
    _data: [u8; 0],
}

#[repr(C)]
pub struct JanetByteView {
    pub items: *const u8,
    pub len: i32,
}
