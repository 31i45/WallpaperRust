//! 桌面壁纸工具：双击运行，更换桌面壁纸后立即退出。
//!
//! 设计理念：一次性工具，不常驻后台。用户想换的时候双击即可。

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::fs;
use wallpaper_rust::{download_to_temp, set_desktop_wallpaper};

fn main() {
    // 下载壁纸到临时文件（内部含崩溃残留垃圾回收）
    let temp_path = match download_to_temp() {
        Some(p) => p,
        None => return, // 所有图源失败，静默退出
    };

    // 设为桌面壁纸，无论成败都清理临时文件
    let _ = set_desktop_wallpaper(&temp_path);
    let _ = fs::remove_file(&temp_path);
}
