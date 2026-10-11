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

/// 残留临时文件视为过期的阈值（秒）。超过 1 小时的临时文件只可能来自
/// 崩溃/被杀的旧进程（正常进程在数秒内退出并自行删除）。
pub const STALE_TEMP_SECS: u64 = 3600;

/// 判断一个临时文件（修改时间 modified_secs）在 now_secs 时刻是否已过期。
///
/// 时钟跳变（now < modified）时按未过期处理，避免误删并发实例的新文件。
pub fn is_stale(modified_secs: u64, now_secs: u64, max_age_secs: u64) -> bool {
    now_secs > modified_secs && now_secs - modified_secs >= max_age_secs
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

    #[test]
    fn is_stale_boundary_and_clock_skew() {
        // 刚好达到阈值 → 过期
        assert!(is_stale(1000, 4600, 3600));
        // 差 1 秒 → 未过期
        assert!(!is_stale(1000, 4599, 3600));
        // 时钟回拨（now < modified）→ 未过期，不误删
        assert!(!is_stale(1000, 500, 3600));
        // 刚创建的文件 → 未过期
        assert!(!is_stale(1000, 1000, 3600));
    }
}
