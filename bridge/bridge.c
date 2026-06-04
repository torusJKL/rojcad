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
extern void rust_init_sphere(void *dest,
                              double radius,
                              const double *cx, const double *cy, const double *cz);

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

/* ── Helper: check if a Janet value is a rojcad/shape abstract ──────────── */

static void *unwrap_shape_or_panic(Janet val, int index) {
    JanetAbstract abs = janet_checkabstract(val, &rojcad_shape_type);
    if (!abs) {
        janet_panicf("expected rojcad/shape, got %T at argument %d", val, index);
    }
    return abs;
}

/* Helper: parse optional :center keyword tuple.
 * Returns 0 if no :center given, 1 if parsed successfully, panics on error.
 * Searches through argv from kw_start to argc for :center keyword. */
static int parse_center_keyword(const Janet *argv, int32_t argc, int32_t kw_start,
                                 double *cx, double *cy, double *cz) {
    for (int32_t i = kw_start; i < argc; i++) {
        if (janet_checktype(argv[i], JANET_KEYWORD)) {
            const uint8_t *kw = janet_unwrap_keyword(argv[i]);
            if (strcmp((const char *)kw, "center") == 0) {
                if (i + 1 >= argc) {
                    janet_panic(":center keyword requires a tuple argument");
                }
                Janet tuple_val = argv[i + 1];
                if (!janet_checktype(tuple_val, JANET_TUPLE)) {
                    janet_panicf(":center expects a tuple, got %T", tuple_val);
                }
                const Janet *parts = janet_unwrap_tuple(tuple_val);
                int32_t tlen = janet_tuple_length(parts);
                if (tlen != 3) {
                    janet_panicf(":center tuple must have 3 elements, got %d", tlen);
                }
                *cx = janet_unwrap_number(parts[0]);
                *cy = janet_unwrap_number(parts[1]);
                *cz = janet_unwrap_number(parts[2]);
                return 1;
            }
        }
    }
    return 0;
}

/* ── JANET_FN implementations ───────────────────────────────────────────── */

/* With JANET_NO_SOURCEMAPS defined (see build.rs), JANET_FN expands to
 * JANET_FN_D(CNAME, USAGE, DOCSTRING), which creates a static docstring
 * combining USAGE and DOCSTRING separated by "\n\n". */

JANET_FN(cad_make_box,
         "(make-box width depth height &keys :center)",
         "Create a box with the given width, depth, and height. "
         "The box extends from the origin into positive XYZ.\n\n"
         "Optional :center keyword provides a tuple '(cx cy cz) to "
         "position the geometric center of the box.\n\n"
         "Returns a rojcad/shape abstract value.")
{
    /* Validate arity and types manually */
    if (argc < 3) {
        janet_panicf("make-box expects at least 3 arguments, got %d", argc);
    }
    if (!janet_checktype(argv[0], JANET_NUMBER) ||
        !janet_checktype(argv[1], JANET_NUMBER) ||
        !janet_checktype(argv[2], JANET_NUMBER)) {
        janet_panic("make-box: width, depth, height must be numbers");
    }

    double width = janet_unwrap_number(argv[0]);
    double depth = janet_unwrap_number(argv[1]);
    double height = janet_unwrap_number(argv[2]);

    double cx = 0, cy = 0, cz = 0;
    int has_center = parse_center_keyword(argv, argc, 3, &cx, &cy, &cz);

    /* Allocate via Janet GC and initialize via Rust */
    void *shape_data = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!shape_data) {
        janet_panic("failed to allocate shape");
    }
    rust_init_box(shape_data, width, depth, height,
                  has_center ? &cx : NULL,
                  has_center ? &cy : NULL,
                  has_center ? &cz : NULL);

    return janet_wrap_abstract(shape_data);
}

JANET_FN(cad_make_sphere,
         "(make-sphere radius &keys :center)",
         "Create a sphere with the given radius, centered at the origin.\n\n"
         "Optional :center keyword provides a tuple '(cx cy cz) to "
         "reposition the center of the sphere.\n\n"
         "Returns a rojcad/shape abstract value.")
{
    if (argc < 1) {
        janet_panicf("make-sphere expects at least 1 argument, got %d", argc);
    }
    if (!janet_checktype(argv[0], JANET_NUMBER)) {
        janet_panic("make-sphere: radius must be a number");
    }

    double radius = janet_unwrap_number(argv[0]);

    double cx = 0, cy = 0, cz = 0;
    int has_center = parse_center_keyword(argv, argc, 1, &cx, &cy, &cz);

    void *shape_data = janet_abstract(&rojcad_shape_type, rust_shape_data_size());
    if (!shape_data) {
        janet_panic("failed to allocate shape");
    }
    rust_init_sphere(shape_data, radius,
                     has_center ? &cx : NULL,
                     has_center ? &cy : NULL,
                     has_center ? &cz : NULL);

    return janet_wrap_abstract(shape_data);
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
        /* Invoke the callback if one is registered and there's an event.
         * Since poll returns 0 after consuming, we only reach this when
         * there's genuinely no event. */
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
        /* Query not supported via simple C API; return a neutral value */
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
        {"make-box",         cad_make_box,         cad_make_box_docstring_},
        {"make-sphere",      cad_make_sphere,      cad_make_sphere_docstring_},
        {"cut",              cad_cut,              cad_cut_docstring_},
        {"common",           cad_common,           cad_common_docstring_},
        {"shape-type",       cad_shape_type,       cad_shape_type_docstring_},
        {"hide",             cad_hide,             cad_hide_docstring_},
        {"show",             cad_show,             cad_show_docstring_},
        {"visible?",         cad_visible_q,        cad_visible_q_docstring_},
        {"write-step",       cad_write_step,       cad_write_step_docstring_},
        {"write-stl",        cad_write_stl,        cad_write_stl_docstring_},
        {"on-select",                   cad_on_select,                   cad_on_select_docstring_},
        {"poll-selection",              cad_poll_selection,              cad_poll_selection_docstring_},
        {"edge-toggle-inactive",        cad_edge_toggle_inactive,        cad_edge_toggle_inactive_docstring_},
        {"edge-toggle-active",          cad_edge_toggle_active,          cad_edge_toggle_active_docstring_},
        {"edge-inactive-show?",         cad_edge_inactive_showing,       cad_edge_inactive_showing_docstring_},
        {"edge-active-show?",           cad_edge_active_showing,         cad_edge_active_showing_docstring_},
        {"edge-thickness",              cad_edge_thickness,              cad_edge_thickness_docstring_},
        {"edge-color-inactive",         cad_edge_color_inactive,         cad_edge_color_inactive_docstring_},
        {"edge-color-active",           cad_edge_color_active,           cad_edge_color_active_docstring_},
        {NULL, NULL, NULL}
    };

    janet_cfuns(env, NULL, cfuns);
}
