use std::{process::Command, io::{self, Write}, env};

fn main() {
    // We don't need to do anything if we're not on windows
    if !cfg!(windows) {
        return;
    }

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        #[cfg(windows)]
        panic!("Invalid argument count, expected 'onca_post_build.exe <path-to-manifest>'");
    }

    let manifest_path = &args[1];
    println!("manifest located at at: {manifest_path}");

    // We expect that the windows kits are in their default directory:
    // C:/Program Files (x86)/Windows Kits/10/bin
    // We then look for the highest version installed to get mt.exe from there
    let res = get_windows_kit_dir();
    let dir = match res {
        Ok(dir) => dir,
        Err(err) => {
            let err_code = err.raw_os_error().unwrap_or_default();
            panic!("Failed to find windows kit path (err: {:X})", err_code);
        },
    };

    println!("found microsoft manifest tool at: {dir}x64\\mt.exe");

    // onca.exe is in the same directory, so get it from there
    let mut onca_exe_path = env::current_exe().unwrap();
    onca_exe_path.set_file_name("onca.exe");
    let onca_exe_path_str = onca_exe_path.to_str().unwrap();

    println!("onca.exe is located in: {:?}", onca_exe_path_str);

    println!("running command line: {dir}x64\\mt.exe -manifest {manifest_path} -outputresource:{onca_exe_path_str}");

    let ouput_resource_arg = "-outputresource:".to_string() + onca_exe_path_str;

    let output = Command::new(dir + r"x64\mt.exe")
        .args(["-manifest", manifest_path, &ouput_resource_arg])
        .output()
        .expect("Failed to include manifest in EXE");
    let _ = std::io::stdout().write(&output.stdout);
}

fn get_windows_kit_dir() -> io::Result<String> {
    let windows_kits_path = r"C:\Program Files (x86)\Windows Kits\10\bin\";

    let mut version_dir_name = String::new();
    for entry in std::fs::read_dir(windows_kits_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(comp) = path.components().last() {
                let sub_path = comp.as_os_str().to_str().unwrap_or("");
                if sub_path.starts_with("10.") {
                    version_dir_name = sub_path.to_string();
                }

            }
        }
    }

    Ok(windows_kits_path.to_string() + &version_dir_name + r"\")
}