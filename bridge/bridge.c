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

/* Shape constructors — initialize at destination pointer */
extern void rust_init_box(void *dest,
                           double width, double depth, double height,
                           const double *cx, const double *cy, const double *cz);
extern void rust_init_cube(void *dest,
                            double size,
                            const double *cx, const double *cy, const double *cz);
extern void rust_init_box_from_corners(void *dest,
                                        double c1x, double c1y, double c1z,
                                        double c2x, double c2y, double c2z);
extern void rust_init_sphere(void *dest,
                              double radius,
                              const double *cx, const double *cy, const double *cz,
                              const double *angle);
extern void rust_init_cylinder(void *dest,
                                double radius, double height,
                                const double *cx, const double *cy, const double *cz);
extern void rust_init_cylinder_from_points(void *dest,
                                            double p1x, double p1y, double p1z,
                                            double p2x, double p2y, double p2z,
                                            double radius);
extern void rust_init_cylinder_point_dir(void *dest,
                                          double px, double py, double pz,
                                          double radius,
                                          double dx, double dy, double dz,
                                          double height);
extern void rust_init_cone(void *dest,
                            double bottom_radius, double top_radius, double height,
                            const double *cx, const double *cy, const double *cz,
                            const double *angle);
extern void rust_init_torus(void *dest,
                             double ring_radius, double tube_radius,
                             const double *cx, const double *cy, const double *cz,
                             const double *zx, const double *zy, const double *zz,
                             const double *angle,
                             const double *angle_start, const double *angle_end);

/* Boolean operations — allocate new shape via janet_abstract internally */
extern void rust_init_cut(void *dest, void *a, void *b);
extern void rust_init_common(void *dest, void *a, void *b);

/* Inspection */
extern const char *rust_shape_type(void *data);

/* Export */
extern int rust_write_step(void *data, const char *path);
extern int rust_write_stl(void *data, const char *path);

/* Visibility */
extern void rust_shape_set_visible(void *data, int visible);
extern int rust_shape_get_visible(void *data);

/* Selection */
extern uint64_t rust_poll_selection(void);

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

/* Allocate a new rojcad/shape abstract via Janet GC */
static void *alloc_shape(void) {
    void *data = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!data) {
        janet_panic("failed to allocate shape");
    }
    return data;
}

/* ── JANET_FN implementations ───────────────────────────────────────────── */

/* With JANET_NO_SOURCEMAPS defined (see build.rs), JANET_FN expands to
 * JANET_FN_D(CNAME, USAGE, DOCSTRING), which creates a static docstring
 * combining USAGE and DOCSTRING separated by "\n\n". */

JANET_FN(cad_box,
         "(box width depth height &keys :w :d :h :c :pl :ph)",
         "Create a box or cube.\n\n"
         "Positional: (box w d h) or (box size) for a cube.\n"
         "Keywords: :w :d :h (dimensions), :c (center [x y z]),\n"
         "         :pl :ph (opposite corners [x y z]).\n\n"
         "Examples:\n"
         "  (box 10 20 30)           — box at origin\n"
         "  (box 10 20 30 :c [5 5 5]) — centered box\n"
         "  (box 5)                  — 5x5x5 cube\n"
         "  (box :pl [0 0 0] :ph [10 20 30]) — from corners\n"
         "  (box :w 10 :d 20 :h 30) — keyword style\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, pl[3], ph[3];
    int has_c, has_pl, has_ph;

    has_pl = kw_array_3(argv, argc, "pl", &pl[0], &pl[1], &pl[2]);
    has_ph = kw_array_3(argv, argc, "ph", &ph[0], &ph[1], &ph[2]);

    if (has_pl || has_ph) {
        if (!has_pl || !has_ph) {
            janet_panic("box: :pl and :ph must both be provided");
        }
        void *shape = alloc_shape();
        rust_init_box_from_corners(shape, pl[0], pl[1], pl[2], ph[0], ph[1], ph[2]);
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
        rust_init_box(shape, w, d, h,
                      has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL);
        return janet_wrap_abstract(shape);
    }

    if (has_w || has_d || has_h) {
        janet_panic("box: specify :w, :d, :h together, or use positional args");
    }

    if (argc == 1) {
        double size = janet_unwrap_number(argv[0]);
        void *shape = alloc_shape();
        rust_init_cube(shape, size,
                       has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL);
        return janet_wrap_abstract(shape);
    }

    if (argc >= 3) {
        w = janet_unwrap_number(argv[0]);
        d = janet_unwrap_number(argv[1]);
        h = janet_unwrap_number(argv[2]);
        void *shape = alloc_shape();
        rust_init_box(shape, w, d, h,
                      has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL);
        return janet_wrap_abstract(shape);
    }

    janet_panicf("box: expected 1 or 3 positional arguments, got %d", argc);
}

