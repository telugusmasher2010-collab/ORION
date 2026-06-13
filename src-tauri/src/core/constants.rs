use std::path::PathBuf;

pub fn orion_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("../../")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("C:/ORION"))
}
