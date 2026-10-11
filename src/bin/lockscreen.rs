//! 锁屏壁纸工具：双击运行，预览后设置锁屏壁纸。
//!
//! 流程：检测管理员权限（无则提权重启）→ 预览窗口（内部下载+预览）→ 确认后设为锁屏。
//!
//! 为什么需要预览？因为设置锁屏后无法直接查看效果，需先预览确认满意。

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::fs;
use wallpaper_rust::set_lock_wallpaper;
use wallpaper_rust::win32::preview::show_preview;
use wallpaper_rust::win32::sysinfo::{is_admin, relaunch_as_admin};

fn main() {
    // 1. 管理员权限检测：无则提权重启，当前进程退出
    if !is_admin() {
        if relaunch_as_admin() {
            return; // 提权启动成功，当前进程退出
        }
        // 提权失败（用户取消 UAC），静默退出
        return;
    }

    // 2. 预览窗口：内部下载并显示，用户确认后返回图片路径
    let image_path = match show_preview() {
        Some(p) => p,
        None => return, // 用户取消
    };

    // 3. 设为锁屏壁纸
    let _ = set_lock_wallpaper(&image_path);

    // 4. 清理临时文件
    let _ = fs::remove_file(&image_path);
}
