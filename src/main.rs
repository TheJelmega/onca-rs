

fn main() {
    onca_core::sys::ensure_utf8().unwrap_or_else(|err_code|
        panic!("Failed to ensure the app is using UTF-8, this might happen because of an incorrect .manifest file. (err code: {})", err_code)
    );
}
