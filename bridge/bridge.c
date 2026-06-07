/**
 * bridge.c — C glue layer between Janet C API and Rust CAD functions.
 *
 * 1. Defines the `rojcad/shape` Janet abstract type with finalizer + tostring
 * 2. Implements JANET_FN wrappers for each CAD operation
 * 3. Provides `cad_register_functions(env)` called from Rust main
 *
 * Compatible with Janet 1.41+ API.
 */

#include <janet.h>
#include <stdint.h>
#include <string.h>

/* ── Forward declarations of Rust extern "C" functions ──────────────────── */

/* Return the size of ShapeData (Rust type) for janet_abstract allocation */
extern size_t rust_shape_data_size(void);

/* Shape lifecycle */
extern void rust_shape_drop(void *data, size_t len);
extern const char *rust_shape_type_string(void *data);

/* Retrieve the last CAD error message as a C string (caller frees) */
extern const char *rust_take_last_error(void);

/* Shape constructors — return 0 on success, 1 on error */
extern int rust_init_box(void *dest,
                           double width, double depth, double height,
                           const double *cx, const double *cy, const double *cz,
                           int eager);
extern int rust_init_cube(void *dest,
                             double size,
                             const double *cx, const double *cy, const double *cz,
                             int eager);
extern int rust_init_box_from_corners(void *dest,
                                         double c1x, double c1y, double c1z,
                                         double c2x, double c2y, double c2z,
                                         int eager);
extern int rust_init_sphere(void *dest,
                               double radius,
                               const double *cx, const double *cy, const double *cz,
                               const double *angle,
                               int eager);
extern int rust_init_cylinder(void *dest,
                                 double radius, double height,
                                 const double *cx, const double *cy, const double *cz,
                                 int eager);
extern int rust_init_cylinder_from_points(void *dest,
                                             double p1x, double p1y, double p1z,
                                             double p2x, double p2y, double p2z,
                                             double radius,
                                             int eager);
extern int rust_init_cylinder_point_dir(void *dest,
                                           double px, double py, double pz,
                                           double radius,
                                           double dx, double dy, double dz,
                                           double height,
                                           int eager);
extern int rust_init_cone(void *dest,
                             double bottom_radius, double top_radius, double height,
                             const double *cx, const double *cy, const double *cz,
                             const double *angle,
                             int eager);
extern int rust_init_torus(void *dest,
                              double ring_radius, double tube_radius,
                              const double *cx, const double *cy, const double *cz,
                              const double *zx, const double *zy, const double *zz,
                              const double *angle,
                              const double *angle_start, const double *angle_end,
                              int eager);

/* Boolean operations */
extern int rust_init_cut(void *dest, void *a, void *b, int eager);
extern int rust_init_common(void *dest, void *a, void *b, int eager);
extern int rust_init_fuse(void *dest, void *a, void *b, int eager);

/* Transformation operations */
extern int rust_init_translate(void *dest, void *data, double dx, double dy, double dz, int eager);
extern int rust_init_rotate(void *dest, void *data, double ax, double ay, double az, double angle, int eager);
extern int rust_init_scale(void *dest, void *data, double factor, const double *cx, const double *cy, const double *cz, int eager);
extern int rust_init_mirror(void *dest, void *data, double ox, double oy, double oz, double dx, double dy, double dz, int eager);

/* Inspection */
extern const char *rust_shape_type(void *data);

/* Import */
extern int rust_init_read_step(void *dest, const char *path, int eager);

/* Export */
extern int rust_write_step(void *data, const char *path);
extern int rust_write_stl(void *data, const char *path);

/* Visibility */
extern void rust_shape_show(void *data);
extern void rust_shape_hide(void *data);
extern void rust_shape_remove_from_registry(void *data);
extern int rust_shape_get_visible(void *data);

/* Selection */
extern uint64_t rust_poll_selection(uint8_t *action);

/* Shape queries */
extern uint64_t *rust_get_selected_shape_ids(size_t *count_out);
extern uint64_t *rust_get_registered_shape_ids(uint8_t filter, size_t *count_out);
extern void *rust_get_shape_pointer(uint64_t id);
extern void rust_free_u64_array(uint64_t *ptr, size_t count);

/* Quit request */
extern int rust_quit_requested(void);

/* Edge visibility toggles */
extern int rust_edge_toggle_inactive(void);
extern int rust_edge_toggle_active(void);
extern int rust_edge_inactive_showing(void);
extern int rust_edge_active_showing(void);

/* Edge style (thickness / color) */
extern double rust_edge_get_thickness(void);
extern void rust_edge_set_thickness(double value);
extern void rust_edge_set_color_inactive(double r, double g, double b);
extern void rust_edge_set_color_active(double r, double g, double b);

extern int rust_back_edges_toggle(void);
extern int rust_back_edges_showing(void);
extern void rust_back_edges_set(int value);
extern int rust_projection_perspective_toggle(void);
extern int rust_projection_perspective_showing(void);
extern void rust_projection_perspective_set(int value);

/* Stats overlay */
extern int rust_stats_overlay_toggle(void);
extern int rust_stats_overlay_showing(void);
extern void rust_stats_overlay_set(int value);

/* Help overlay */
extern int rust_help_overlay_toggle(void);
extern int rust_help_overlay_showing(void);

/* View angle */
extern void rust_view_set_angles(double yaw, double pitch, int has_distance, double distance);
extern void rust_help_overlay_set(int value);

/* View fit */
extern void rust_view_fit_shapes(void **shapes, int count, int reset);
extern void rust_view_fit_all(int include_hidden, int reset);

/* Window size / fullscreen / maximized */
extern void rust_window_set_size(uint32_t width, uint32_t height);
extern void rust_window_size_query(uint32_t *out_width, uint32_t *out_height);
extern void rust_window_set_fullscreen(int fs);
extern int  rust_window_fullscreen_query(void);
extern void rust_window_set_maximized(int mx);
extern int  rust_window_maximized_query(void);

/* 2D primitives */
extern int rust_init_rect(void *dest, double w, double d, int is_wire,
                            const char *plane, double ax, double ay, double az, int eager);
extern int rust_init_circle(void *dest, double r, int is_wire,
                             const char *plane, double ax, double ay, double az, int eager);
extern int rust_init_polygon(void *dest, const double *pts, int npts, int is_wire,
                              const char *plane, double ax, double ay, double az, int eager);

/* Extrusion / Revolution */
extern int rust_init_extrude(void *dest, void *data, double height,
                              double dx, double dy, double dz, int both, int eager);
extern int rust_init_revolve(void *dest, void *data, double angle,
                              double ox, double oy, double oz,
                              double dx, double dy, double dz, int eager);
extern int rust_init_extrude_polygon(void *dest, const double *pts, int npts, double height,
                                      const char *plane, double ax, double ay, double az, int eager);

/* Wire operations */
extern int rust_init_wire_to_face(void *dest, void *data, int eager);
extern int rust_init_wire_fillet(void *dest, void *data, double radius, int eager);
extern int rust_init_wire_chamfer(void *dest, void *data, double distance, int eager);
extern int rust_init_wire_offset(void *dest, void *data, double distance, int eager);

/* Sketch */
extern size_t rust_sketch_data_size(void);
extern void rust_sketch_drop(void *data, size_t len);
extern void rust_sketch_new(void *dest, const char *plane, double ax, double ay, double az);
extern void rust_sketch_move_to(void *dest, void *src, double x, double y);
extern void rust_sketch_line_to(void *dest, void *src, double x, double y);
extern void rust_sketch_line_dx(void *dest, void *src, double dx);
extern void rust_sketch_line_dy(void *dest, void *src, double dy);
extern void rust_sketch_line_dx_dy(void *dest, void *src, double dx, double dy);
extern void rust_sketch_arc_to(void *dest, void *src, double x2, double y2, double x3, double y3);
extern void rust_sketch_close(void *shape_dest, void *src);
extern void rust_sketch_build_wire(void *shape_dest, void *src);

/* Helper queries */
extern int rust_is_wire(void *data);
extern int rust_is_face(void *data);
extern int rust_is_solid(void *data);

/* Text */
extern int rust_init_text(void *dest, const char *text, const char *font_path,
                           double size, const char *plane,
                           double ax, double ay, double az, int eager);
extern int rust_init_text_extruded(void *dest, const char *text, const char *font_path,
                                    double size, double depth, int both,
                                    const char *plane,
                                    double ax, double ay, double az, int eager);
extern char **rust_list_fonts(int *count_out);
extern void rust_free_fonts_list(char **list, int count);

/* ── Abstract type definition ───────────────────────────────────────────── */

/* The abstract type descriptor for rojcad/shape.
 * Uses JANET_ATEND_* macros to fill remaining fields with NULL defaults
 * so new fields can be added to JanetAbstractType in future versions. */
static JanetAbstractType rojcad_shape_type = {
    .name = "rojcad/shape",
    .gc = NULL,           /* will be set in cad_register_functions */
    .tostring = NULL,     /* will be set in cad_register_functions */
    JANET_ATEND_GCPERTHREAD
};

/* GC finalizer: called by Janet GC when the abstract value is collected.
 * Returns 0 on success. */
static int shape_gc_finish(void *data, size_t len) {
    if (data) {
        rust_shape_drop(data, len);
    }
    return 0;
}

/* tostring: produce "#<Shape(SOLID)>" etc. */
static void shape_to_string(void *data, JanetBuffer *buffer) {
    const char *type_str = rust_shape_type_string(data);
    janet_buffer_push_cstring(buffer, "#<Shape(");
    janet_buffer_push_cstring(buffer, type_str);
    janet_buffer_push_cstring(buffer, ")>");
}

/* ── Sketch abstract type ────────────────────────────────────────────────── */

static JanetAbstractType rojcad_sketch_type = {
    .name = "rojcad/sketch",
    .gc = NULL,
    .tostring = NULL,
    JANET_ATEND_GCPERTHREAD
};

static int sketch_gc_finish(void *data, size_t len) {
    if (data) {
        rust_sketch_drop(data, len);
    }
    return 0;
}

static void sketch_to_string(void *data, JanetBuffer *buffer) {
    (void)data;
    janet_buffer_push_cstring(buffer, "#<Sketch>");
}

static void *alloc_sketch(void) {
    void *data = janet_abstract(&rojcad_sketch_type, rust_sketch_data_size());
    if (!data) {
        janet_panic("failed to allocate sketch");
    }
    return data;
}

/* ── Helper functions ───────────────────────────────────────────────────── */

static void *unwrap_shape_or_panic(Janet val, int index) {
    JanetAbstract abs = janet_checkabstract(val, &rojcad_shape_type);
    if (!abs) {
        janet_panicf("expected rojcad/shape, got %T at argument %d", val, index);
    }
    return abs;
}

/* Find a keyword in argv, return its index or -1 */
static int find_keyword(const Janet *argv, int32_t argc, const char *kw) {
    for (int32_t i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) {
            const uint8_t *s = janet_unwrap_keyword(argv[i]);
            if (strcmp((const char *)s, kw) == 0) {
                return i;
            }
        }
    }
    return -1;
}

/* Parse a keyword's double value. Returns 1 if found, 0 if not. */
static int kw_double(const Janet *argv, int32_t argc, const char *kw, double *val) {
    int idx = find_keyword(argv, argc, kw);
    if (idx < 0) return 0;
    if (idx + 1 >= argc) {
        janet_panicf("keyword :%s requires a value", kw);
    }
    if (!janet_checktype(argv[idx + 1], JANET_NUMBER)) {
        janet_panicf("keyword :%s expects a number", kw);
    }
    *val = janet_unwrap_number(argv[idx + 1]);
    return 1;
}

/* Parse a keyword's array or tuple [x y z] / '(x y z) value.
 * Returns 1 if found, 0 if not. */
