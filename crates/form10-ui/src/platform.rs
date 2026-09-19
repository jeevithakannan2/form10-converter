use std::path::Path;

pub fn open_file(path: &Path) -> Result<(), String> {
    opener::open(path).map_err(|error| format!("Could not open the file: {error}"))
}

pub fn reveal_file(path: &Path) -> Result<(), String> {
    opener::reveal(path).map_err(|error| format!("Could not show the file: {error}"))
}