JANET_FN(cad_sphere,
         "(sphere radius &keys :r :c :a)",
         "Create a sphere.\n\n"
         "Positional: (sphere radius)\n"
         "Keywords: :r (radius), :c (center [x y z]), :a (angle in radians).\n\n"
         "Examples:\n"
         "  (sphere 10)               — full sphere at origin\n"
         "  (sphere 10 :c [1 2 3])    — repositioned\n"
         "  (sphere 10 :a 3.14159)    — hemisphere\n"
         "  (sphere :r 10)            — keyword style\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double radius, cx, cy, cz, angle;
    int has_c, has_a;

    has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    has_a = kw_double(argv, argc, "a", &angle);

    /* Try keyword :r first, then positional */
    if (!kw_double(argv, argc, "r", &radius)) {
        if (argc < 1) janet_panic("sphere: radius is required");
        radius = janet_unwrap_number(argv[0]);
    }

    void *shape = alloc_shape();
    rust_init_sphere(shape, radius,
                     has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL,
                     has_a ? &angle : NULL);
    return janet_wrap_abstract(shape);
}

JANET_FN(cad_cylinder,
         "(cylinder radius height &keys :r :h :c :dir :fp :tp)",
         "Create a cylinder.\n\n"
         "Positional: (cylinder radius height) — along Z axis, base at Z=0\n"
         "Keywords: :r (radius), :h (height), :c (center [x y z]),\n"
         "         :dir (direction [dx dy dz]),\n"
         "         :fp (from-point [x y z]), :tp (to-point [x y z]).\n\n"
         "Examples:\n"
         "  (cylinder 5 10)                       — simple\n"
         "  (cylinder 5 10 :c [0 0 5])            — centered\n"
         "  (cylinder :fp [0 0 0] :tp [0 0 10] :r 5) — point-to-point\n"
         "  (cylinder :r 5 :h 10)                 — keyword style\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, dir[3], fp[3], tp[3];
    int has_c, has_dir, has_fp, has_tp;

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
        rust_init_cylinder_from_points(shape, fp[0], fp[1], fp[2], tp[0], tp[1], tp[2], r);
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
        rust_init_cylinder_point_dir(shape, ox, oy, oz, radius, dir[0], dir[1], dir[2], height);
        return janet_wrap_abstract(shape);
    }

    {
        void *shape = alloc_shape();
        rust_init_cylinder(shape, radius, height,
                           has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_cone,
         "(cone bottom-radius height &keys :br :tr :h :c :a)",
         "Create a cone or truncated cone.\n\n"
         "Positional: (cone br h) for full cone, (cone br tr h) for truncated.\n"
         "Keywords: :br (bottom radius), :tr (top radius), :h (height),\n"
         "         :c (center [x y z]), :a (angle in radians, partial cone).\n\n"
         "Examples:\n"
         "  (cone 5 10)                — full cone, br=5, h=10\n"
         "  (cone 5 3 10)              — truncated cone\n"
         "  (cone 5 10 :a 3.14159)     — half cone\n"
         "  (cone :br 5 :h 10)         — keyword style\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, angle;
    int has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    int has_a = kw_double(argv, argc, "a", &angle);

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
        rust_init_cone(shape, br, tr, h,
                       has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL,
                       has_a ? &angle : NULL);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_torus,
         "(torus ring-radius tube-radius &keys :rr :tr :c :a :as :ae :dir)",
         "Create a torus.\n\n"
         "Positional: (torus rr tr)\n"
         "Keywords: :rr (ring radius), :tr (tube radius),\n"
         "         :c (center [x y z]), :a (angle in radians, partial),\n"
         "         :as (angle start), :ae (angle end),\n"
         "         :dir (axis direction [dx dy dz]).\n\n"
         "Examples:\n"
         "  (torus 20 10)                    — full torus\n"
         "  (torus 20 10 :c [0 0 5])         — repositioned\n"
         "  (torus 20 10 :a 3.14159)         — half torus\n"
         "  (torus :rr 20 :tr 10 :as 0 :ae 3.14) — angled range\n"
         "  (torus :rr 20 :tr 10 :dir [0 1 0]) — oriented\n\n"
         "Returns a rojcad/shape abstract value.")
{
    double cx, cy, cz, dir[3], angle, a_start, a_end;
    int has_c = kw_array_3(argv, argc, "c", &cx, &cy, &cz);
    int has_dir = kw_array_3(argv, argc, "dir", &dir[0], &dir[1], &dir[2]);
    int has_a = kw_double(argv, argc, "a", &angle);
    int has_as = kw_double(argv, argc, "as", &a_start);
    int has_ae = kw_double(argv, argc, "ae", &a_end);

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
        rust_init_torus(shape, rr, tr,
                        has_c ? &cx : NULL, has_c ? &cy : NULL, has_c ? &cz : NULL,
                        has_dir ? &dir[0] : NULL,
                        has_dir ? &dir[1] : NULL,
                        has_dir ? &dir[2] : NULL,
                        has_a ? &angle : NULL,
                        has_as ? &a_start : NULL,
                        has_ae ? &a_end : NULL);
        return janet_wrap_abstract(shape);
    }
}

