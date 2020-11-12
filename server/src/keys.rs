use std::{
    convert::AsRef,
    path::Path,
};

pub fn to_key_path<P: AsRef<Path>>(path: P) -> impl AsRef<Path> {
    Path::new(crate::KEY_PATH).join(path)
}
pub fn read_key_file<P: AsRef<Path>>(path: P) -> String {
    let path = to_key_path(path);
    std::fs::read_to_string(path.as_ref())
        .map(|s| s.trim_end_matches("\n").to_string())
        .expect(&format!("Failed to read {}", path.as_ref().display()))
}
