use std::{path::{PathBuf, Path}, env};

fn get_mimalloc_dir() -> PathBuf {
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Path when the working directory is the cargo directory: <root or manifest path>/crates/onca_ral_dx12
    let path = Path::new(&manifest_dir_string);

    return PathBuf::from(path.parent().unwrap().parent().unwrap());
}


fn main() {
    let mut mimalloc_dir = get_mimalloc_dir();
    mimalloc_dir.push("external");
    mimalloc_dir.push("mimalloc");

    

    let mut cc = cc::Build::new();

    let mut mimalloc_include_dir = mimalloc_dir.clone();
    mimalloc_include_dir.push("include");
    println!("cargo:warning=mimalloc include path={}", mimalloc_include_dir.to_str().unwrap());
    cc.include(&mimalloc_include_dir);
    
    let mut mimalloc_src_dir = mimalloc_dir.clone();
    mimalloc_src_dir.push("src");
    println!("cargo:warning=mimalloc src path={}", mimalloc_src_dir.to_str().unwrap());
    cc.include(&mimalloc_src_dir);

    mimalloc_src_dir.push("static.c");
    cc.file(&mimalloc_src_dir);

    cc.define("MI_BUILD_SHARED", "1");

    cc.compile("mimalloc");

    println!("cargo:warning=mimalloc built sucessfully");
    

    // link to mimalloc-redirect.lib
    
    // mimalloc_dir.push("bin");

    // let lib_search_path_str = mimalloc_dir.to_str().unwrap();
    // let lib_path_str = mimalloc_dir.to_str().unwrap();
    // println!("cargo:warning=mimalloc search lib path={lib_search_path_str}");
    // println!("cargo:warning=mimalloc lib path={lib_path_str}");
    // println!("cargo:warning=mimalloc lib path exists? {}", Path::new(lib_path_str).exists());

    // println!("cargo:rustc-link-search=native={lib_search_path_str}");
    // println!("cargo:rustc-link-lib=dylib=mimalloc-redirect");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../external/mimalloc");
}