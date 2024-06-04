

fn main() {
    println!("cargo:rustc-link-lib=onca_malloc.dll");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(os_target, values(\"windows\"))");
}