




fn version_commands() -> Result<String,String> {
    println!(" {} c/cpp build system and package manager ,\n version: {}",crate::constants::PROGRAM_NAME,crate::constants::PROGRAM_VERSION);
    Ok("".to_string())
}