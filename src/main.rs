

fn main() {
    onca_core::os::ensure_utf8().unwrap_or_else(|err_code|
        panic!("Failed to ensure the OS is using UTF-8, this might happen because of an incorrect .manifest file or forgetting to run `onca_post_build` after building onca. (err code: {})", err_code)
    );
}
