

fn main() {
    println!("cargo:rustc-link-lib=onca_malloc.dll");
    println!("cargo:rerun-if-changed=build.rs");
}