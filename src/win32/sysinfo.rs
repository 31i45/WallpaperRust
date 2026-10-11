//! 系统信息：屏幕分辨率、临时目录、管理员权限检测、当前可执行文件路径。

use super::*;

/// 当前主显示器分辨率（宽，高）。
pub fn screen_resolution() -> (u32, u32) {
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        (
            if w > 0 { w as u32 } else { 1920 },
            if h > 0 { h as u32 } else { 1080 },
        )
    }
}

/// 系统临时目录路径。
pub fn temp_path_root() -> Option<String> {
    unsafe {
        let mut buf = [0u16; MAX_PATH];
        let len = GetTempPathW(buf.len() as DWORD, buf.as_mut_ptr());
        if len == 0 || len as usize > buf.len() {
            return None;
        }
        Some(
            String::from_utf16_lossy(&buf[..len as usize])
                .trim_end_matches('\\')
                .to_string(),
        )
    }
}

/// 当前可执行文件的完整路径。
pub fn current_exe_path() -> Option<String> {
    unsafe {
        let mut buf = [0u16; MAX_PATH];
        let len = GetModuleFileNameW(core::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as DWORD);
        if len == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// 检测当前进程是否以管理员权限运行。
pub fn is_admin() -> bool {
    unsafe {
        let mut token: HANDLE = core::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation: TOKEN_ELEVATION = core::mem::zeroed();
        let mut ret_len: DWORD = 0;
        let ok = GetTokenInformation(
            token,
            20, // TokenElevation
            &mut elevation as *mut _ as LPVOID,
            core::mem::size_of::<TOKEN_ELEVATION>() as DWORD,
            &mut ret_len,
        ) != 0;
        CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// 以管理员权限重新启动自身，当前进程退出。
/// 触发 UAC 弹窗。
pub fn relaunch_as_admin() -> bool {
    let exe = match current_exe_path() {
        Some(p) => p,
        None => return false,
    };
    unsafe {
        let verb: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();
        let file: Vec<u16> = exe.encode_utf16().chain(Some(0)).collect();
        let result = ShellExecuteW(
            core::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            core::ptr::null(),
            core::ptr::null(),
            SW_SHOWNORMAL,
        );
        // ShellExecuteW 返回值 > 32 表示成功
        result as usize > 32
    }
}

#[repr(C)]
struct TOKEN_ELEVATION {
    TokenIsElevated: DWORD,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_resolution_positive() {
        let (w, h) = screen_resolution();
        assert!(w >= 1);
        assert!(h >= 1);
    }

    #[test]
    fn temp_path_root_nonempty() {
        let p = temp_path_root();
        assert!(p.is_some());
        assert!(!p.unwrap().is_empty());
    }

    #[test]
    fn current_exe_path_ends_with_exe() {
        let p = current_exe_path();
        assert!(p.is_some());
        assert!(p.unwrap().to_lowercase().ends_with(".exe"));
    }

    // is_admin 的结果取决于运行环境，不做断言
}
