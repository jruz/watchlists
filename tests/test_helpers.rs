#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use std::fs;
use std::path::PathBuf;

#[must_use]
pub fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

pub fn save_fixture(filename: &str, content: &str) {
    let path = fixture_path(filename);
    fs::write(path, content).expect("Failed to write fixture");
}

#[must_use]
pub fn load_fixture(filename: &str) -> String {
    let path = fixture_path(filename);
    fs::read_to_string(path).expect("Failed to read fixture")
}

#[must_use]
pub fn fixture_exists(filename: &str) -> bool {
    fixture_path(filename).exists()
}
