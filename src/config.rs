//! 应用策略常量与路径配置（纯逻辑，可测试）。

/// 应用名（用于目录、临时文件前缀）。
pub const APP_NAME: &str = "WallpaperRust";

/// 临时文件前缀。
pub const TEMP_PREFIX: &str = "temp_wallpaper_";

/// 下载重试次数（含首次）。
pub const MAX_DOWNLOAD_RETRIES: usize = 3;

/// 两次重试之间的等待秒数。
pub const RETRY_DELAY_SECS: u64 = 1;

/// 给定临时目录根，返回应用临时目录路径。
pub fn temp_dir(root: &str) -> String {
    format!("{}\\{}", root, APP_NAME)
}

/// 给定临时目录、时间戳、进程 ID 与进程内序号，返回临时下载文件路径。
///
/// 同时包含 pid 与单调 seq：仅用秒级时间戳命名时，锁屏"换一张"若在
/// 同一秒内完成下载，新文件会与旧文件同名——旧文件随后被删除会误删新文件。
pub fn temp_download_path(temp_root: &str, timestamp_secs: u64, pid: u32, seq: u64) -> String {
    format!(
        "{}\\{}{}_{}_{}.jpg",
        temp_dir(temp_root),
        TEMP_PREFIX,
        timestamp_secs,
        pid,
        seq
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
    fn temp_download_path_has_timestamp_pid_seq() {
        assert_eq!(
            temp_download_path("C:\\Temp", 12345, 678, 0),
            "C:\\Temp\\WallpaperRust\\temp_wallpaper_12345_678_0.jpg"
        );
    }

    #[test]
    fn temp_download_path_unique_per_seq_even_same_second() {
        // 同秒、同进程，序号不同 → 路径必须不同（"换一张"防误删的关键）
        assert_ne!(
            temp_download_path("C:\\Temp", 100, 1, 0),
            temp_download_path("C:\\Temp", 100, 1, 1)
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
