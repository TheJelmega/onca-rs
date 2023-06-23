use core::str::FromStr;
use std::{env, process::Command, path::{PathBuf, Path}, fs};

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

fn _compile_shader(file: &str, output_dir: &Path, entry_point: &str, target_profile: &str, to_spirv: bool) -> bool {
    let mut command = Command::new("dxc");
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

fn compile_shader(file: &str, output_dir: &Path, entry_point: &str, shader_type: ShaderType) {
    let target_profile = match shader_type {
        ShaderType::Vertex => "vs_6_7",
        ShaderType::Pixel => "ps_6_7",
    };

    println!("Compiling: {file}");

    _compile_shader(file, output_dir, entry_point, target_profile, false);
    _compile_shader(file, output_dir, entry_point, target_profile, true);
}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
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
    copy_and_write_rerun("ral.toml", &profile);
    copy_and_write_rerun("D3D12", &profile);

    let mut shader_output_dir = get_output_path();
    shader_output_dir.push(Path::new("shaders"));
    println!("CARGO_TARGET_DIR: {}", shader_output_dir.to_str().unwrap());
    


    println!("Compiling shaders");
    _ = fs::create_dir(&shader_output_dir);
    compile_shader("data/shaders/tri.vs.hlsl", &shader_output_dir, "main", ShaderType::Vertex);
    compile_shader("data/shaders/tri.ps.hlsl", &shader_output_dir, "main", ShaderType::Pixel);

    #[cfg(windows)]
    println!("cargo:rustc-link-arg=/DEF:D3D12\\agility.def");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/shaders/");
}