JANET_FN(cad_cut,
         "(cut shape-a shape-b)",
         "Subtract shape-b from shape-a. Returns a new rojcad/shape "
         "representing the resulting solid.\n\n"
         "Signals an error if the shapes do not intersect or produce "
         "an empty result.")
{
    janet_arity(argc, 2, 2);
    void *a = unwrap_shape_or_panic(argv[0], 0);
    void *b = unwrap_shape_or_panic(argv[1], 1);

    void *result = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!result) {
        janet_panic("failed to allocate shape");
    }
    /* rust_init_cut will panic on failure via Rust-side panic */
    rust_init_cut(result, a, b);

    return janet_wrap_abstract(result);
}

JANET_FN(cad_common,
         "(common shape-a shape-b)",
         "Intersect shape-a with shape-b. Returns a new rojcad/shape "
         "representing the shared volume.\n\n"
         "Signals an error if the shapes do not intersect.")
{
    janet_arity(argc, 2, 2);
    void *a = unwrap_shape_or_panic(argv[0], 0);
    void *b = unwrap_shape_or_panic(argv[1], 1);

    void *result = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!result) {
        janet_panic("failed to allocate shape");
    }
    rust_init_common(result, a, b);

    return janet_wrap_abstract(result);
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

JANET_FN(cad_hide,
         "(hide shape)",
         "Set a shape's visible flag to false. Returns nil.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    rust_shape_set_visible(data, 0);
    return janet_wrap_nil();
}

JANET_FN(cad_show,
         "(show shape)",
         "Set a shape's visible flag to true. Returns nil.")
{
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    rust_shape_set_visible(data, 1);
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
         " in the viewer. The function receives the selected shape's ID"
         " as an integer, or nil when deselected.\n\n"
         "Pass nil to unregister the callback.")
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
         " selected, or :deselected if the selection was cleared.\n\n"
         "If a callback was registered via (on-select), it will be"
         " invoked automatically with the result.")
{
    janet_arity(argc, 0, 0);
    uint64_t result = rust_poll_selection();
    if (result == 0) {
        return janet_wrap_nil();
    }

    Janet event;
    if (result == UINT64_MAX) {
        event = janet_wrap_nil();
    } else {
        event = janet_wrap_number((double)result);
    }

    /* Invoke the stored callback if registered */
    if (janet_checktype(on_select_callback, JANET_FUNCTION)) {
        JanetFunction *fn = janet_unwrap_function(on_select_callback);
        Janet args[] = { event };
        janet_call(fn, 1, args);
    }

    return event;
}

JANET_FN(cad_edge_toggle_inactive,
         "(edge-toggle-inactive)",
         "Toggle visibility of edges on non-selected shapes. "
         "Returns true if inactive edges are now visible, false if hidden.\n\n"
         "Example: (edge-toggle-inactive)")
{
    janet_arity(argc, 0, 0);
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
    int result = rust_edge_toggle_active();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_inactive_showing,
         "(edge-inactive-show?)",
         "Return true if edges on non-selected shapes are currently visible, "
         "false if hidden.")
{
    janet_arity(argc, 0, 0);
    int result = rust_edge_inactive_showing();
    return result ? janet_wrap_true() : janet_wrap_false();
}

JANET_FN(cad_edge_active_showing,
         "(edge-active-show?)",
         "Return true if edges on the selected shape are currently visible, "
         "false if hidden.")
{
    janet_arity(argc, 0, 0);
    int result = rust_edge_active_showing();
    return result ? janet_wrap_true() : janet_wrap_false();
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

/* ── Registration ───────────────────────────────────────────────────────── */

void cad_register_functions(JanetTable *env) {
    /* Set runtime fields (gc finalizer and tostring) that reference
     * C functions defined in this translation unit. */
    rojcad_shape_type.gc = shape_gc_finish;
    rojcad_shape_type.tostring = shape_to_string;

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
        {"shape-type",             cad_shape_type,             cad_shape_type_docstring_},
        {"hide",                   cad_hide,                   cad_hide_docstring_},
        {"show",                   cad_show,                   cad_show_docstring_},
        {"visible?",               cad_visible_q,              cad_visible_q_docstring_},
        {"write-step",             cad_write_step,             cad_write_step_docstring_},
        {"write-stl",              cad_write_stl,              cad_write_stl_docstring_},
        {"on-select",              cad_on_select,              cad_on_select_docstring_},
        {"poll-selection",         cad_poll_selection,         cad_poll_selection_docstring_},
        {"edge-toggle-inactive",   cad_edge_toggle_inactive,   cad_edge_toggle_inactive_docstring_},
        {"edge-toggle-active",     cad_edge_toggle_active,     cad_edge_toggle_active_docstring_},
        {"edge-inactive-show?",    cad_edge_inactive_showing,  cad_edge_inactive_showing_docstring_},
        {"edge-active-show?",      cad_edge_active_showing,    cad_edge_active_showing_docstring_},
        {"edge-thickness",         cad_edge_thickness,         cad_edge_thickness_docstring_},
        {"edge-color-inactive",    cad_edge_color_inactive,    cad_edge_color_inactive_docstring_},
        {"edge-color-active",      cad_edge_color_active,      cad_edge_color_active_docstring_},
        {NULL, NULL, NULL}
    };

    janet_cfuns(env, NULL, cfuns);
}
