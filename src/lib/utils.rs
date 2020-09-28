// pub fn slowlog<B, F: FnOnce() -> B>(ms_threshold: u32, description: &str, f: F) -> B {
//     let start_time = std::time::Instant::now();
//     let result = f();
//     let elapsed_ms = start_time.elapsed().subsec_millis();
//     if elapsed_ms > ms_threshold {
//         println!("Slow operation, {:5}ms: {}", elapsed_ms, description);
//     }
//     return result;
// }

const MATCH_EXTENSIONS: &'static [&'static str] = &["jpg", "JPG", "png", "PNG"];

fn str_matches_ext(a: &std::ffi::OsStr) -> bool {
    MATCH_EXTENSIONS
        .iter()
        .any(|b| a == std::ffi::OsStr::new(b))
}

pub fn path_matches_ext<P: AsRef<std::path::Path>>(path: &P) -> bool {
    path.as_ref()
        .extension()
        .map(str_matches_ext)
        .unwrap_or(false)
}
