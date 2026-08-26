use std::error::Error;

#[cfg(target_os = "windows")]
pub fn get_minecraft_dir() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let appdata = dirs::data_dir().ok_or("appdata doesnt exist")?;
    Ok(appdata.join(".minecraft"))
}

#[cfg(target_os = "linux")]
pub fn get_minecraft_dir() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("homedir doesnt exist")?;
    Ok(home.join(".minecraft"))
}
