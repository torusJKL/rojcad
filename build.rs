fn main() {
    // Compile Janet amalgamation
    cc::Build::new()
        .file("vendor/janet.c")
        .include("vendor")
        .flag("-DJANET_NO_DYNAMIC_MODULES=1")
        .flag("-DJANET_REDUCE_JIT_MEMORY=1")
        // Disable source maps (simpler, avoids __FILE__ dependency)
        .flag("-DJANET_NO_SOURCEMAPS=1")
        .opt_level(2)
        .compile("janet");

    // Compile the C bridge layer
    cc::Build::new()
        .file("bridge/bridge.c")
        .include("vendor")
        .flag("-DJANET_NO_DYNAMIC_MODULES=1")
        .flag("-DJANET_NO_SOURCEMAPS=1")
        .opt_level(2)
        .compile("rojcad_bridge");

    // Link OCCT libraries from opencascade-rs
    // opencascade-rs provides the OCCT libs via its own build.rs
    // We need to link them here as well for our final binary
    println!("cargo:rustc-link-lib=static=janet");
    println!("cargo:rustc-link-lib=static=rojcad_bridge");

    // Rerun if vendor or bridge files change
    println!("cargo:rerun-if-changed=vendor/janet.c");
    println!("cargo:rerun-if-changed=vendor/janet.h");
    println!("cargo:rerun-if-changed=bridge/bridge.c");
}
