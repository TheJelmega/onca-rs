use core::str::FromStr;
use std::{env, process::Command, path::{PathBuf, Path}, fs, ffi::OsStr};

use embed_manifest::{
    manifest::{ActiveCodePage, SupportedOS::{Windows10}, Setting, DpiAwareness},
    embed_manifest, new_manifest,
};
use copy_to_output::copy_to_output;

enum ShaderType {
    Vertex,
    Pixel
}

fn copy_and_write_rerun(file: &str, profile: &str) {
    copy_to_output(file, &profile).expect("Failed to copy '{file}'");
    println!("cargo:rerun-if-changed={file}");
}

fn get_dxc_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let (os_dir, exec) = ("windows", "dxc.exe");
    #[cfg(target_os = "linux")]
    let (os_dir, exec) = ("linux", "dxc");

    let mut buf = PathBuf::new();
    buf.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    buf.push("dxc");
    buf.push(os_dir);
    buf.push("dxc_2023_03_01");
    buf.push("bin");
    buf.push("x64");
    buf.push(exec);
    buf
}

fn _compile_shader(dxc: &Path, file: &str, output_dir: &Path, entry_point: &str, target_profile: &str, to_spirv: bool) -> bool {
    let mut command = Command::new(dxc);
    command.args(["-E", entry_point])
        .args(["-HV", "2021"])
        .args(["-T", target_profile])
        .arg("-WX");
        
    let mut out_binary = output_dir.to_path_buf();

    if to_spirv {
        out_binary.push("spirv");
    } else {
        out_binary.push("dxil");
    }

    _ = fs::create_dir(&out_binary);

    let file_name_start = file.rfind("/").map_or(0, |idx| idx + 1);
    out_binary.push(Path::new(&file[file_name_start..]));
    
    if to_spirv {
        out_binary.set_extension("spirv");
        command.arg("-spirv");
        if target_profile.contains("vs") {
            command.arg("-fvk-invert-y");
        }
    } else {
        out_binary.set_extension("dxil");
    }
    
    let out_str = out_binary.to_str().unwrap();
    command.args(["-Fo", out_str]);
    command.arg(file);

    let output = command.output().expect("Failed to run dxc");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("{stdout}");

    for line in stderr.lines() {
        println!("cargo:warning={line}");
    }

    !stderr.is_empty()
}

fn compile_shader(dxc_path: &Path, file: &str, output_dir: &Path, entry_point: &str, shader_type: ShaderType) {
    let target_profile = match shader_type {
        ShaderType::Vertex => "vs_6_7",
        ShaderType::Pixel => "ps_6_7",
    };

    println!("cargo:warning=Compiling: {file}");

    _compile_shader(dxc_path, file, output_dir, entry_point, target_profile, false);
    _compile_shader(dxc_path, file, output_dir, entry_point, target_profile, true);
}

fn get_output_path() -> PathBuf {
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Path when working directory is executable folder <root or manifest path>/target/<profile>/
    //let build_type = env::var("PROFILE").unwrap();
    //let path = Path::new(&manifest_dir_string).join("target").join(build_type);

    // Path when the working directory is the cargo directory: <root or manifest path>/
    let path = Path::new(&manifest_dir_string);

    return PathBuf::from(path);
}

fn main() {
    // Windows embeded manifest
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("onca.exe.manifest")
            // Remove defaults we don't care about
            .remove_dependency("Microsoft.Windows.Common-Controls")
            .remove_max_version_tested()
            // Set what we care about
            .active_code_page(ActiveCodePage::Utf8)
            .supported_os(Windows10..=Windows10) // Also includes Windows 11
            .long_path_aware(Setting::Enabled)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
        )
        .expect("unable to embed manifest file");
    }
    
    // Settings files copy
    let profile = env::var("PROFILE").unwrap();
    copy_and_write_rerun("D3D12", &profile);

    // Only needed when the working directory is not the cargo root
    //copy_and_write_rerun("ral.toml", &profile);

    let mut shader_output_dir = get_output_path();
    shader_output_dir.push(Path::new("data"));
    shader_output_dir.push(Path::new("shaders"));
    println!("cargo:warning=CARGO_TARGET_DIR: {}", shader_output_dir.to_str().unwrap());
    


    println!("cargo:warning=Compiling shaders");
    _ = fs::create_dir_all(&shader_output_dir);

    let dxc_path = get_dxc_path();
    println!("cargo:warning=DXC is located at: {}", dxc_path.to_str().unwrap());

    compile_shader(&dxc_path, "data/shaders/tri.vs.hlsl", &shader_output_dir, "main", ShaderType::Vertex);
    compile_shader(&dxc_path, "data/shaders/tri.ps.hlsl", &shader_output_dir, "main", ShaderType::Pixel);

    #[cfg(windows)]
    println!("cargo:rustc-link-arg=/DEF:D3D12\\agility.def");

    // Link onca_alloc dylib
    // TODO: filename is only for windows atm
    println!("cargo:rustc-link-lib=onca_malloc{dylib_ext}");

    // Setup rerun
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/shaders/");
}