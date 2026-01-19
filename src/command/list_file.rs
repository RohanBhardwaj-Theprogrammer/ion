

use std::fs;
use std::io;


pub fn list_file_cmd(root_dir: &str) -> io::Result<Vec<String>> {
    let mut file_list: Vec<String> = Vec::new();
    
    for entry in fs::read_dir(root_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    file_list.push(file_name_str.to_string());
                }
            }
        } else if path.is_dir() {
            if let Some(dir_path_str) = path.to_str() {
                let mut sub_files = list_file_cmd(dir_path_str)?;
                let dir_name = path.file_name()
                    .and_then(|os| os.to_str())
                    .unwrap_or("")
                    .to_string();

                sub_files.iter_mut().for_each(|f| {
                    *f = format!("{}\\{}", dir_name, f);
                });

                file_list.extend(sub_files);
            }
        }
    }

    Ok(file_list)
}


pub fn select_files_by_extension(file_list: Vec<String>, extension: Vec<&str>) -> Vec<String> {
    let mut selected_files : Vec<String> = Vec::new();

    for file in file_list {
        if extension.iter().any(|ext| file.ends_with(ext)) {
            selected_files.push(file);
        }
    }

    selected_files
}



pub fn user_file_selection(file_list: Vec<String>) -> Option<String> {
    println!("Select a file from the list below:");
    for (index, file) in file_list.iter().enumerate() {
        println!("{}: {}", index + 1, file);
    }
    println!("Enter the number of the file you want to select:");

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let selection: usize = input.trim().parse().ok()?;

    if selection == 0 || selection > file_list.len() {
        println!("Invalid selection.");
        panic!("User selected an invalid file");
    }

    Some(file_list[selection - 1].clone())

}



pub fn single_file_selection(root_dir: &str, extensions: Vec<&str>) -> io::Result<String> {
    let all_files = list_file_cmd(root_dir)?;
    let filtered_files = select_files_by_extension(all_files, extensions);

    if filtered_files.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No files found with the specified extensions"));
    }

    match user_file_selection(filtered_files) {
        Some(file) => Ok(file),
        None => Err(io::Error::new(io::ErrorKind::InvalidInput, "File selection failed")),
    }
}


#[cfg(test)]
mod tests {
    use std::path;
    use super::*;

    #[test]
    fn test_select_files_by_extension() {
        let files = vec![
            "main.c".to_string(),
            "utils.cpp".to_string(),
            "readme.md".to_string(),
            "script.py".to_string(),
        ];
        let extensions = vec![".c", ".cpp"];
        let selected_files = select_files_by_extension(files, extensions);
        assert_eq!(selected_files, vec!["main.c".to_string(), "utils.cpp".to_string()]);
    }

    #[test]
    fn test_modified_list_file_cmd(){
        let file = list_file_cmd(r"C:\Users\rohan\temp-folder-temp-data-storage-base\03-11-2025");  
        assert!(file.is_ok());
        let file_list = file.unwrap();
        for f in &file_list {
            println!("\n out put file :      {}\n", f);
            let path = format!("C:\\Users\\rohan\\temp-folder-temp-data-storage-base\\03-11-2025\\{}", f) ;
            println!("{} |   |    {} \n ---------------------- \n", f, path::Path::new(&path).exists());
        }
    }
}
