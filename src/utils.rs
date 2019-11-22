// pub fn slowlog<B, F: FnOnce() -> B>(ms_threshold: u32, description: &str, f: F) -> B {
//     let start_time = std::time::Instant::now();
//     let result = f();
//     let elapsed_ms = start_time.elapsed().subsec_millis();
//     if elapsed_ms > ms_threshold {
//         println!("Slow operation, {:5}ms: {}", elapsed_ms, description);
//     }
//     return result;
// }
