# Fix: C++ exceptions in BRepFilletAPI cause std::terminate

## Problem

`BRepFilletAPI_MakeFillet::Shape()` and `BRepFilletAPI_MakeChamfer::Shape()` throw
`StdFail_NotDone` when the operation fails (e.g., fillet radius too large relative to
edge length). In some cases `Add()` can also throw.

Because CXX 1.0.x generates `extern "C"` wrappers with `noexcept` (C++17 default for
`extern "C"` functions), any uncaught C++ exception immediately calls `std::terminate`.
There is no way for Rust callers to recover.

## Root Cause

The current `fillet_edges()` and `chamfer_edges()` methods call `.Shape()` directly
without checking `IsDone()` first, and `Add()` is called without exception protection:

`crates/opencascade/src/primitives/shape.rs:469-482`:
```rust
pub fn fillet_edges<T: AsRef<Edge>>(...) -> Self {
    let mut make_fillet = ffi::b_rep_fillet_api::BRepFilletAPI_MakeFillet_new(&self.inner);
    for edge in edges.into_iter() {
        make_fillet.pin_mut().add_edge(radius, &edge.as_ref().inner);
    }
    Self::from_shape(make_fillet.pin_mut().Shape())  // throws → terminate
}
```

The same pattern exists in `chamfer_edges()` and `variable_fillet_edges()`.

## Fix

Each method needs to catch `Standard_Failure` (OCCT's base exception type) before the
exception reaches the `noexcept` `extern "C"` wrapper. The approach we used in rojcad:

1. Add helper functions in `b_rep_fillet_api.hxx` that wrap `Add()` and `Shape()` with
   `try/catch`:

```cpp
#include <Standard_Failure.hxx>

inline bool rojcad_MakeFillet_TryAdd(
    BRepFilletAPI_MakeFillet &maker,
    double radius,
    const TopoDS_Edge &edge
) {
    try { maker.Add(radius, edge); return true; }
    catch (Standard_Failure const &) { return false; }
}

inline std::unique_ptr<TopoDS_Shape> rojcad_MakeFillet_TryShape(
    BRepFilletAPI_MakeFillet &maker
) {
    try {
        return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(maker.Shape()));
    } catch (Standard_Failure const &) {
        return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape); // null shape
    }
}
```

And analogous functions for `BRepFilletAPI_MakeChamfer`.

2. Declare these as free functions (not methods) in `b_rep_fillet_api.rs` via
   `#[cxx::bridge]`. Use a non-`self` parameter name so CXX generates a free-function
   call:

```rust
#[rust_name = "try_add_edge"]
pub fn rojcad_MakeFillet_TryAdd(
    s: Pin<&mut BRepFilletAPI_MakeFillet>,
    radius: f64,
    edge: &TopoDS_Edge,
) -> bool;
```

3. Use the helpers in `shape.rs`:

```rust
pub fn fillet_edges<T: AsRef<Edge>>(...) -> Self {
    let mut make_fillet = ...;
    for edge in edges.into_iter() {
        if !ffi::b_rep_fillet_api::try_add_edge(
            make_fillet.pin_mut(),
            radius,
            &edge.as_ref().inner,
        ) {
            return Self::empty();  // edge couldn't be added, fail gracefully
        }
    }
    let result = ffi::b_rep_fillet_api::try_shape(make_fillet.pin_mut());
    if !result.IsNull() {
        Self { inner: result }
    } else {
        Self::empty()  // shape construction failed, fail gracefully
    }
}
```

## Alternative Approaches That Were Considered

- **Call `Build()` then `IsDone()` before `Shape()`**: OCCT best practice says to call
  `Build()` explicitly then check `IsDone()`. However, `Add()` itself can also throw
  `StdFail_NotDone` for some edge+parameter combinations, so `IsDone()` alone is
  insufficient — `Add()` needs protection too.

- **Using `Result<>` return types in CXX**: CXX supports `Result<T>` for some return
  types, which generates proper try/catch in the bridge. However, our functions return
  references and `UniquePtr`, which don't easily map to `Result<&T>`.

- **Catching in Rust with `catch_unwind`**: Doesn't work — Rust panic handlers do not
  catch C++ exceptions across the FFI boundary.

## Scope

All three methods need the fix:
- `fillet_edges`
- `chamfer_edges`
- `variable_fillet_edges`

The `ChamferMaker` struct's `build()` method has the same issue.

The `catch(...)` is slightly broader than necessary (`Standard_Failure` would be more
precise) but is acceptable for safety.
