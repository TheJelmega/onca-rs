/// Get the number of kilobytes in bytes
#[allow(non_snake_case)]
pub const fn KB(val: usize) -> usize {
    val * 1000
}

/// Get the number of megabytes in bytes
#[allow(non_snake_case)]
pub const fn MB(val: usize) -> usize {
    val * 1000 * 1000
}

/// Get the number of gigabytes in bytes
#[allow(non_snake_case)]
pub const fn GB(val: usize) -> usize {
    val * 1000 * 1000 * 1000
}

/// Get the number of kibibytes in bytes
#[allow(non_snake_case)]
pub const fn KiB(val: usize) -> usize {
    val * 1024
}

/// Get the number of mibibytes in bytes
#[allow(non_snake_case)]
pub const fn MiB(val: usize) -> usize {
    val * 1024 * 1024
}

/// Get the number of gibibytes in bytes
#[allow(non_snake_case)]
pub const fn GiB(val: usize) -> usize {
    val * 1024 * 1024 * 1024
}