static int kw_array_3(const Janet *argv, int32_t argc, const char *kw,
                       double *x, double *y, double *z) {
    int idx = find_keyword(argv, argc, kw);
    if (idx < 0) return 0;
    if (idx + 1 >= argc) {
        janet_panicf("keyword :%s requires an array or tuple argument", kw);
    }
    Janet val = argv[idx + 1];
    if (janet_checktype(val, JANET_ARRAY)) {
        JanetArray *arr = janet_unwrap_array(val);
        if (arr->count != 3) {
            janet_panicf("keyword :%s expects 3 numbers, got %d", kw, arr->count);
        }
        *x = janet_unwrap_number(arr->data[0]);
        *y = janet_unwrap_number(arr->data[1]);
        *z = janet_unwrap_number(arr->data[2]);
    } else if (janet_checktype(val, JANET_TUPLE)) {
        const Janet *parts = janet_unwrap_tuple(val);
        int32_t tlen = janet_tuple_length(parts);
        if (tlen != 3) {
            janet_panicf("keyword :%s expects 3 numbers, got %d", kw, tlen);
        }
        *x = janet_unwrap_number(parts[0]);
        *y = janet_unwrap_number(parts[1]);
        *z = janet_unwrap_number(parts[2]);
    } else {
        janet_panicf("keyword :%s expects an array or tuple of 3 numbers", kw);
    }
    return 1;
}

/* Check if :eager keyword is present in argv. Returns 1 if found, 0 if not. */
static int has_eager(const Janet *argv, int32_t argc) {
    return find_keyword(argv, argc, "eager") >= 0 ? 1 : 0;
}

/* If :hide keyword is present, mark shape as not visible.
 * Safe to call on unregistered shapes — just sets visible=false. */
static void maybe_hide(void *data, const Janet *argv, int32_t argc) {
    if (find_keyword(argv, argc, "hide") >= 0) {
        rust_shape_hide(data);
    }
}

/* Allocate a new rojcad/shape abstract via Janet GC */
static void *alloc_shape(void) {
    void *data = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!data) {
        janet_panic("failed to allocate shape");
    }
    return data;
}

/* Check return value from a rust_init_* call. If non-zero, retrieve the
 * last error from the thread-local buffer and call janet_panic. */
#define CAD_CHECK(call) do { \
    if ((call)) { \
        const char *_msg = rust_take_last_error(); \
        janet_panic(_msg); \
    } \
} while (0)

/* ── JANET_FN implementations ───────────────────────────────────────────── */

/* With JANET_NO_SOURCEMAPS defined (see build.rs), JANET_FN expands to
 * JANET_FN_D(CNAME, USAGE, DOCSTRING), which creates a static docstring
 * combining USAGE and DOCSTRING separated by "\n\n". */

