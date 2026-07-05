#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
  if let Ok(source) = std::str::from_utf8(data) {
    let source_type = oxc::span::SourceType::ts();
    let _ = pulsar_frontend_oxc::extract(source, source_type, "fuzz.ts");
  }
});
