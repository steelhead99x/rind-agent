fn main() {
    let lib = pkg_config::Config::new()
        .atleast_version("2.46")
        .probe("librsvg-2.0")
        .expect("librsvg-2.0 (with cairo) is required to build gen-frames");
    for include in &lib.include_paths {
        println!("cargo:rerun-if-changed={}", include.display());
    }
}