JANET_FN(cad_box,
         "(box width depth height &keys :w :d :h :c :pl :ph :eager :hide)",
         "Create a box or cube.\n\n"
         "Positional: (box w d h) or (box size) for a cube.\n"
         "Keywords: :w :d :h (dimensions), :c (center [x y z]),\n"
         "         :pl :ph (opposite corners [x y z]).\n"
         "         :eager (tessellate immediately).\n"
         "         :hide (skip automatic show on def).\n\n"
         "Examples:\n"
         "  (box 10 20 30)           — box at origin\n"
         "  (box 10 20 30 :c [5 5 5]) — centered box\n"
         "  (box 5)                  — 5x5x5 cube\n"
         "  (box :pl [0 0 0] :ph [10 20 30]) — from corners\n"
         "  (box :w 10 :d 20 :h 30) — keyword style\n"
         "  (box 10 :eager)          — eager tessellation\n"
         "  (box 10 :hide)           — create without showing\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, pl[3], ph[3];
    int has_c, has_pl, has_ph;
    int eager = has_eager(argv, argc);

    /* Count positional args (stop at first keyword) */
    int pos_count = 0;
    for (int i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) break;
        pos_count++;
    }

    has_pl = kw_array_3(argv, argc, "pl", &pl[0], &pl[1], &pl[2]);
    has_ph = kw_array_3(argv, argc, "ph", &ph[0], &ph[1], &ph[2]);

    if (has_pl || has_ph) {
        if (!has_pl || !has_ph) {
            janet_panic("box: :pl and :ph must both be provided");
        }
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_box_from_corners(shape, pl[0], pl[1], pl[2], ph[0], ph[1], ph[2], eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }

    double w = 0, d = 0, h = 0;
    int has_w, has_d, has_h;
    has_w = kw_double(argv, argc, "w", &w);
    has_d = kw_double(argv, argc, "d", &d);
    has_h = kw_double(argv, argc, "h", &h);
    has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);

    if (has_w && has_d && has_h) {
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_box(shape, w, d, h,
                      has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }

    if (has_w || has_d || has_h) {
        janet_panic("box: specify :w, :d, :h together, or use positional args");
    }

    if (pos_count == 1) {
        double size = janet_unwrap_number(argv[0]);
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_cube(shape, size,
                       has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }

    if (pos_count >= 3) {
        w = janet_unwrap_number(argv[0]);
        d = janet_unwrap_number(argv[1]);
        h = janet_unwrap_number(argv[2]);
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_box(shape, w, d, h,
                      has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }

    janet_panicf("box: expected 1 or 3 positional arguments, got %d", argc);
}

JANET_FN(cad_sphere,
         "(sphere radius &keys :r :c :a :ar :eager)",
         "Create a sphere.\n\n"
         "Positional: (sphere radius)\n"
         "Keywords: :r (radius), :c (center [x y z]),\n"
         "         :a (angle in degrees), :ar (angle in radians),\n"
         "         :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (sphere 10)               — full sphere at origin\n"
         "  (sphere 10 :c [1 2 3])    — repositioned\n"
         "  (sphere 10 :a 180)        — hemisphere\n"
         "  (sphere :r 10)            — keyword style\n"
         "  (sphere 10 :eager)        — eager tessellation\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double radius, cx, cy, cz, angle;
    int has_c, has_a;
    int eager = has_eager(argv, argc);

    has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    has_a = kw_double(argv, argc, "a", &angle);
    if (has_a) {
        angle *= (M_PI / 180.0);
    } else {
        has_a = kw_double(argv, argc, "ar", &angle);
    }

    /* Try keyword :r first, then positional */
    if (!kw_double(argv, argc, "r", &radius)) {
        if (argc < 1) janet_panic("sphere: radius is required");
        radius = janet_unwrap_number(argv[0]);
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_sphere(shape, radius,
                     has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL,
                     has_a ? &angle : NULL, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_cylinder,
         "(cylinder radius height &keys :r :h :c :dir :fp :tp :eager)",
         "Create a cylinder.\n\n"
         "Positional: (cylinder radius height) — along Z axis, base at Z=0\n"
         "Keywords: :r (radius), :h (height), :c (center [x y z]),\n"
         "         :dir (direction [dx dy dz]),\n"
         "         :fp (from-point [x y z]), :tp (to-point [x y z]).\n"
         "         :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (cylinder 5 10)                       — simple\n"
         "  (cylinder 5 10 :c [0 0 5])            — centered\n"
         "  (cylinder :fp [0 0 0] :tp [0 0 10] :r 5) — point-to-point\n"
         "  (cylinder :r 5 :h 10)                 — keyword style\n"
         "  (cylinder 5 10 :eager)                — eager tessellation\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, dir[3], fp[3], tp[3];
    int has_c, has_dir, has_fp, has_tp;
    int eager = has_eager(argv, argc);

    has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    has_dir = kw_array_3(argv, argc, "dir", &dir[0], &dir[1], &dir[2]);
    has_fp = kw_array_3(argv, argc, "fp", &fp[0], &fp[1], &fp[2]);
    has_tp = kw_array_3(argv, argc, "tp", &tp[0], &tp[1], &tp[2]);

    /* Check for from-point / to-point mode */
    if (has_fp || has_tp) {
        if (!has_fp || !has_tp) {
            janet_panic("cylinder: :fp and :tp must both be provided");
        }
        double r;
        if (!kw_double(argv, argc, "r", &r)) {
            janet_panic("cylinder: :r (radius) is required with :fp/:tp");
        }
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_cylinder_from_points(shape, fp[0], fp[1], fp[2], tp[0], tp[1], tp[2], r, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }

    double radius, height;

    /* Get radius and height: try keywords first, then positional */
    if (!kw_double(argv, argc, "r", &radius)) {
        if (argc < 1) janet_panic("cylinder: radius is required");
        radius = janet_unwrap_number(argv[0]);
    }
    if (!kw_double(argv, argc, "h", &height)) {
        if (argc < 2) janet_panic("cylinder: height is required");
        height = janet_unwrap_number(argv[1]);
    }

    if (has_dir) {
        double ox = has_c ? cx : 0.0, oy = has_c ? cy : 0.0, oz = has_c ? cz : 0.0;
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_cylinder_point_dir(shape, ox, oy, oz, radius, dir[0], dir[1], dir[2], height, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }

    {
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_cylinder(shape, radius, height,
                           has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_cone,
         "(cone bottom-radius height &keys :br :tr :h :c :a :ar :eager)",
         "Create a cone or truncated cone.\n\n"
         "Positional: (cone br h) for full cone, (cone br tr h) for truncated.\n"
         "Keywords: :br (bottom radius), :tr (top radius), :h (height),\n"
         "         :c (center [x y z]),\n"
         "         :a (angle in degrees), :ar (angle in radians, partial cone),\n"
         "         :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (cone 5 10)                — full cone, br=5, h=10\n"
         "  (cone 5 3 10)              — truncated cone\n"
         "  (cone 5 10 :a 180)         — half cone\n"
         "  (cone :br 5 :h 10)         — keyword style\n"
         "  (cone 5 10 :eager)         — eager tessellation\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, angle;
    int eager = has_eager(argv, argc);
    int has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    int has_a = kw_double(argv, argc, "a", &angle);
    if (has_a) {
        angle *= (M_PI / 180.0);
    } else {
        has_a = kw_double(argv, argc, "ar", &angle);
    }

    double br = 0, tr = 0, h = 0;
    int has_br, has_tr, has_h;

    has_br = kw_double(argv, argc, "br", &br);
    has_tr = kw_double(argv, argc, "tr", &tr);
    has_h = kw_double(argv, argc, "h", &h);

    if (has_br && has_h) {
        if (!has_tr) tr = 0.0;
        goto create;
    }
    if (has_br || has_h) {
        janet_panic("cone: provide :br, :h, and optionally :tr, or use positional args");
    }

    /* Count positional args (stop at first keyword) */
    int pos_count = 0;
    for (int i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) break;
        pos_count++;
    }

    /* Positional mode */
    if (pos_count == 2) {
        br = janet_unwrap_number(argv[0]);
        tr = 0.0;
        h = janet_unwrap_number(argv[1]);
    } else if (pos_count == 3) {
        br = janet_unwrap_number(argv[0]);
        tr = janet_unwrap_number(argv[1]);
        h = janet_unwrap_number(argv[2]);
    } else {
        janet_panicf("cone: expected 2 or 3 positional arguments, got %d", pos_count);
    }

create:
    {
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_cone(shape, br, tr, h,
                       has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL,
                       has_a ? &angle : NULL, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_torus,
         "(torus ring-radius tube-radius &keys :rr :tr :c :a :ar :as :asr :ae :aer :dir :eager)",
         "Create a torus.\n\n"
         "Positional: (torus rr tr)\n"
         "Keywords: :rr (ring radius), :tr (tube radius),\n"
         "         :c (center [x y z]),\n"
         "         :a (angle in degrees), :ar (angle in radians, partial),\n"
         "         :as (start angle degrees), :asr (start angle radians),\n"
         "         :ae (end angle degrees), :aer (end angle radians),\n"
         "         :dir (axis direction [dx dy dz]),\n"
         "         :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (torus 20 10)                    — full torus\n"
         "  (torus 20 10 :c [0 0 5])         — repositioned\n"
         "  (torus 20 10 :a 180)             — half torus\n"
         "  (torus :rr 20 :tr 10 :as 0 :ae 180) — angled range\n"
         "  (torus :rr 20 :tr 10 :dir [0 1 0]) — oriented\n"
         "  (torus 20 10 :eager)             — eager tessellation\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, dir[3], angle, a_start, a_end;
    int eager = has_eager(argv, argc);
    int has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    int has_dir = kw_array_3(argv, argc, "dir", &dir[0], &dir[1], &dir[2]);
    int has_a = kw_double(argv, argc, "a", &angle);
    if (has_a) {
        angle *= (M_PI / 180.0);
    } else {
        has_a = kw_double(argv, argc, "ar", &angle);
    }
    int has_as = kw_double(argv, argc, "as", &a_start);
    if (has_as) {
        a_start *= (M_PI / 180.0);
    } else {
        has_as = kw_double(argv, argc, "asr", &a_start);
    }
    int has_ae = kw_double(argv, argc, "ae", &a_end);
    if (has_ae) {
        a_end *= (M_PI / 180.0);
    } else {
        has_ae = kw_double(argv, argc, "aer", &a_end);
    }

    double rr = 0, tr = 0;
    int has_rr, has_tr;

    has_rr = kw_double(argv, argc, "rr", &rr);
    has_tr = kw_double(argv, argc, "tr", &tr);

    if (has_rr && has_tr) {
        goto create;
    }
    if (has_rr || has_tr) {
        janet_panic("torus: :rr and :tr must be provided together");
    }

    /* Positional mode */
    if (argc < 2) janet_panic("torus: ring-radius and tube-radius are required");
    rr = janet_unwrap_number(argv[0]);
    tr = janet_unwrap_number(argv[1]);

create:
    {
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_torus(shape, rr, tr,
                        has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL,
                        has_dir ? &dir[0] : NULL,
                        has_dir ? &dir[1] : NULL,
                        has_dir ? &dir[2] : NULL,
                        has_a ? &angle : NULL,
                        has_as ? &a_start : NULL,
                        has_ae ? &a_end : NULL, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_cut,
         "(cut shape-a shape-b &keys :eager)",
         "Subtract shape-b from shape-a. Returns a new rojcad/shape "
         "representing the resulting solid.\n\n"
         "Signals an error if the shapes do not intersect or produce "
         "an empty result.\n"
         "Keywords: :eager (tessellate immediately).")
{
    janet_arity(argc, 2, 3);
    int eager = has_eager(argv, argc);
    void *a = unwrap_shape_or_panic(argv[0], 0);
    void *b = unwrap_shape_or_panic(argv[1], 1);

    void *result = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!result) {
        janet_panic("failed to allocate shape");
    }
    CAD_CHECK(rust_init_cut(result, a, b, eager));
    maybe_hide(result, argv, argc);
    return janet_wrap_abstract(result);
}

JANET_FN(cad_common,
         "(common shape-a shape-b &keys :eager)",
         "Intersect shape-a with shape-b. Returns a new rojcad/shape "
         "representing the shared volume.\n\n"
         "Signals an error if the shapes do not intersect.\n"
         "Keywords: :eager (tessellate immediately).")
{
    janet_arity(argc, 2, 3);
    int eager = has_eager(argv, argc);
    void *a = unwrap_shape_or_panic(argv[0], 0);
    void *b = unwrap_shape_or_panic(argv[1], 1);

    void *result = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!result) {
        janet_panic("failed to allocate shape");
    }
    CAD_CHECK(rust_init_common(result, a, b, eager));
    maybe_hide(result, argv, argc);
    return janet_wrap_abstract(result);
}

JANET_FN(cad_fuse,
         "(fuse shape-a shape-b &keys :eager)",
         "Combine shape-a and shape-b into a single solid. Returns a new rojcad/shape "
         "representing the union of both shapes.\n\n"
         "Signals an error if the operation produces an empty result.\n"
         "Keywords: :eager (tessellate immediately).")
{
    janet_arity(argc, 2, 3);
    int eager = has_eager(argv, argc);
    void *a = unwrap_shape_or_panic(argv[0], 0);
    void *b = unwrap_shape_or_panic(argv[1], 1);

    void *result = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!result) {
        janet_panic("failed to allocate shape");
    }
    CAD_CHECK(rust_init_fuse(result, a, b, eager));
    maybe_hide(result, argv, argc);
    return janet_wrap_abstract(result);
}

JANET_FN(cad_translate,
         "(translate shape dx dy dz &keys :t :eager)",
         "Create a translated copy of shape.\n\n"
         "Positional: (translate shape dx dy dz)\n"
         "Keywords: :t [dx dy dz], :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (translate box 5 0 0)               — move 5 units in X\n"
         "  (translate box :t [1 2 3])          — keyword style\n"
         "  (translate box 5 0 0 :eager)        — eager tessellation\n\n"
         "Returns a new rojcad/shape abstract value. The original shape is unchanged.")
{
    double dx, dy, dz;
    void *data;
    int eager = has_eager(argv, argc);

    if (kw_array_3(argv, argc, "t", &dx, &dy, &dz)) {
        /* Keyword style: find the shape as the first non-keyword arg */
        if (argc < 1) janet_panic("translate: shape is required");
        data = unwrap_shape_or_panic(argv[0], 0);
    } else {
        janet_arity(argc, 4, 4);
        data = unwrap_shape_or_panic(argv[0], 0);
        if (!janet_checktype(argv[1], JANET_NUMBER) ||
            !janet_checktype(argv[2], JANET_NUMBER) ||
            !janet_checktype(argv[3], JANET_NUMBER)) {
            janet_panic("translate: dx, dy, dz must be numbers");
        }
        dx = janet_unwrap_number(argv[1]);
        dy = janet_unwrap_number(argv[2]);
        dz = janet_unwrap_number(argv[3]);
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_translate(shape, data, dx, dy, dz, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_rotate,
         "(rotate shape &keys :a :ar :x :y :z :r :eager)",
         "Create a rotated copy of shape.\n\n"
         "Angle is specified via :a (degrees) or :ar (radians).\n"
         "Axis is specified via :x, :y, :z (cardinal), or :r [dx dy dz] (custom).\n"
         ":eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (rotate box :a 45 :z)           — 45 degrees about Z\n"
         "  (rotate box :ar 1.5708 :x)      — pi/2 radians about X\n"
         "  (rotate box :a 90 :r [1 1 0])   — 90 degrees about custom axis\n"
         "  (rotate box :a 90 :z :eager)    — eager tessellation\n\n"
         "Returns a new rojcad/shape abstract value. The original shape is unchanged.")
{
    if (argc < 2) janet_panic("rotate: shape and angle are required");
    void *data = unwrap_shape_or_panic(argv[0], 0);
    int eager = has_eager(argv, argc);

    double angle;

    if (kw_double(argv, argc, "ar", &angle)) {
        /* radians — pass through as-is */
    } else if (kw_double(argv, argc, "a", &angle)) {
        angle *= (M_PI / 180.0);
    } else {
        janet_panic("rotate: specify angle via :a (degrees) or :ar (radians)");
    }

    double ax, ay, az;
    if (find_keyword(argv, argc, "x") >= 0) {
        ax = 1.0; ay = 0.0; az = 0.0;
    } else if (find_keyword(argv, argc, "y") >= 0) {
        ax = 0.0; ay = 1.0; az = 0.0;
    } else if (find_keyword(argv, argc, "z") >= 0) {
        ax = 0.0; ay = 0.0; az = 1.0;
    } else if (kw_array_3(argv, argc, "r", &ax, &ay, &az)) {
        /* custom axis */
    } else {
        janet_panic("rotate: specify axis via :x, :y, :z, or :r [dx dy dz]");
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_rotate(shape, data, ax, ay, az, angle, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_scale,
         "(scale shape factor &keys :o :eager)",
         "Create a uniformly scaled copy of shape.\n\n"
         "Positional: (scale shape factor)\n"
         "Keywords: :o [x y z] (center point, defaults to origin),\n"
         "         :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (scale box 2.0)                — 2x about origin\n"
         "  (scale box 2.0 :o [5 5 5])     — 2x about custom point\n"
         "  (scale box 2.0 :eager)         — eager tessellation\n\n"
         "Returns a new rojcad/shape abstract value. The original shape is unchanged.")
{
    if (argc < 2) janet_panic("scale: shape and factor are required");
    void *data = unwrap_shape_or_panic(argv[0], 0);
    if (!janet_checktype(argv[1], JANET_NUMBER)) {
        janet_panic("scale: factor must be a number");
    }
    double factor = janet_unwrap_number(argv[1]);
    int eager = has_eager(argv, argc);

    double cx, cy, cz;
    int has_o = kw_array_3(argv, argc, "o", &cx, &cy, &cz);

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_scale(shape, data, factor,
                    has_o ? &cx : NULL,
                    has_o ? &cy : NULL,
                    has_o ? &cz : NULL, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_mirror,
         "(mirror shape ox oy oz dx dy dz &keys :eager)",
         "Create a mirrored copy of shape about an axis.\n\n"
         "Positional: (mirror shape ox oy oz dx dy dz)\n"
         "Where (ox, oy, oz) is a point on the axis and (dx, dy, dz) is the axis direction.\n"
         "Keywords: :eager (tessellate immediately).\n\n"
         "Examples:\n"
         "  (mirror box 0 0 0 1 0 0)       — mirror across X axis through origin\n"
         "  (mirror box 5 0 0 0 1 0)       — mirror across Y axis through (5,0,0)\n"
         "  (mirror box 0 0 0 1 0 0 :eager) — eager tessellation\n\n"
         "Returns a new rojcad/shape abstract value. The original shape is unchanged.")
{
    if (argc < 7) janet_panic("mirror: shape and 6 coordinates are required");
    void *data = unwrap_shape_or_panic(argv[0], 0);
    int eager = has_eager(argv, argc);
    if (!janet_checktype(argv[1], JANET_NUMBER) ||
        !janet_checktype(argv[2], JANET_NUMBER) ||
        !janet_checktype(argv[3], JANET_NUMBER) ||
        !janet_checktype(argv[4], JANET_NUMBER) ||
        !janet_checktype(argv[5], JANET_NUMBER) ||
        !janet_checktype(argv[6], JANET_NUMBER)) {
        janet_panic("mirror: all coordinates must be numbers");
    }
    double ox = janet_unwrap_number(argv[1]);
    double oy = janet_unwrap_number(argv[2]);
    double oz = janet_unwrap_number(argv[3]);
    double dx = janet_unwrap_number(argv[4]);
    double dy = janet_unwrap_number(argv[5]);
    double dz = janet_unwrap_number(argv[6]);

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_mirror(shape, data, ox, oy, oz, dx, dy, dz, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_purge,
         "(purge shape)",
         "Remove a shape from the viewer registry and mark it as purged.\n"
         "The shape will no longer be rendered. To also unbind the Janet variable,\n"
         "use (purge shape) followed by (def name nil).\n\n"
         "Examples:\n"
         "  (purge b)          — remove b from viewer\n"
         "  (purge b) (def b nil) (gc)  — full cleanup\n\n"
         "Returns nil.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    rust_shape_remove_from_registry(data);
    return janet_wrap_nil();
}

JANET_FN(cad_shape_type,
         "(shape-type shape)",
         "Return the OCCT topological type of a shape as a keyword. "
         "Returns :solid, :face, :edge, :wire, :shell, :vertex, "
         ":compound, :compound-solid, or :shape.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    const char *type_str = rust_shape_type(data);
    return janet_ckeywordv(type_str);
}

JANET_FN(cad_show,
         "(show shape)",
         "Register a shape in the viewer and make it visible.\n\n"
         "If the shape has not been tessellated, tessellation happens automatically.\n"
         "Calling show on an already-visible shape is a no-op.\n\n"
         "Examples:\n"
         "  (def b (box 10))\n"
         "  (show b)         — tessellates if needed, registers, makes visible\n"
         "  (show b)         — second call is a no-op (already visible)\n\n"
         "Returns nil.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    rust_shape_show(data);
    return janet_wrap_nil();
}

JANET_FN(cad_hide,
         "(hide shape)",
         "Set a shape's visible flag to false. The shape stays registered"
         " in the viewer but is no longer rendered.\n\n"
         "Examples:\n"
         "  (hide b)         — shape disappears from viewer\n"
         "  (show b)         — reappears without re-tessellating\n\n"
         "Returns nil.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    rust_shape_hide(data);
    return janet_wrap_nil();
}

JANET_FN(cad_registry_remove,
         "(registry-remove shape)",
         "Immediately remove a shape from the viewer registry and mark it as purged.\n"
         "The shape will no longer be rendered. The underlying OCCT shape memory\n"
         "is freed when Janet's GC collects the shape value.\n\n"
         "This is used internally by the `purge` macro.\n\n"
         "Returns nil.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    rust_shape_remove_from_registry(data);
    return janet_wrap_nil();
}

JANET_FN(cad_visible_q,
         "(visible? shape)",
         "Return true if the shape's visible flag is set, false otherwise.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    int visible = rust_shape_get_visible(data);
    return visible ? janet_wrap_true() : janet_wrap_false();
}

/* Global callback for selection events */
static Janet on_select_callback = {0};

JANET_FN(cad_on_select,
         "(on-select callback)",
         "Register a Janet function to be called when a shape is selected"
         " in the viewer. The function receives the same value as"
         " (poll-selection): a shape ID (integer), :deselected, or"
         " [:deselected id]. Pass nil to unregister the callback.\n\n"
         "Example:\n"
         "  (on-select (fn [event] (print \"selection: \" event)))")
{
    janet_arity(argc, 1, 1);
    if (janet_checktype(argv[0], JANET_NIL)) {
        on_select_callback = janet_wrap_nil();
    } else if (janet_checktype(argv[0], JANET_FUNCTION)) {
        on_select_callback = argv[0];
    } else {
        janet_panic("on-select expects a function or nil");
    }
    return janet_wrap_nil();
}

JANET_FN(cad_poll_selection,
         "(poll-selection)",
         "Check for a pending selection event from the viewer.\n\n"
         "Returns nil if no event, the shape ID (integer) if a shape was"
         " selected, a tuple [:deselected id] if a shape was toggled off,\n"
         " or :deselected if the entire selection was cleared.\n\n"
         "If a callback was registered via (on-select), it will be"
         " invoked automatically with the result.")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    uint8_t action;
    uint64_t id = rust_poll_selection(&action);
    if (action == 0) {
        return janet_wrap_nil();
    }

    Janet event;
    if (action == 3) {
        /* Cleared */
        event = janet_ckeywordv("deselected");
    } else if (action == 2) {
        /* Toggled off — return [:deselected id] */
        Janet parts[2];
        parts[0] = janet_ckeywordv("deselected");
        parts[1] = janet_wrap_number((double)id);
        event = janet_wrap_tuple(janet_tuple_n(parts, 2));
    } else {
        /* Toggled on (action == 1) — return shape id (backward compat) */
        event = janet_wrap_number((double)id);
    }

    /* Invoke the stored callback if registered */
    if (janet_checktype(on_select_callback, JANET_FUNCTION)) {
        JanetFunction *fn = janet_unwrap_function(on_select_callback);
        Janet args[] = { event };
        janet_call(fn, 1, args);
    }

    return event;
}

JANET_FN(cad_quit_requested,
         "(quit-requested)",
         "Check if the application should quit.\n\n"
         "Returns true if Ctrl+Q was pressed or the window was closed.\n"
         "Used by boot.janet to exit the event loop.\n\n"
         "This is a one-shot check -- returns true only once per quit request.")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    return rust_quit_requested() ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_toggle_inactive,
         "(edge-toggle-inactive)",
         "Toggle visibility of edges on non-selected shapes. "
         "Returns true if inactive edges are now visible, false if hidden.\n\n"
         "Example: (edge-toggle-inactive)")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_edge_toggle_inactive();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_toggle_active,
         "(edge-toggle-active)",
         "Toggle visibility of edges on the selected shape. "
         "Returns true if active edges are now visible, false if hidden.\n\n"
         "Example: (edge-toggle-active)")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_edge_toggle_active();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_inactive_showing,
         "(edge-inactive-show?)",
         "Return true if edges on non-selected shapes are currently visible, "
         "false if hidden.")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_edge_inactive_showing();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_active_showing,
         "(edge-active-show?)",
         "Return true if edges on the selected shape are currently visible, "
         "false if hidden.")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_edge_active_showing();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_hidden_toggle,
         "(edge-hidden-toggle)",
         "Toggle visibility of hidden (occluded) edges. "
         "Returns true if hidden edges are now visible, false if hidden.\n\n"
         "Example: (edge-hidden-toggle)")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_back_edges_toggle();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_hidden_showing,
         "(edge-hidden-show?)",
         "Return true if hidden (occluded) edges are currently visible, "
         "false if hidden.")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_back_edges_showing();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_hidden,
         "(edge-hidden &opt value)",
         "Get or set visibility of hidden (occluded) edges.\n\n"
         "Called with no arguments, returns true if hidden edges are shown, "
         "false if hidden.\n"
         "Called with one boolean argument, sets the visibility.\n\n"
         "Example: (edge-hidden)        — query\n"
         "         (edge-hidden true)    — show hidden edges\n"
         "         (edge-hidden false)   — hide hidden edges")
{
    janet_arity(argc, 0, 1);
    if (argc == 0) {
        int result = rust_back_edges_showing();
        return result ? janet_wrap_true() : janet_wrap_false();
    }
    int val = janet_truthy(argv[0]);
    rust_back_edges_set(val);
    return janet_wrap_true();
}

JANET_FN(cad_projection_toggle,
         "(projection-toggle)",
         "Toggle camera projection between perspective and orthographic. "
         "Returns true if now in perspective mode, false if orthographic.\n\n"
         "Example: (projection-toggle)")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_projection_perspective_toggle();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_projection_perspective,
         "(projection-perspective &opt value)",
         "Get or set the camera projection mode.\n\n"
         "Called with no arguments, returns true if perspective mode is active, "
         "false if orthographic.\n"
         "Called with one boolean argument, sets the mode.\n\n"
         "Example: (projection-perspective)        — query\n"
         "         (projection-perspective true)    — perspective\n"
         "         (projection-perspective false)   — orthographic")
{
    janet_arity(argc, 0, 1);
    if (argc == 0) {
        int result = rust_projection_perspective_showing();
        return result ? janet_wrap_true() : janet_wrap_false();
    }
    int val = janet_truthy(argv[0]);
    rust_projection_perspective_set(val);
    return janet_wrap_true();
}

JANET_FN(cad_stats_overlay,
         "(stats-overlay &opt value)",
         "Get or set the stats overlay visibility.\n\n"
         "Called with no arguments, returns true if the overlay is visible, "
         "false if hidden.\n"
         "Called with one boolean argument, sets the visibility.\n\n"
         "Example: (stats-overlay)        — query\n"
         "         (stats-overlay true)    — show overlay\n"
         "         (stats-overlay false)   — hide overlay\n\n"
         "The overlay can also be toggled with Ctrl+Shift+Alt+S in the viewer window.")
{
    janet_arity(argc, 0, 1);
    if (argc == 0) {
        int result = rust_stats_overlay_showing();
        return result ? janet_wrap_true() : janet_wrap_false();
    }
    int val = janet_truthy(argv[0]);
    rust_stats_overlay_set(val);
    return janet_wrap_true();
}

JANET_FN(cad_help_toggle,
         "(window-help-toggle)",
         "Toggle the help window visibility. "
         "Returns true if now visible, false if hidden.\n\n"
         "Example: (window-help-toggle)")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_help_overlay_toggle();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_help_showing,
         "(window-help-show?)",
         "Return true if the help window is currently visible, "
         "false if hidden.\n\n"
         "Example: (window-help-show?)")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_help_overlay_showing();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_help_set,
         "(window-help-show &opt value)",
         "Get or set the help window visibility.\n\n"
         "Called with no arguments, returns true if visible, false if hidden.\n"
         "Called with one boolean argument, sets the visibility.\n\n"
         "Example: (window-help-show)        — query\n"
         "         (window-help-show true)    — show\n"
         "         (window-help-show false)   — hide")
{
    janet_arity(argc, 0, 1);
    if (argc == 0) {
        int result = rust_help_overlay_showing();
        return result ? janet_wrap_true() : janet_wrap_false();
    }
    int val = janet_truthy(argv[0]);
    rust_help_overlay_set(val);
    return janet_wrap_true();
}

JANET_FN(cad_window_size,
         "(window-size width height)",
         "Resize the viewer window to the given logical pixel dimensions.\n\n"
         "Both width and height must be positive integers.\n\n"
         "Example:\n"
         "  (window-size 800 600)    ; resize to 800x600")
{
    janet_arity(argc, 2, 2);
    int32_t w = janet_getinteger(argv, 0);
    int32_t h = janet_getinteger(argv, 1);
    rust_window_set_size((uint32_t)(w > 0 ? w : 0), (uint32_t)(h > 0 ? h : 0));
    return janet_wrap_nil();
}

JANET_FN(cad_window_size_query,
         "(window-size?)",
         "Return the current viewer window dimensions as a tuple "
         "[width height] in logical pixels.\n\n"
         "Example:\n"
         "  (window-size?)    ; e.g., returns [1024 768]")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    uint32_t w = 0, h = 0;
    rust_window_size_query(&w, &h);
    Janet result[2];
    result[0] = janet_wrap_integer((int32_t)w);
    result[1] = janet_wrap_integer((int32_t)h);
    return janet_wrap_tuple(result);
}

JANET_FN(cad_window_fullscreen,
         "(window-fullscreen value)",
         "Enter or exit fullscreen mode.\n\n"
         "Pass true to enter borderless fullscreen, false to return to windowed mode.\n\n"
         "Examples:\n"
         "  (window-fullscreen true)   ; enter fullscreen\n"
         "  (window-fullscreen false)  ; exit fullscreen")
{
    janet_arity(argc, 1, 1);
    int fs = janet_truthy(argv[0]);
    rust_window_set_fullscreen(fs);
    return janet_wrap_nil();
}

JANET_FN(cad_window_fullscreen_query,
         "(window-fullscreen?)",
         "Return true if the viewer is in fullscreen mode, false otherwise.\n\n"
         "Example:\n"
         "  (window-fullscreen?)   ; returns true or false")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_window_fullscreen_query();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_window_maximized,
         "(window-maximized value)",
         "Enter or exit maximized state.\n\n"
         "Pass true to maximize, false to restore to windowed.\n\n"
         "Examples:\n"
         "  (window-maximized true)   ; maximize\n"
         "  (window-maximized false)  ; restore")
{
    janet_arity(argc, 1, 1);
    int mx = janet_truthy(argv[0]);
    rust_window_set_maximized(mx);
    return janet_wrap_nil();
}

JANET_FN(cad_window_maximized_query,
         "(window-maximized?)",
         "Return true if the viewer window is maximized, false otherwise.\n\n"
         "Example:\n"
         "  (window-maximized?)   ; returns true or false")
{
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_window_maximized_query();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_view_fit,
         "(view-fit shape & shapes ; reset)",
         "Fit camera to the bounding box of one or more shapes.\n\n"
         "Animates the 3D camera over 0.5s to frame the union bounding\n"
         "box of the given shapes. The current orbit angle is preserved.\n\n"
         "Use :reset to return to the default isometric angle\n"
         "(yaw=0, pitch=0.4).\n\n"
         "Examples:\n"
         "  (view-fit my-shape)\n"
         "  (view-fit box1 cylinder2)\n"
         "  (view-fit :reset part1 part2)")
{
    int shape_count = 0;
    int reset = 0;

    for (int32_t i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) {
            const uint8_t *kw = janet_unwrap_keyword(argv[i]);
            if (strcmp((const char *)kw, "reset") == 0) {
                reset = 1;
            }
        } else {
            shape_count++;
        }
    }

    if (shape_count < 1) {
        janet_panic("expected at least one shape");
    }

    void **shape_ptrs = janet_malloc((size_t)shape_count * sizeof(void *));
    if (!shape_ptrs) {
        janet_panic("out of memory");
    }

    int32_t idx = 0;
    for (int32_t i = 0; i < argc; i++) {
        if (!janet_checktype(argv[i], JANET_KEYWORD)) {
            shape_ptrs[idx++] = janet_getabstract(argv, i, &rojcad_shape_type);
        }
    }

    rust_view_fit_shapes(shape_ptrs, shape_count, reset);
    janet_free(shape_ptrs);
    return janet_wrap_nil();
}

