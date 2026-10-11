//! 锁屏壁纸工具：双击运行，预览后设置锁屏壁纸。
//!
//! 流程：检测管理员权限（无则提权重启）→ 预览窗口 → 确认后设为锁屏。
//!
//! 为什么需要预览？因为设置锁屏后无法直接查看效果，需先预览确认满意。

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::fs;
use wallpaper_rust::{
    download_to_temp, is_admin, relaunch_as_admin, set_lock_wallpaper, show_preview,
};

fn main() {
    // 1. 管理员权限检测：无则尝试提权重启。无论用户是否同意 UAC，
    //    当前这个未提权的进程都退出（提权后的新进程会继续后续流程）。
    if !is_admin() {
        let _ = relaunch_as_admin();
        return;
    }

    // 2. 预览窗口：内部下载并显示，用户确认后返回图片路径。
    //    图片获取逻辑由本入口注入，预览窗口本身不依赖下载编排。
    let image_path = match show_preview(download_to_temp) {
        Some(p) => p,
        None => return, // 用户取消
    };

    // 3. 设为锁屏壁纸
    let _ = set_lock_wallpaper(&image_path);

    // 4. 清理临时文件
    let _ = fs::remove_file(&image_path);
}
