fn main() {
    // Compile individual Janet core source files
    let mut janet_build = cc::Build::new();
    janet_build
        .include("vendor")
        .include("vendor/core")
        // Build core env from scratch instead of loading pre-compiled boot image
        .define("JANET_BOOTSTRAP", Some("1"))
        .opt_level(2);

    for entry in std::fs::read_dir("vendor/core").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "c") {
            janet_build.file(&path);
        }
    }

    janet_build.compile("janet");

    // Compile the C bridge layer
    cc::Build::new()
        .file("bridge/bridge.c")
        .include("vendor")
        .include("vendor/core")
        .define("JANET_BOOTSTRAP", Some("1"))
        .opt_level(2)
        .compile("rojcad_bridge");

    println!("cargo:rustc-link-lib=static=janet");
    println!("cargo:rustc-link-lib=static=rojcad_bridge");

    // Rerun if vendor or bridge files change
    println!("cargo:rerun-if-changed=vendor/core/");
    println!("cargo:rerun-if-changed=vendor/janet.h");
    println!("cargo:rerun-if-changed=vendor/janetconf.h");
    println!("cargo:rerun-if-changed=bridge/bridge.c");
    println!("cargo:rerun-if-changed=boot.janet");
}