JANET_FN(cad_view_fit_all,
         "(view-fit-all ; hidden ; reset)",
         "Fit camera to the bounding box of shapes.\n\n"
         "By default only visible shapes are framed. "
         "Use :hidden to include hidden shapes as well.\n"
         "Animates the 3D camera over 0.5s to frame the union bounding box.\n"
         "The current orbit angle is preserved.\n"
         "If no shapes are found, resets the camera to default position.\n\n"
         "Keywords:\n"
         "  :hidden  — include hidden shapes in the bounding box\n"
         "  :reset   — return to the default isometric angle\n\n"
         "Examples:\n"
         "  (view-fit-all)\n"
         "  (view-fit-all :reset)\n"
         "  (view-fit-all :hidden)\n"
         "  (view-fit-all :hidden :reset)")
{
    int reset = 0;
    int include_hidden = 0;
    for (int32_t i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) {
            const uint8_t *kw = janet_unwrap_keyword(argv[i]);
            if (strcmp((const char *)kw, "reset") == 0) {
                reset = 1;
            } else if (strcmp((const char *)kw, "hidden") == 0) {
                include_hidden = 1;
            }
        }
    }
    rust_view_fit_all(include_hidden, reset);
    return janet_wrap_nil();
}

JANET_FN(cad_view_angle,
         "(view-angle yaw pitch ; distance)",
         "Set camera to arbitrary yaw/pitch angles (radians).\n\n"
         "Animates the 3D camera over 0.5s to the given orientation.\n"
         "Yaw and pitch are in radians. An optional third argument sets\n"
         "the camera distance (zoom); omitted preserves the current distance.\n\n"
         "Examples:\n"
         "  (view-angle 0 1.57)      — top view (yaw=0, pitch=~90°)\n"
         "  (view-angle 0 0 100)     — look along +X at distance 100")
{
    janet_arity(argc, 2, 3);
    double yaw = janet_getnumber(argv, 0);
    double pitch = janet_getnumber(argv, 1);
    double dist = (argc > 2) ? janet_getnumber(argv, 2) : 0.0;
    rust_view_set_angles(yaw, pitch, argc > 2, dist);
    return janet_wrap_nil();
}

