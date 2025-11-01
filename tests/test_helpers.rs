use std::fs;
use std::path::PathBuf;

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

pub fn load_fixture(filename: &str) -> String {
    let path = fixture_path(filename);
    fs::read_to_string(path).expect("Failed to read fixture")
}

pub fn fixture_exists(filename: &str) -> bool {
    fixture_path(filename).exists()
}
