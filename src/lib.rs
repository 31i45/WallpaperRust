//! WallpaperRust 核心库。
//!
//! 桌面版与锁屏版共享：图源策略、下载、JPEG 校验、壁纸设置。
//! 两个 binary 各自负责交互逻辑（桌面版无界面直接退出，锁屏版带预览窗口）。

pub mod config;
pub mod sources;
pub mod validate;
pub mod win32;

use sources::*;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use validate::is_valid_jpeg;
use win32::http;
use win32::sysinfo;
use win32::wallpaper;

/// 临时文件进程内单调序号，保证同秒内连续下载也不会重名。
static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// 解析图源，返回候选图片 URL 列表（按优先级排序）。
pub(crate) fn resolve_image_urls(source: &Source, width: u32, height: u32) -> Vec<String> {
    match source.kind {
        SourceKind::Direct => vec![render_url(source.url, width, height)],
        SourceKind::BingDaily => {
            // 请求 n=8 获取 8 张图片，随机选一张增加多样性
            let json = match http::download_text(source.url) {
                Some(t) => t,
                None => return vec![],
            };
            let urlbases = extract_all_bing_urlbases(&json);
            if urlbases.is_empty() {
                return vec![];
            }
            let idx = (fast_rand() as usize) % urlbases.len();
            bing_image_urls(urlbases[idx])
        }
    }
}

/// 遍历所有图源，下载并校验第一张有效 JPEG，返回字节数据。
/// 全部失败返回 None。
pub(crate) fn fetch_wallpaper_bytes(width: u32, height: u32) -> Option<Vec<u8>> {
    let sources = builtin_sources();
    let ordered = shuffle_sources(&sources, fast_rand);

    for source in &ordered {
        let urls = resolve_image_urls(source, width, height);
        for url in &urls {
            for attempt in 0..config::MAX_DOWNLOAD_RETRIES {
                if let Some(data) = http::download(url)
                    && is_valid_jpeg(&data)
                {
                    return Some(data);
                }
                // 仅在两次尝试之间等待，最后一次失败后立即尝试下一个 URL
                if attempt + 1 < config::MAX_DOWNLOAD_RETRIES {
                    thread::sleep(std::time::Duration::from_secs(config::RETRY_DELAY_SECS));
                }
            }
        }
    }
    None
}

/// 清扫应用临时目录中崩溃/被杀进程遗留的过期临时文件（垃圾回收）。
///
/// 只删除本应用命名（temp_wallpaper_*.jpg）且修改时间超过
/// [`config::STALE_TEMP_SECS`] 的文件；新文件（可能属于并发实例）不动。
/// 单次运行只持续数秒且正常退出会自行删除，因此能留下的旧文件必然来自异常退出。
pub(crate) fn cleanup_stale_temp_files() {
    let Some(root) = sysinfo::temp_path_root() else {
        return;
    };
    let dir = config::temp_dir(&root);
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    let now = now_secs();
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !config::is_temp_file(&name) {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let modified_secs = modified
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if config::is_stale(modified_secs, now, config::STALE_TEMP_SECS) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// 下载壁纸并写入临时文件，返回临时文件路径。
///
/// 下载前先执行一次垃圾回收（覆盖桌面、锁屏、换一张三个入口）。
pub fn download_to_temp() -> Option<String> {
    let root = sysinfo::temp_path_root().unwrap_or_else(|| "C:\\Windows\\Temp".to_string());
    let dir = config::temp_dir(&root);
    fs::create_dir_all(&dir).ok()?;
    cleanup_stale_temp_files();

    let (w, h) = sysinfo::screen_resolution();
    let data = fetch_wallpaper_bytes(w, h)?;
    let ts = now_secs();
    // pid + 进程内单调序号：即使同秒内连续下载两次，文件名也不重复
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let path = config::temp_download_path(&root, ts, std::process::id(), seq);
    fs::write(&path, &data).ok()?;
    Some(path)
}

/// 设为桌面壁纸。
pub fn set_desktop_wallpaper(image_path: &str) -> bool {
    wallpaper::set_wallpaper(image_path)
}

/// 设为锁屏壁纸（需管理员权限）。
pub fn set_lock_wallpaper(image_path: &str) -> bool {
    wallpaper::set_lock_screen_wallpaper(image_path)
}

/// 当前进程是否拥有管理员权限。
pub fn is_admin() -> bool {
    sysinfo::is_admin()
}

/// 以管理员身份重新启动自身（触发 UAC）。成功时调用方应立即退出当前进程。
pub fn relaunch_as_admin() -> bool {
    sysinfo::relaunch_as_admin()
}

/// 弹出锁屏壁纸预览窗口。
///
/// `fetcher` 负责提供新图片（首次加载与每次"换一张"都调用它），
/// 返回用户确认的图片路径；关闭窗口则返回 None。
/// 通过注入 fetcher，预览窗口这一机制层不依赖下载编排，可独立测试。
pub fn show_preview<F>(fetcher: F) -> Option<String>
where
    F: FnMut() -> Option<String> + 'static,
{
    win32::preview::show_preview(fetcher)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn fast_rand() -> u64 {
    static SEED: AtomicU64 = AtomicU64::new(0);
    let mut s = SEED.load(Ordering::Relaxed);
    if s == 0 {
        s = now_secs() ^ (std::process::id() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        // xorshift 以 0 为不动点，极端情况下种子恰好为 0 时强制置非零
        if s == 0 {
            s = 0x9E37_79B9_7F4A_7C15;
        }
    }
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    SEED.store(s, Ordering::Relaxed);
    s
}