JANET_FN(cad_edge_thickness,
         "(edge-thickness &opt value)",
         "Get or set the edge line thickness in NDC units.\n\n"
         "Called with no arguments, returns the current thickness.\n"
         "Called with one numeric argument, sets the thickness and returns it.\n\n"
         "Example: (edge-thickness 0.008) — thicker lines\n"
         "         (edge-thickness)      — query")
{
    janet_arity(argc, 0, 1);
    double result;
    if (argc == 0) {
        result = rust_edge_get_thickness();
    } else {
        if (!janet_checktype(argv[0], JANET_NUMBER)) {
            janet_panic("edge-thickness: expected a number");
        }
        double val = janet_unwrap_number(argv[0]);
        rust_edge_set_thickness(val);
        result = val;
    }
    return janet_wrap_number(result);
}

JANET_FN(cad_edge_color_inactive,
         "(edge-color-inactive &opt r g b)",
         "Get or set the inactive edge color as RGB values in [0, 1].\n\n"
         "Called with no arguments, returns the current color as a tuple '(r g b).\n"
         "Called with three numeric arguments (r g b), sets the color.\n\n"
         "Example: (edge-color-inactive 0.8 0.8 0.8)  — light grey\n"
         "         (edge-color-inactive)               — query")
{
    janet_arity(argc, 0, 3);
    if (argc == 0) {
        return janet_wrap_nil();
    }
    if (argc != 3) {
        janet_panic("edge-color-inactive expects 0 or 3 arguments");
    }
    if (!janet_checktype(argv[0], JANET_NUMBER) ||
        !janet_checktype(argv[1], JANET_NUMBER) ||
        !janet_checktype(argv[2], JANET_NUMBER)) {
        janet_panic("edge-color-inactive: r, g, b must be numbers");
    }
    double r = janet_unwrap_number(argv[0]);
    double g = janet_unwrap_number(argv[1]);
    double b = janet_unwrap_number(argv[2]);
    rust_edge_set_color_inactive(r, g, b);
    return janet_wrap_nil();
}

JANET_FN(cad_edge_color_active,
         "(edge-color-active &opt r g b)",
         "Get or set the active (selected) edge color as RGB values in [0, 1].\n\n"
         "Called with no arguments, returns the current color as a tuple '(r g b).\n"
         "Called with three numeric arguments (r g b), sets the color.\n\n"
         "Example: (edge-color-active 0.3 0.5 1.0)  — light blue\n"
         "         (edge-color-active)               — query")
{
    janet_arity(argc, 0, 3);
    if (argc == 0) {
        return janet_wrap_nil();
    }
    if (argc != 3) {
        janet_panic("edge-color-active expects 0 or 3 arguments");
    }
    if (!janet_checktype(argv[0], JANET_NUMBER) ||
        !janet_checktype(argv[1], JANET_NUMBER) ||
        !janet_checktype(argv[2], JANET_NUMBER)) {
        janet_panic("edge-color-active: r, g, b must be numbers");
    }
    double r = janet_unwrap_number(argv[0]);
    double g = janet_unwrap_number(argv[1]);
    double b = janet_unwrap_number(argv[2]);
    rust_edge_set_color_active(r, g, b);
    return janet_wrap_nil();
}

JANET_FN(cad_write_step,
         "(write-step shape path)",
         "Export a shape to a STEP file at the given path. "
         "Returns nil on success, signals an error on failure.")
{
    janet_arity(argc, 2, 2);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    if (!janet_checktype(argv[1], JANET_STRING)) {
        janet_panic("write-step: path must be a string");
    }
    const uint8_t *path_bytes = janet_unwrap_string(argv[1]);
    const char *path = (const char *)path_bytes;

    int result = rust_write_step(data, path);
    if (result != 0) {
        janet_panic("STEP export failed");
    }
    return janet_wrap_nil();
}

JANET_FN(cad_write_stl,
         "(write-stl shape path)",
         "Export a shape to an STL file at the given path. "
         "Returns nil on success, signals an error on failure.")
{
    janet_arity(argc, 2, 2);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    if (!janet_checktype(argv[1], JANET_STRING)) {
        janet_panic("write-stl: path must be a string");
    }
    const uint8_t *path_bytes = janet_unwrap_string(argv[1]);
    const char *path = (const char *)path_bytes;

    int result = rust_write_stl(data, path);
    if (result != 0) {
        janet_panic("STL export failed");
    }
    return janet_wrap_nil();
}

