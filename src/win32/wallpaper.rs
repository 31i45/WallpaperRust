//! 桌面与锁屏壁纸设置。

use super::*;
use std::fs;
use std::path::Path;

/// 锁屏壁纸在 ProgramData 下的固定路径。
fn lock_screen_dest() -> Option<String> {
    let program_data = std::env::var("ProgramData").ok()?;
    Some(format!(
        "{}\\Microsoft\\Windows\\Lockscreen.jpg",
        program_data
    ))
}

/// 将指定图片文件设为桌面壁纸。
///
/// 传 0x3（UPDATEINIFILE | SENDCHANGE）确保 Shell 收到广播。
pub fn set_wallpaper(image_path: &str) -> bool {
    let wide: Vec<u16> = image_path.encode_utf16().chain(Some(0)).collect();
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide.as_ptr() as LPVOID,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        ) != 0
    }
}

/// 将指定图片设为锁屏壁纸。
///
/// 需要管理员权限（写 HKLM\PersonalizationCSP）。无权限时静默返回 false。
/// 参考 Win11-Wallpaper 项目的正确实现。
pub fn set_lock_screen_wallpaper(image_path: &str) -> bool {
    let dest = match lock_screen_dest() {
        Some(d) => d,
        None => return false,
    };

    // 确保目标目录存在并复制图片
    if let Some(parent) = Path::new(&dest).parent()
        && fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    if fs::copy(image_path, &dest).is_err() {
        return false;
    }

    // 写 HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP
    unsafe {
        let subkey: Vec<u16> = r"SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut hkey: HKEY = core::ptr::null_mut();
        let status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_WRITE, &mut hkey);
        if status != ERROR_SUCCESS || hkey.is_null() {
            return false; // 无管理员权限
        }

        let path_wide: Vec<u16> = dest.encode_utf16().chain(Some(0)).collect();
        let name_path: Vec<u16> = "LockScreenImagePath"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let name_status: Vec<u16> = "LockScreenImageStatus"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let status_val: DWORD = 1;

        let ok = RegSetValueExW(
            hkey,
            name_path.as_ptr(),
            0,
            REG_SZ,
            path_wide.as_ptr() as *const u8,
            (path_wide.len() * 2) as DWORD,
        ) == ERROR_SUCCESS
            && RegSetValueExW(
                hkey,
                name_status.as_ptr(),
                0,
                REG_DWORD,
                &status_val as *const _ as *const u8,
                core::mem::size_of::<DWORD>() as DWORD,
            ) == ERROR_SUCCESS;

        RegCloseKey(hkey);
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_screen_dest_under_programdata() {
        let dest = lock_screen_dest();
        assert!(dest.is_some());
        let dest = dest.unwrap();
        assert!(dest.ends_with("\\Microsoft\\Windows\\Lockscreen.jpg"));
        assert!(dest.to_lowercase().contains("programdata"));
    }

    // set_wallpaper / set_lock_screen_wallpaper 的视觉效果无法单元测试，
    // 由端到端验证覆盖。
}
