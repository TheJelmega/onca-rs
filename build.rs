use embed_manifest::{
    manifest::{ActiveCodePage, SupportedOS::{Windows10}, Setting, DpiAwareness},
    embed_manifest, new_manifest,
};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
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
    println!("cargo:rerun-if-changed=build.rs");
}