JANET_FN(cad_read_step,
         "(read-step path &keys :eager :hide)",
         "Read a STEP file from disk and return a shape.\n\n"
         "Example:\n"
         "  (read-step \"/tmp/model.step\")       — load from file\n"
         "  (read-step \"/tmp/model.step\" :eager) — load and tessellate\n\n"
         "Returns a rojcad/shape abstract value. Signals an error on failure.")
{
    janet_arity(argc, 1, 1);
    if (!janet_checktype(argv[0], JANET_STRING)) {
        janet_panic("read-step: path must be a string");
    }
    const uint8_t *path_bytes = janet_unwrap_string(argv[0]);
    const char *path = (const char *)path_bytes;
    int eager = has_eager(argv, argc);
    void *shape = alloc_shape();
    CAD_CHECK(rust_init_read_step(shape, path, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

// ── Helper for extracting :plane keyword ─────────────────────────────────

static const char *kw_plane(const Janet *argv, int32_t argc) {
    int idx = find_keyword(argv, argc, "plane");
    if (idx < 0) return "xy";
    if (idx + 1 >= argc) janet_panic(":plane requires a keyword value");
    if (!janet_checktype(argv[idx + 1], JANET_KEYWORD)) janet_panic(":plane expects a keyword (:xy, :xz, :yz, etc.)");
    return (const char *)janet_unwrap_keyword(argv[idx + 1]);
}

// ── 2D Primitives ─────────────────────────────────────────────────────────

JANET_FN(cad_rect,
         "(rect width depth &keys :w :d :h :wire :plane :at :eager :hide)",
         "Create a rectangle.\n\n"
         "Positional: (rect w d)\n"
         "Keywords: :w :d or :h (dimensions), :wire (return Wire instead of Face),\n"
         "         :plane (workplane, default :xy), :at (position [x y z]),\n"
         "         :eager (tessellate immediately), :hide (skip auto-show).\n\n"
         "Examples:\n"
         "  (rect 10 20)                     — on XY plane\n"
         "  (rect :w 10 :d 20 :wire)         — rect wire\n"
         "  (rect :w 10 :h 20)               — :h alias for :d\n"
         "  (rect :w 10 :d 20 :plane :xz :at [5 0 0]) — on XZ plane\n\n"
         "Returns a rojcad/shape abstract value (FACE by default, WIRE with :wire).")
{
    int eager = has_eager(argv, argc);
    double w, d;
    int has_w = kw_double(argv, argc, "w", &w);
    int has_d = kw_double(argv, argc, "d", &d);
    if (!has_d) has_d = kw_double(argv, argc, "h", &d);
    int is_wire = find_keyword(argv, argc, "wire") >= 0 ? 1 : 0;
    double ax, ay, az;
    int has_at = kw_array_3(argv, argc, "at", &ax, &ay, &az);
    const char *plane = kw_plane(argv, argc);

    int pos_count = 0;
    for (int i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) break;
        pos_count++;
    }

    if (has_w && has_d) goto create;
    if (pos_count >= 2) {
        w = janet_unwrap_number(argv[0]);
        d = janet_unwrap_number(argv[1]);
        goto create;
    }
    janet_panic("rect: expected :w and :d keywords, or 2 positional args");

create:
    {
        void *shape = alloc_shape();
        CAD_CHECK(rust_init_rect(shape, w, d, is_wire, plane,
                       has_at ? ax : 0, has_at ? ay : 0, has_at ? az : 0, eager));
        maybe_hide(shape, argv, argc);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_circle,
         "(circle radius &keys :r :wire :plane :at :eager :hide)",
         "Create a circle.\n\n"
         "Positional: (circle radius)\n"
         "Keywords: :r (radius), :wire (return Wire instead of Face),\n"
         "         :plane (workplane, default :xy), :at (position [x y z]),\n"
         "         :eager (tessellate immediately), :hide (skip auto-show).\n\n"
         "Examples:\n"
         "  (circle 5)                       — on XY plane\n"
         "  (circle :r 5 :wire)              — circle wire\n"
         "  (circle :r 5 :plane :xz)         — on XZ plane\n\n"
         "Returns a rojcad/shape abstract value.")
{
    int eager = has_eager(argv, argc);
    double r;
    int has_r = kw_double(argv, argc, "r", &r);
    int is_wire = find_keyword(argv, argc, "wire") >= 0 ? 1 : 0;
    double ax, ay, az;
    int has_at = kw_array_3(argv, argc, "at", &ax, &ay, &az);
    const char *plane = kw_plane(argv, argc);

    if (!has_r) {
        if (argc < 1) janet_panic("circle: radius is required");
        r = janet_unwrap_number(argv[0]);
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_circle(shape, r, is_wire, plane,
                     has_at ? ax : 0, has_at ? ay : 0, has_at ? az : 0, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_polygon,
         "(polygon &keys :pts :wire :plane :at :eager :hide)",
         "Create a polygon from a list of 2D points.\n\n"
         "Keywords: :pts (array of [x y] tuples), :wire (return Wire instead of Face),\n"
         "         :plane (workplane, default :xy), :at (position [x y z]),\n"
         "         :eager (tessellate immediately), :hide (skip auto-show).\n\n"
         "Examples:\n"
         "  (polygon :pts [[0 0] [10 0] [10 10] [0 10]])  — square on XY\n"
         "  (polygon :pts [[0 0] [10 0] [10 10]] :wire)    — L-shaped wire\n\n"
         "Returns a rojcad/shape abstract value.")
{
    int eager = has_eager(argv, argc);
    int is_wire = find_keyword(argv, argc, "wire") >= 0 ? 1 : 0;
    double ax, ay, az;
    int has_at = kw_array_3(argv, argc, "at", &ax, &ay, &az);
    const char *plane = kw_plane(argv, argc);

    int pts_idx = find_keyword(argv, argc, "pts");
    if (pts_idx < 0) janet_panic("polygon: :pts keyword is required");

    Janet pts_val = argv[pts_idx + 1];
    if (!janet_checktype(pts_val, JANET_ARRAY) && !janet_checktype(pts_val, JANET_TUPLE)) {
        janet_panic("polygon: :pts expects an array or tuple of [x y] pairs");
    }

    int32_t npts;
    const Janet *pts_data;
    if (janet_checktype(pts_val, JANET_ARRAY)) {
        JanetArray *arr = janet_unwrap_array(pts_val);
        npts = arr->count;
        pts_data = arr->data;
    } else {
        pts_data = janet_unwrap_tuple(pts_val);
        npts = janet_tuple_length(pts_data);
    }

    // Flatten 2D points into a single array: [x0, y0, x1, y1, ...]
    double *flat = janet_smalloc((size_t)npts * 2 * sizeof(double));
    for (int32_t i = 0; i < npts; i++) {
        if (!janet_checktype(pts_data[i], JANET_ARRAY) && !janet_checktype(pts_data[i], JANET_TUPLE)) {
            janet_panicf("polygon: point %d must be an [x y] array or tuple", i);
        }
        const Janet *pt;
        int32_t ptlen;
        if (janet_checktype(pts_data[i], JANET_ARRAY)) {
            JanetArray *arr = janet_unwrap_array(pts_data[i]);
            pt = arr->data;
            ptlen = arr->count;
        } else {
            pt = janet_unwrap_tuple(pts_data[i]);
            ptlen = janet_tuple_length(pt);
        }
        if (ptlen < 2) janet_panicf("polygon: point %d needs at least 2 coordinates", i);
        flat[i * 2 + 0] = janet_unwrap_number(pt[0]);
        flat[i * 2 + 1] = janet_unwrap_number(pt[1]);
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_polygon(shape, flat, npts * 2, is_wire, plane,
                      has_at ? ax : 0, has_at ? ay : 0, has_at ? az : 0, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

// ── Extrusion / Revolution ────────────────────────────────────────────────

JANET_FN(cad_extrude,
         "(extrude shape &keys :h :z :x :y :dir :both :eager :hide)",
         "Extrude a Face into a Solid.\n\n"
         "Keywords: :h (height, required), :z/:x/:y (cardinal axis),\n"
         "         :dir [dx dy dz] (custom direction),\n"
         "         :both (extrude both sides),\n"
         "         :eager (tessellate immediately), :hide (skip auto-show).\n\n"
         "Default direction is the face normal.\n\n"
         "Examples:\n"
         "  (extrude face :h 20)               — along face normal\n"
         "  (extrude face :h 20 :z)            — along Z axis\n"
         "  (extrude face :h 10 :both)         — both sides\n"
         "  (extrude face :h 5 :dir [0 0 -1])  — custom direction\n\n"
         "Returns a rojcad/shape abstract value (SOLID).")
{
    int eager = has_eager(argv, argc);
    if (argc < 1) janet_panic("extrude: shape is required");
    void *data = unwrap_shape_or_panic(argv[0], 0);

    double height;
    if (!kw_double(argv, argc, "h", &height)) {
        janet_panic("extrude: :h (height) is required");
    }

    int both = find_keyword(argv, argc, "both") >= 0 ? 1 : 0;

    double dx = 0, dy = 0, dz = 0;

    if (find_keyword(argv, argc, "z") >= 0) {
        dz = 1.0;
    } else if (find_keyword(argv, argc, "y") >= 0) {
        dy = 1.0;
    } else if (find_keyword(argv, argc, "x") >= 0) {
        dx = 1.0;
    } else {
        kw_array_3(argv, argc, "dir", &dx, &dy, &dz);
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_extrude(shape, data, height, dx, dy, dz, both, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_revolve,
         "(revolve shape &keys :a :ar :c :dir :eager :hide)",
         "Revolve a Face into a Solid.\n\n"
         "Angle via :a (degrees) or :ar (radians).\n"
         "Axis via :c (point [x y z], default [0 0 0]) and :dir (direction, default [0 0 1]).\n"
         "Keywords: :eager (tessellate immediately), :hide (skip auto-show).\n\n"
         "Examples:\n"
         "  (revolve face :a 360)                     — full revolution about Z\n"
         "  (revolve face :a 180)                     — half revolution\n"
         "  (revolve face :a 180 :c [0 0 0] :dir [0 1 0]) — about Y axis\n\n"
         "Returns a rojcad/shape abstract value (SOLID).")
{
    int eager = has_eager(argv, argc);
    if (argc < 1) janet_panic("revolve: shape is required");
    void *data = unwrap_shape_or_panic(argv[0], 0);

    double angle;
    if (kw_double(argv, argc, "ar", &angle)) {
        /* radians — pass through */
    } else if (kw_double(argv, argc, "a", &angle)) {
        angle *= (M_PI / 180.0);
    } else {
        angle = 2.0 * M_PI; /* default: full circle */
    }

    double ox, oy, oz, dx, dy, dz;
    int has_c = kw_array_3(argv, argc, "c", &ox, &oy, &oz);
    int has_dir = kw_array_3(argv, argc, "dir", &dx, &dy, &dz);

    if (!has_c) { ox = 0; oy = 0; oz = 0; }
    if (!has_dir) { dx = 0; dy = 0; dz = 1; }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_revolve(shape, data, angle, ox, oy, oz, dx, dy, dz, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_extrude_polygon,
         "(extrude-polygon points height &keys :h :plane :at :eager :hide)",
         "Create a Solid by extruding a polygon from points.\n\n"
         "Positional: (extrude-polygon points height)\n"
         "Points is an array of [x y] tuples.\n"
         "Keywords: :h (height), :plane (workplane, default :xy),\n"
         "         :at (position [x y z]),\n"
         "         :eager (tessellate immediately), :hide (skip auto-show).\n\n"
         "Examples:\n"
         "  (extrude-polygon [[0 0][10 0][10 10][0 10]] 20)\n"
         "  (extrude-polygon [[0 0][10 0][10 10]] :h 5)\n\n"
         "Returns a rojcad/shape abstract value (SOLID).")
{
    int eager = has_eager(argv, argc);
    double ax, ay, az;
    int has_at = kw_array_3(argv, argc, "at", &ax, &ay, &az);
    const char *plane = kw_plane(argv, argc);

    double height;
    int has_h = kw_double(argv, argc, "h", &height);

    // Find points value — before any keywords
    if (argc < 1) janet_panic("extrude-polygon: points are required");
    int pos_count = 0;
    for (int i = 0; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) break;
        pos_count++;
    }

    Janet pts_val;
    if (pos_count >= 1) {
        pts_val = argv[0];
    } else {
        int pts_idx = find_keyword(argv, argc, "pts");
        if (pts_idx < 0) janet_panic("extrude-polygon: provide points as first argument or :pts");
        pts_val = argv[pts_idx + 1];
    }

    if (has_h && pos_count >= 2) {
        height = janet_unwrap_number(argv[1]);
    } else if (!has_h) {
        if (pos_count >= 2) {
            height = janet_unwrap_number(argv[1]);
        } else {
            janet_panic("extrude-polygon: height is required");
        }
    }

    if (!janet_checktype(pts_val, JANET_ARRAY) && !janet_checktype(pts_val, JANET_TUPLE)) {
        janet_panic("extrude-polygon: points must be an array or tuple of [x y] pairs");
    }

    int32_t npts;
    const Janet *pts_data;
    if (janet_checktype(pts_val, JANET_ARRAY)) {
        JanetArray *arr = janet_unwrap_array(pts_val);
        npts = arr->count;
        pts_data = arr->data;
    } else {
        pts_data = janet_unwrap_tuple(pts_val);
        npts = janet_tuple_length(pts_data);
    }

    double *flat = janet_smalloc((size_t)npts * 2 * sizeof(double));
    for (int32_t i = 0; i < npts; i++) {
        if (!janet_checktype(pts_data[i], JANET_ARRAY) && !janet_checktype(pts_data[i], JANET_TUPLE)) {
            janet_panicf("extrude-polygon: point %d must be an [x y] array or tuple", i);
        }
        const Janet *pt;
        int32_t ptlen;
        if (janet_checktype(pts_data[i], JANET_ARRAY)) {
            JanetArray *arr = janet_unwrap_array(pts_data[i]);
            pt = arr->data;
            ptlen = arr->count;
        } else {
            pt = janet_unwrap_tuple(pts_data[i]);
            ptlen = janet_tuple_length(pt);
        }
        if (ptlen < 2) janet_panicf("extrude-polygon: point %d needs at least 2 coordinates", i);
        flat[i * 2 + 0] = janet_unwrap_number(pt[0]);
        flat[i * 2 + 1] = janet_unwrap_number(pt[1]);
    }

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_extrude_polygon(shape, flat, npts * 2, height, plane,
                              has_at ? ax : 0, has_at ? ay : 0, has_at ? az : 0, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

// ── Wire Operations ───────────────────────────────────────────────────────

JANET_FN(cad_wire_to_face,
         "(wire-to-face wire &keys :eager :hide)",
         "Convert a Wire shape into a Face by filling its boundary.\n\n"
         "Keywords: :eager, :hide\n\n"
         "Returns a rojcad/shape abstract value (FACE).")
{
    janet_arity(argc, 1, 2);
    int eager = has_eager(argv, argc);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    void *shape = alloc_shape();
    CAD_CHECK(rust_init_wire_to_face(shape, data, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_wire_fillet,
         "(wire-fillet wire &keys :r :eager :hide)",
         "Round all vertices of a closed Wire by a radius.\n\n"
         "Keywords: :r (radius, required), :eager, :hide\n\n"
         "Returns a rojcad/shape abstract value (WIRE).")
{
    int eager = has_eager(argv, argc);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    double radius;
    if (!kw_double(argv, argc, "r", &radius)) {
        janet_panic("wire-fillet: :r (radius) is required");
    }
    void *shape = alloc_shape();
    CAD_CHECK(rust_init_wire_fillet(shape, data, radius, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_wire_chamfer,
         "(wire-chamfer wire &keys :d :eager :hide)",
         "Bevel all vertices of a closed Wire by a distance.\n\n"
         "Keywords: :d (distance, required), :eager, :hide\n\n"
         "Returns a rojcad/shape abstract value (WIRE).")
{
    int eager = has_eager(argv, argc);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    double dist;
    if (!kw_double(argv, argc, "d", &dist)) {
        janet_panic("wire-chamfer: :d (distance) is required");
    }
    void *shape = alloc_shape();
    CAD_CHECK(rust_init_wire_chamfer(shape, data, dist, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_wire_offset,
         "(wire-offset wire &keys :d :eager :hide)",
         "Create a parallel offset of a closed Wire by a distance.\n\n"
         "Keywords: :d (distance, required), :eager, :hide\n\n"
         "Returns a rojcad/shape abstract value (WIRE).")
{
    int eager = has_eager(argv, argc);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    double dist;
    if (!kw_double(argv, argc, "d", &dist)) {
        janet_panic("wire-offset: :d (distance) is required");
    }
    void *shape = alloc_shape();
    CAD_CHECK(rust_init_wire_offset(shape, data, dist, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

// ── Sketch ────────────────────────────────────────────────────────────────

JANET_FN(cad_sketch,
         "(sketch &keys :plane :at)",
         "Create a new sketch on a workplane.\n\n"
         "Keywords: :plane (workplane, default :xy), :at (position [x y z]).\n\n"
         "Returns a rojcad/sketch abstract value. Each sketch operation returns\n"
         "a new sketch — no mutation.\n\n"
         "Examples:\n"
         "  (sketch)                              — XY plane at origin\n"
         "  (sketch :plane :xz :at [10 0 5])      — XZ plane at [10, 0, 5]\n\n"
         "Combine with -> for threading:\n"
         "  (-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))")
{
    double ax, ay, az;
    int has_at = kw_array_3(argv, argc, "at", &ax, &ay, &az);
    const char *plane = kw_plane(argv, argc);

    void *sk = alloc_sketch();
    rust_sketch_new(sk, plane,
                    has_at ? ax : 0, has_at ? ay : 0, has_at ? az : 0);
    return janet_wrap_abstract(sk);
}

/* Helper: unwrap a rojcad/sketch abstract, panic if wrong type */
static void *unwrap_sketch_or_panic(Janet val, int index) {
    JanetAbstract abs = janet_checkabstract(val, &rojcad_sketch_type);
    if (!abs) {
        janet_panicf("expected rojcad/sketch, got %T at argument %d", val, index);
    }
    return abs;
}

/* Helper: apply a sketch operation that returns a new sketch */
static Janet sketch_op(Janet sketch_val, void (*op)(void *, void *, double, double),
                       double a1, double a2) {
    void *src = unwrap_sketch_or_panic(sketch_val, 0);
    void *dest = alloc_sketch();
    op(dest, src, a1, a2);
    return janet_wrap_abstract(dest);
}

JANET_FN(cad_move_to,
         "(move-to sketch x y)",
         "Move the sketch cursor to (x, y) without drawing. Returns a new sketch.")
{
    janet_arity(argc, 3, 3);
    return sketch_op(argv[0], rust_sketch_move_to,
                     janet_unwrap_number(argv[1]), janet_unwrap_number(argv[2]));
}

JANET_FN(cad_line_to,
         "(line-to sketch x y)",
         "Draw a line from the current cursor to (x, y). Returns a new sketch.")
{
    janet_arity(argc, 3, 3);
    return sketch_op(argv[0], rust_sketch_line_to,
                     janet_unwrap_number(argv[1]), janet_unwrap_number(argv[2]));
}

JANET_FN(cad_line_dx,
         "(line-dx sketch dx)",
         "Draw a horizontal line by dx units. Returns a new sketch.")
{
    janet_arity(argc, 2, 2);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *dest = alloc_sketch();
    rust_sketch_line_dx(dest, src, janet_unwrap_number(argv[1]));
    return janet_wrap_abstract(dest);
}

JANET_FN(cad_line_dy,
         "(line-dy sketch dy)",
         "Draw a vertical line by dy units. Returns a new sketch.")
{
    janet_arity(argc, 2, 2);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *dest = alloc_sketch();
    rust_sketch_line_dy(dest, src, janet_unwrap_number(argv[1]));
    return janet_wrap_abstract(dest);
}

JANET_FN(cad_line_dx_dy,
         "(line-dx-dy sketch dx dy)",
         "Draw a line by (dx, dy) offset. Returns a new sketch.")
{
    janet_arity(argc, 3, 3);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *dest = alloc_sketch();
    rust_sketch_line_dx_dy(dest, src,
                           janet_unwrap_number(argv[1]),
                           janet_unwrap_number(argv[2]));
    return janet_wrap_abstract(dest);
}

JANET_FN(cad_arc_to,
         "(arc-to sketch x2 y2 x3 y3)",
         "Draw a circular arc from current cursor through (x2, y2) to (x3, y3). "
         "Returns a new sketch.")
{
    janet_arity(argc, 5, 5);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *dest = alloc_sketch();
    rust_sketch_arc_to(dest, src,
                       janet_unwrap_number(argv[1]),
                       janet_unwrap_number(argv[2]),
                       janet_unwrap_number(argv[3]),
                       janet_unwrap_number(argv[4]));
    return janet_wrap_abstract(dest);
}

JANET_FN(cad_close_sketch,
         "(close-sketch sketch &keys :eager :hide)",
         "Close the sketch and return a Face. Adds a closing edge if needed.\n\n"
         "Keywords: :eager, :hide\n\n"
         "Returns a rojcad/shape abstract value (FACE).")
{
    int eager = has_eager(argv, argc);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *shape = alloc_shape();
    rust_sketch_close(shape, src);
    if (eager) { /* tessellation would happen on show, skip for now */ }
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_build_wire,
         "(build-wire sketch &keys :eager :hide)",
         "Return the sketch as an unclosed Wire. Does not close the loop.\n\n"
         "Keywords: :eager, :hide\n\n"
         "Returns a rojcad/shape abstract value (WIRE).")
{
    int eager = has_eager(argv, argc);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *shape = alloc_shape();
    rust_sketch_build_wire(shape, src);
    if (eager) {}
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

// ── Helper Queries ────────────────────────────────────────────────────────

JANET_FN(cad_wire_q,
         "(wire? shape)",
         "Return true if the shape is a Wire.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    return rust_is_wire(data) ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_face_q,
         "(face? shape)",
         "Return true if the shape is a Face.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    return rust_is_face(data) ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_solid_q,
         "(solid? shape)",
         "Return true if the shape is a Solid.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    return rust_is_solid(data) ? janet_wrap_true() : janet_wrap_false();
}

// ── Text ──────────────────────────────────────────────────────────────────

JANET_FN(cad_text,
         "(text string font-path size &keys :depth :plane :at :eager :hide)",
         "Create a 2D or 3D text shape from a TrueType/OpenType font.\n\n"
         "Positional: (text \"Hello\" \"DejaVuSans.ttf\" 10)\n"
         "Keywords: :depth (extrude 3D text), :plane (\"xy\"/\"xz\"/\"yz\"...),\n"
         "         :at (position [x y z]), :eager, :hide.\n\n"
         "Without :depth returns a Face. With :depth returns an extruded Solid.\n\n"
         "Examples:\n"
         "  (text \"Hi\" \"font.ttf\" 10)              — 2D text face\n"
         "  (text \"Hi\" \"font.ttf\" 10 :depth 5)     — 3D extruded text\n"
         "  (text \"Hi\" \"font.ttf\" 10 :plane \"xz\") — on XZ plane\n"
         "  (text \"Hi\" \"font.ttf\" 10 :at [5 0 0])  — positioned\n\n"
         "Returns a rojcad/shape abstract value.")
{
    janet_arity(argc, 3, -1);

    const char *text_str = (const char *)janet_unwrap_string(argv[0]);
    const char *font_str = (const char *)janet_unwrap_string(argv[1]);
    double size = janet_unwrap_number(argv[2]);

    double depth = 0.0;
    int has_depth = kw_double(argv, argc, "depth", &depth);
    int both = find_keyword(argv, argc, "both") >= 0 ? 1 : 0;
    int eager = has_eager(argv, argc);

    /* Parse :plane and :at */
    const char *plane = NULL;
    int plane_used = 0;
    {
        int pi = find_keyword(argv, argc, "plane");
        if (pi >= 0 && pi + 1 < argc) {
            plane = (const char *)janet_unwrap_keyword(argv[pi + 1]);
            plane_used = 1;
        }
    }
    double ax = 0.0, ay = 0.0, az = 0.0;
    kw_array_3(argv, argc, "at", &ax, &ay, &az);

    void *shape = alloc_shape();
    int rc;
    if (has_depth && depth > 0.0) {
        rc = rust_init_text_extruded(shape, text_str, font_str, size,
                                      depth, both,
                                      plane_used ? plane : NULL,
                                      ax, ay, az, eager);
    } else {
        rc = rust_init_text(shape, text_str, font_str, size,
                             plane_used ? plane : NULL,
                             ax, ay, az, eager);
    }
    if (rc) {
        const char *msg = rust_take_last_error();
        janet_panic(msg);
    }
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_text3d,
         "(text3d string font-path size depth &keys :plane :at :both :eager :hide)",
         "Create a 3D extruded text shape.\n\n"
         "Positional: (text3d \"Hello\" \"DejaVuSans.ttf\" 10 5)\n"
         "Keywords: :plane (\"xy\"/\"xz\"/\"yz\"...), :at (position [x y z]),\n"
         "         :both (extrude symetrically), :eager, :hide.\n\n"
         "Equivalent to (text ... :depth depth).\n\n"
         "Returns a rojcad/shape abstract value.")
{
    janet_arity(argc, 4, -1);

    const char *text_str = (const char *)janet_unwrap_string(argv[0]);
    const char *font_str = (const char *)janet_unwrap_string(argv[1]);
    double size = janet_unwrap_number(argv[2]);
    double depth = janet_unwrap_number(argv[3]);

    int both = find_keyword(argv, argc, "both") >= 0 ? 1 : 0;
    int eager = has_eager(argv, argc);

    const char *plane = NULL;
    int plane_used = 0;
    {
        int pi = find_keyword(argv, argc, "plane");
        if (pi >= 0 && pi + 1 < argc) {
            plane = (const char *)janet_unwrap_keyword(argv[pi + 1]);
            plane_used = 1;
        }
    }
    double ax = 0.0, ay = 0.0, az = 0.0;
    kw_array_3(argv, argc, "at", &ax, &ay, &az);

    void *shape = alloc_shape();
    CAD_CHECK(rust_init_text_extruded(shape, text_str, font_str, size,
                                       depth, both,
                                       plane_used ? plane : NULL,
                                       ax, ay, az, eager));
    maybe_hide(shape, argv, argc);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_list_fonts,
         "(list-fonts)",
         "List available system fonts.\n\n"
         "Scans standard OS font directories and returns an array of\n"
         "[name path aspect] tuples for each discovered TTF/OTF font.\n"
         "Aspect is :regular, :bold, :italic, or :bold-italic.\n\n"
         "Examples:\n"
         "  (list-fonts)  — all system fonts\n"
         "  (keep [name path] (list-fonts)) — just names and paths\n\n"
         "Returns an array of tuples.")
{
    janet_arity(argc, 0, 0);

    int count = 0;
    char **list = rust_list_fonts(&count);

    JanetArray *arr = janet_array(count);
    for (int i = 0; i < count; i++) {
        /* Split "name|path|aspect" */
        char *entry = list[i];
        char *sep1 = strchr(entry, '|');
        if (!sep1) continue;
        *sep1 = '\0';
        char *name = entry;
        char *rest = sep1 + 1;
        char *sep2 = strchr(rest, '|');
        if (!sep2) continue;
        *sep2 = '\0';
        char *path = rest;
        char *aspect_str = sep2 + 1;

        /* Build Janet tuple (name path aspect) */
        Janet parts[3];
        parts[0] = janet_cstringv(name);
        parts[1] = janet_cstringv(path);
        /* Map aspect string to keyword */
        JanetKeyword kw;
        if (strcmp(aspect_str, "bold") == 0) {
            kw = janet_ckeyword("bold");
        } else if (strcmp(aspect_str, "italic") == 0) {
            kw = janet_ckeyword("italic");
        } else if (strcmp(aspect_str, "bold-italic") == 0) {
            kw = janet_ckeyword("bold-italic");
        } else {
            kw = janet_ckeyword("regular");
        }
        parts[2] = janet_wrap_keyword(kw);

        const Janet *tup = janet_tuple_n(parts, 3);
        janet_array_push(arr, janet_wrap_tuple(tup));
    }

    rust_free_fonts_list(list, count);
    return janet_wrap_array(arr);
}

// ── Shape Query Functions ────────────────────────────────────────────────────

JANET_FN(cad_selected_shapes,
         "(selected-shapes)",
         "Return a tuple of ShapeData abstract values currently selected in the"
         " 3D viewer.\n\n"
         "Returns an empty tuple `()` if nothing is selected.\n\n"
         "Examples:\n"
         "  (selected-shapes)                     — get selected shapes\n"
         "  (each s (selected-shapes) (hide s))   — hide all selected\n\n"
         "Returns a tuple of rojcad/shape abstract values.")
{
    janet_arity(argc, 0, 0);
    (void)argv;

    size_t count = 0;
    uint64_t *ids = rust_get_selected_shape_ids(&count);

    Janet *parts = janet_smalloc(sizeof(Janet) * count);
    for (size_t i = 0; i < count; i++) {
        void *ptr = rust_get_shape_pointer(ids[i]);
        parts[i] = ptr ? janet_wrap_abstract(ptr) : janet_wrap_nil();
    }
    const Janet *tup = janet_tuple_n(parts, count);
    janet_sfree(parts);
    rust_free_u64_array(ids, count);
    return janet_wrap_tuple(tup);
}

JANET_FN(cad_list_shapes,
         "(list-shapes &keys :visible :hidden)",
         "Return a tuple of all registered ShapeData abstract values,"
         " optionally filtered by visibility.\n\n"
         "With no arguments, returns all registered shapes.\n"
         "With :visible, returns only visible shapes.\n"
         "With :hidden, returns only hidden shapes.\n"
         "If both :visible and :hidden are given, :hidden takes precedence.\n\n"
         "Only shapes that have been shown (registered in the viewer)"
         " are included.\n\n"
         "Examples:\n"
         "  (list-shapes)                    — all registered shapes\n"
         "  (list-shapes :visible)           — visible shapes only\n"
         "  (list-shapes :hidden)            — hidden shapes only\n"
         "  (each s (list-shapes) (print s)) — print all shapes\n\n"
         "Returns a tuple of rojcad/shape abstract values.")
{
    int hidden = find_keyword(argv, argc, "hidden") >= 0 ? 1 : 0;
    int visible = find_keyword(argv, argc, "visible") >= 0 ? 1 : 0;

    uint8_t filter = 0;
    if (hidden) {
        filter = 2;
    } else if (visible) {
        filter = 1;
    }

    size_t count = 0;
    uint64_t *ids = rust_get_registered_shape_ids(filter, &count);

    Janet *parts = janet_smalloc(sizeof(Janet) * count);
    for (size_t i = 0; i < count; i++) {
        void *ptr = rust_get_shape_pointer(ids[i]);
        parts[i] = ptr ? janet_wrap_abstract(ptr) : janet_wrap_nil();
    }
    const Janet *tup = janet_tuple_n(parts, count);
    janet_sfree(parts);
    rust_free_u64_array(ids, count);
    return janet_wrap_tuple(tup);
}

/* ── CAD function metadata ────────────────────────────────────────────────── */

static const char *cad_fn_categories[][2] = {
    {"box", "primitives"},
    {"sphere", "primitives"},
    {"cylinder", "primitives"},
    {"cone", "primitives"},
    {"torus", "primitives"},
    {"cut", "booleans"},
    {"common", "booleans"},
    {"fuse", "booleans"},
    {"translate", "transforms"},
    {"rotate", "transforms"},
    {"scale", "transforms"},
    {"mirror", "transforms"},
    {"shape-type", "queries"},
    {"visible?", "queries"},
    {"purge", "registry"},
    {"hide", "registry"},
    {"show", "registry"},
    {"registry-remove", "registry"},
    {"write-step", "io"},
    {"write-stl", "io"},
    {"read-step", "io"},
    {"on-select", "selection"},
    {"poll-selection", "selection"},
    {"edge-toggle-inactive", "edge-styling"},
    {"edge-toggle-active", "edge-styling"},
    {"edge-inactive-show?", "edge-styling"},
    {"edge-active-show?", "edge-styling"},
    {"edge-thickness", "edge-styling"},
    {"edge-color-inactive", "edge-styling"},
    {"edge-color-active", "edge-styling"},

    {"rect", "2d-primitives"},
    {"circle", "2d-primitives"},
    {"polygon", "2d-primitives"},
    {"extrude", "operations"},
    {"revolve", "operations"},
    {"extrude-polygon", "operations"},
    {"wire-to-face", "wire-operations"},
    {"wire-fillet", "wire-operations"},
    {"wire-chamfer", "wire-operations"},
    {"wire-offset", "wire-operations"},
    {"sketch", "sketch"},
    {"move-to", "sketch"},
    {"line-to", "sketch"},
    {"line-dx", "sketch"},
    {"line-dy", "sketch"},
    {"line-dx-dy", "sketch"},
    {"arc-to", "sketch"},
    {"close-sketch", "sketch"},
    {"build-wire", "sketch"},
    {"wire?", "queries"},
    {"face?", "queries"},
    {"solid?", "queries"},
    {"view-fit", "view"},
    {"view-fit-all", "view"},
    {"view-angle", "view"},
    {"stats-overlay", "view"},
    {"window-help-toggle", "view"},
    {"window-help-show?", "view"},
    {"window-help-show", "view"},
    {"text", "text"},
    {"text3d", "text"},
    {"list-fonts", "text"},
    {"selected-shapes", "queries"},
    {"list-shapes", "queries"},

    {"quit-requested",         "view"},
    {"edge-hidden-toggle",     "edge-styling"},
    {"edge-hidden-show?",      "edge-styling"},
    {"edge-hidden",            "edge-styling"},
    {"projection-toggle",      "view"},
    {"projection-perspective", "view"},
    {"window-size",            "view"},
    {"window-size?",           "view"},
    {"window-fullscreen",      "view"},
    {"window-fullscreen?",     "view"},
    {"window-maximized",       "view"},
    {"window-maximized?",      "view"},
    {NULL, NULL}
};

/* ── Registration ───────────────────────────────────────────────────────── */

void cad_register_functions(JanetTable *env) {
    /* Set runtime fields (gc finalizer and tostring) that reference
     * C functions defined in this translation unit. */
    rojcad_shape_type.gc = shape_gc_finish;
    rojcad_shape_type.tostring = shape_to_string;
    rojcad_sketch_type.gc = sketch_gc_finish;
    rojcad_sketch_type.tostring = sketch_to_string;

    /* Manual 3-field JanetReg array (avoid JANET_REG macros which emit 5-field
     * JanetRegExt initializers, triggering -Wexcess-initializers warnings). */
    JanetReg cfuns[] = {
        {"box",                    cad_box,                    cad_box_docstring_},
        {"sphere",                 cad_sphere,                 cad_sphere_docstring_},
        {"cylinder",               cad_cylinder,               cad_cylinder_docstring_},
        {"cone",                   cad_cone,                   cad_cone_docstring_},
        {"torus",                  cad_torus,                  cad_torus_docstring_},
        {"cut",                    cad_cut,                    cad_cut_docstring_},
        {"common",                 cad_common,                 cad_common_docstring_},
        {"fuse",                   cad_fuse,                   cad_fuse_docstring_},
        {"translate",              cad_translate,              cad_translate_docstring_},
        {"rotate",                 cad_rotate,                 cad_rotate_docstring_},
        {"scale",                  cad_scale,                  cad_scale_docstring_},
        {"mirror",                 cad_mirror,                 cad_mirror_docstring_},
        {"shape-type",             cad_shape_type,             cad_shape_type_docstring_},

        {"purge",                  cad_purge,                  cad_purge_docstring_},
        {"hide",                   cad_hide,                   cad_hide_docstring_},
        {"show",                   cad_show,                   cad_show_docstring_},
        {"registry-remove",        cad_registry_remove,        cad_registry_remove_docstring_},
        {"visible?",               cad_visible_q,              cad_visible_q_docstring_},
        {"write-step",             cad_write_step,             cad_write_step_docstring_},
        {"write-stl",              cad_write_stl,              cad_write_stl_docstring_},
        {"read-step",              cad_read_step,              cad_read_step_docstring_},
        {"on-select",              cad_on_select,              cad_on_select_docstring_},
        {"poll-selection",         cad_poll_selection,         cad_poll_selection_docstring_},
        {"selected-shapes",        cad_selected_shapes,        cad_selected_shapes_docstring_},
        {"list-shapes",            cad_list_shapes,            cad_list_shapes_docstring_},
        {"quit-requested",         cad_quit_requested,         cad_quit_requested_docstring_},
        {"edge-toggle-inactive",   cad_edge_toggle_inactive,   cad_edge_toggle_inactive_docstring_},
        {"edge-toggle-active",     cad_edge_toggle_active,     cad_edge_toggle_active_docstring_},
        {"edge-inactive-show?",    cad_edge_inactive_showing,  cad_edge_inactive_showing_docstring_},
        {"edge-active-show?",      cad_edge_active_showing,    cad_edge_active_showing_docstring_},
        {"edge-thickness",         cad_edge_thickness,         cad_edge_thickness_docstring_},
        {"edge-color-inactive",    cad_edge_color_inactive,    cad_edge_color_inactive_docstring_},
        {"edge-color-active",      cad_edge_color_active,      cad_edge_color_active_docstring_},
        {"edge-hidden-toggle",     cad_edge_hidden_toggle,     cad_edge_hidden_toggle_docstring_},
        {"edge-hidden-show?",      cad_edge_hidden_showing,    cad_edge_hidden_showing_docstring_},
        {"edge-hidden",            cad_edge_hidden,            cad_edge_hidden_docstring_},
        {"projection-toggle",      cad_projection_toggle,      cad_projection_toggle_docstring_},
        {"projection-perspective", cad_projection_perspective, cad_projection_perspective_docstring_},

        /* Stats overlay */
        {"stats-overlay",          cad_stats_overlay,          cad_stats_overlay_docstring_},

        /* Help overlay */
        {"window-help-toggle",     cad_help_toggle,            cad_help_toggle_docstring_},
        {"window-help-show?",      cad_help_showing,           cad_help_showing_docstring_},
        {"window-help-show",       cad_help_set,               cad_help_set_docstring_},

        /* Window size */
        {"window-size",            cad_window_size,            cad_window_size_docstring_},
        {"window-size?",           cad_window_size_query,      cad_window_size_query_docstring_},

        /* Window fullscreen */
        {"window-fullscreen",      cad_window_fullscreen,      cad_window_fullscreen_docstring_},
        {"window-fullscreen?",     cad_window_fullscreen_query,cad_window_fullscreen_query_docstring_},

        /* Window maximized */
        {"window-maximized",       cad_window_maximized,       cad_window_maximized_docstring_},
        {"window-maximized?",      cad_window_maximized_query, cad_window_maximized_query_docstring_},

        /* View fit */
        {"view-fit",               cad_view_fit,               cad_view_fit_docstring_},
        {"view-fit-all",           cad_view_fit_all,           cad_view_fit_all_docstring_},

        /* View angle (low-level primitive) */
        {"view-angle",             cad_view_angle,             cad_view_angle_docstring_},

        /* 2D primitives */
        {"rect",                   cad_rect,                   cad_rect_docstring_},
        {"circle",                 cad_circle,                 cad_circle_docstring_},
        {"polygon",                cad_polygon,                cad_polygon_docstring_},

        /* Extrusion / Revolution */
        {"extrude",                cad_extrude,                cad_extrude_docstring_},
        {"revolve",                cad_revolve,                cad_revolve_docstring_},
        {"extrude-polygon",        cad_extrude_polygon,        cad_extrude_polygon_docstring_},

        /* Wire operations */
        {"wire-to-face",           cad_wire_to_face,           cad_wire_to_face_docstring_},
        {"wire-fillet",            cad_wire_fillet,            cad_wire_fillet_docstring_},
        {"wire-chamfer",           cad_wire_chamfer,           cad_wire_chamfer_docstring_},
        {"wire-offset",            cad_wire_offset,            cad_wire_offset_docstring_},

        /* Sketch */
        {"sketch",                 cad_sketch,                 cad_sketch_docstring_},
        {"move-to",                cad_move_to,                cad_move_to_docstring_},
        {"line-to",                cad_line_to,                cad_line_to_docstring_},
        {"line-dx",                cad_line_dx,                cad_line_dx_docstring_},
        {"line-dy",                cad_line_dy,                cad_line_dy_docstring_},
        {"line-dx-dy",             cad_line_dx_dy,             cad_line_dx_dy_docstring_},
        {"arc-to",                 cad_arc_to,                 cad_arc_to_docstring_},
        {"close-sketch",           cad_close_sketch,           cad_close_sketch_docstring_},
        {"build-wire",             cad_build_wire,             cad_build_wire_docstring_},

        /* Helper queries */
        {"wire?",                  cad_wire_q,                 cad_wire_q_docstring_},
        {"face?",                  cad_face_q,                 cad_face_q_docstring_},
        {"solid?",                 cad_solid_q,                cad_solid_q_docstring_},

        /* Text */
        {"text",                   cad_text,                   cad_text_docstring_},
        {"text3d",                 cad_text3d,                 cad_text3d_docstring_},
        {"list-fonts",             cad_list_fonts,             cad_list_fonts_docstring_},
        {NULL, NULL, NULL}
    };

    janet_cfuns(env, NULL, cfuns);

    /* Tag each CAD function with :source (for cad-fns filtering) and
     * :category (for group display) metadata. */
    for (int32_t i = 0; cad_fn_categories[i][0] != NULL; i++) {
        Janet sym = janet_csymbolv(cad_fn_categories[i][0]);
        Janet binding = janet_table_get(env, sym);
        if (janet_checktype(binding, JANET_TABLE)) {
            JanetTable *t = janet_unwrap_table(binding);
            janet_table_put(t, janet_ckeywordv("source"), janet_cstringv("rojcad"));
            janet_table_put(t, janet_ckeywordv("category"), janet_cstringv(cad_fn_categories[i][1]));
        }
    }
}
