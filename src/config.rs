//! 路径配置（纯逻辑，可测试）。

/// 应用名（用于目录、临时文件前缀）。
pub const APP_NAME: &str = "WallpaperRust";

/// 临时文件前缀。
pub const TEMP_PREFIX: &str = "temp_wallpaper_";

/// 给定临时目录根，返回应用临时目录路径。
pub fn temp_dir(root: &str) -> String {
    format!("{}\\{}", root, APP_NAME)
}

/// 给定临时目录和时间戳，返回临时下载文件路径。
pub fn temp_download_path(temp_root: &str, timestamp_secs: u64) -> String {
    format!(
        "{}\\{}{}.jpg",
        temp_dir(temp_root),
        TEMP_PREFIX,
        timestamp_secs
    )
}

/// 判断某个文件名是否是本应用的临时下载文件。
pub fn is_temp_file(filename: &str) -> bool {
    filename.starts_with(TEMP_PREFIX) && filename.ends_with(".jpg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dir_appends_app_name() {
        assert_eq!(temp_dir("C:\\Temp"), "C:\\Temp\\WallpaperRust");
    }

    #[test]
    fn temp_download_path_has_timestamp() {
        assert_eq!(
            temp_download_path("C:\\Temp", 12345),
            "C:\\Temp\\WallpaperRust\\temp_wallpaper_12345.jpg"
        );
    }

    #[test]
    fn is_temp_file_recognizes_own_files() {
        assert!(is_temp_file("temp_wallpaper_123.jpg"));
        assert!(!is_temp_file("other.jpg"));
    }
}
