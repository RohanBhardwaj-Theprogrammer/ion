use std::fs;

pub fn file_exists(file_path: &str) -> bool {
    fs::metadata(file_path).is_ok()
}


pub fn bin_name(file_path: &str) -> String {
    let bin_name = if let Some(pos) = file_path.rfind('.') {
        file_path[..pos].to_string()
    } else {
        file_path.to_string()
    };
    format!("{}.exe", bin_name)
}

pub fn binary_exists(bin_path: &str) -> bool {
    fs::metadata(bin_path).is_ok()
}

pub fn remove_binary(file_path: &str) -> bool{
    let bin_path = bin_name(file_path);
    if binary_exists(&bin_path) {
        fs::remove_file(&bin_path).expect("Failed to remove existing binary");
        true
    } else {
        false
    }
}
