#![no_main]

use libfuzzer_sys::fuzz_target;
use pulsar_core::SourceLocation;

fuzz_target!(|data: &[u8]| {
  if let Ok(sql) = std::str::from_utf8(data) {
    let loc = SourceLocation { file: String::new(), line: 1, column: 1, span: None };
    let _ = pulsar_frontend_sql::parse_sql(sql, loc);
  }
});
