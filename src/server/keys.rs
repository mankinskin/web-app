use std::{
    convert::AsRef,
    path::Path,
};

pub fn read_key_file<P: AsRef<Path>>(path: P) -> String {
    std::fs::read_to_string(path.as_ref())
        .map(|s| s.trim_end_matches("\n").to_string())
        .expect(&format!("Failed to read {}", path.as_ref().display()))
}
