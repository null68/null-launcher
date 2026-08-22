#[cfg(target_os = "windows")]
pub fn find_java() -> Option<String> {
    use std::process::Command;

    let output = Command::new("cmd")
        .args(["/C", "where java"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();

    Some(path)
}

#[cfg(target_os = "linux")]
pub fn find_java() -> Option<String> {
    use std::process::Command;

    let output = Command::new("sh")
        .args(["/C", "which java"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();

    Some(path)
}
