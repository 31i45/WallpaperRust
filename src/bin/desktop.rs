//! 桌面壁纸工具：双击运行，更换桌面壁纸后立即退出。
//!
//! 设计理念：一次性工具，不常驻后台。用户想换的时候双击即可。

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::fs;
use std::path::Path;
use wallpaper_rust::{config, download_to_temp, set_desktop_wallpaper};

fn main() {
    // 下载壁纸到临时文件（内部含崩溃残留垃圾回收）
    let temp_path = match download_to_temp() {
        Some(p) => p,
        None => return, // 所有图源失败，静默退出
    };

    // SystemParametersInfoW 只登记路径、不复制文件：若壁纸指向 %TEMP% 的文件，
    // 它被删除后 Shell 下次刷新壁纸（唤醒/重启 explorer/分辨率变化）会读不到
    // 而回退纯黑。因此先落一份持久副本（%APPDATA%，固定名覆盖写）再设壁纸。
    let dest = std::env::var("APPDATA")
        .ok()
        .map(|root| config::wallpaper_dest(&root))
        .filter(|d| {
            fs::create_dir_all(Path::new(d).parent().unwrap_or(Path::new("."))).is_ok()
                && fs::copy(&temp_path, d).is_ok()
        });
    let set_path = dest.unwrap_or_else(|| temp_path.clone());

    // 设为桌面壁纸；仅当壁纸已指向持久副本时才删除临时文件
    let _ = set_desktop_wallpaper(&set_path);
    if set_path != temp_path {
        let _ = fs::remove_file(&temp_path);
    }
}
