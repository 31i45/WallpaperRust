//! 手写 Win32 FFI 声明，集中于此以便审计。
//! 零第三方依赖：所有系统调用通过 extern "system" 直接绑定。

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)] // Win32 类型名保持 SDK 原样

pub mod com;
pub mod http;
pub mod preview;
pub mod sysinfo;
pub mod wallpaper;

use core::ffi::{c_int, c_long, c_uint, c_ulong, c_ushort, c_void};

pub type BOOL = c_int;
pub type DWORD = c_ulong;
pub type HANDLE = *mut c_void;
pub type HMODULE = HANDLE;
pub type LPVOID = *mut c_void;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;
pub type UINT = c_uint;
pub type HKEY = *mut c_void;
pub type LSTATUS = c_long;
pub type HINSTANCE = HANDLE;
pub type HWND = HANDLE;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type ATOM = u16;
pub type HDC = HANDLE;
pub type HFONT = HANDLE;
pub type HICON = HANDLE;
pub type HCURSOR = HANDLE;
pub type HBRUSH = HANDLE;
pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
pub struct POINT {
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

#[repr(C)]
pub struct RECT {
    pub left: c_long,
    pub top: c_long,
    pub right: c_long,
    pub bottom: c_long,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: c_int,
    pub cbWndExtra: c_int,
    pub hInstance: HMODULE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: HICON,
}

pub const ERROR_SUCCESS: LSTATUS = 0;
pub const SM_CXSCREEN: c_int = 0;
pub const SM_CYSCREEN: c_int = 1;
pub const SPI_SETDESKWALLPAPER: UINT = 0x0014;
pub const SPIF_UPDATEINIFILE: UINT = 0x0001;
pub const SPIF_SENDCHANGE: UINT = 0x0002;
pub const HKEY_LOCAL_MACHINE: HKEY = (-2147483646isize) as HKEY;
pub const KEY_WRITE: DWORD = 0x20006;
pub const REG_SZ: DWORD = 1;
pub const REG_DWORD: DWORD = 4;
pub const MAX_PATH: usize = 260;

pub const TOKEN_QUERY: DWORD = 0x0008;
pub const SW_SHOWNORMAL: c_int = 1;

pub const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: DWORD = 0;
pub const WINHTTP_FLAG_SECURE: DWORD = 0x0080_0000;
pub const WINHTTP_QUERY_STATUS_CODE: DWORD = 19;
pub const WINHTTP_QUERY_FLAG_NUMBER: DWORD = 0x2000_0000;
pub const WINHTTP_OPTION_REDIRECT_POLICY: DWORD = 89;
pub const WINHTTP_OPTION_REDIRECT_POLICY_ALWAYS: DWORD = 2;
pub const WINHTTP_NO_ADDITIONAL_HEADERS: LPCWSTR = core::ptr::null();
pub const WINHTTP_NO_REQUEST_DATA: LPVOID = core::ptr::null_mut();

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetSystemMetrics(nIndex: c_int) -> c_int;
    pub fn GetTempPathW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetModuleFileNameW(hModule: HMODULE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn SystemParametersInfoW(
        uiAction: UINT,
        uiParam: UINT,
        pvParam: LPVOID,
        fWinIni: UINT,
    ) -> BOOL;
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    pub fn LoadCursorW(hInstance: HMODULE, lpCursorName: LPCWSTR) -> HCURSOR;
    pub fn LoadIconW(hInstance: HMODULE, lpIconName: LPCWSTR) -> HICON;
    pub fn RegisterClassExW(lpWndClassEx: *const WNDCLASSEXW) -> ATOM;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        x: c_int,
        y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        hWndParent: HWND,
        hMenu: HANDLE,
        hInstance: HMODULE,
        lpParam: LPVOID,
    ) -> HWND;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
    ) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn PostQuitMessage(nExitCode: c_int);
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn MoveWindow(
        hWnd: HWND,
        x: c_int,
        y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        bRepaint: BOOL,
    ) -> BOOL;
    pub fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
    pub fn DrawTextW(
        hDC: HDC,
        lpchText: LPCWSTR,
        cchText: c_int,
        lprc: *mut RECT,
        format: UINT,
    ) -> c_int;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: c_int) -> HWND;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetProcessDPIAware() -> BOOL;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: DWORD,
        phkResult: *mut HKEY,
    ) -> LSTATUS;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: DWORD,
        dwType: DWORD,
        lpData: *const u8,
        cbData: DWORD,
    ) -> LSTATUS;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn OpenProcessToken(
        ProcessHandle: HANDLE,
        DesiredAccess: DWORD,
        TokenHandle: *mut HANDLE,
    ) -> BOOL;
    pub fn GetTokenInformation(
        TokenHandle: HANDLE,
        TokenInformationClass: c_int,
        TokenInformation: LPVOID,
        TokenInformationLength: DWORD,
        ReturnLength: *mut DWORD,
    ) -> BOOL;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn ShellExecuteW(
        hwnd: HANDLE,
        lpVerb: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: c_int,
    ) -> HINSTANCE;
}

#[link(name = "winhttp")]
unsafe extern "system" {
    pub fn WinHttpOpen(
        pszAgentW: LPCWSTR,
        dwAccessType: DWORD,
        pszProxyW: LPCWSTR,
        pszProxyBypassW: LPCWSTR,
        dwFlags: DWORD,
    ) -> HANDLE;
    pub fn WinHttpConnect(
        hSession: HANDLE,
        pswzServerName: LPCWSTR,
        nServerPort: c_ushort,
        dwReserved: DWORD,
    ) -> HANDLE;
    pub fn WinHttpOpenRequest(
        hConnect: HANDLE,
        pwszVerb: LPCWSTR,
        pwszObjectName: LPCWSTR,
        pwszVersion: LPCWSTR,
        pwszReferer: LPCWSTR,
        ppwszAcceptTypes: *const LPCWSTR,
        dwFlags: DWORD,
    ) -> HANDLE;
    pub fn WinHttpSetOption(
        hInternet: HANDLE,
        dwOption: DWORD,
        lpBuffer: LPVOID,
        dwBufferLength: DWORD,
    ) -> BOOL;
    pub fn WinHttpSendRequest(
        hRequest: HANDLE,
        lpszHeaders: LPCWSTR,
        dwHeadersLength: DWORD,
        lpOptional: LPVOID,
        dwOptionalLength: DWORD,
        dwTotalLength: DWORD,
        dwContext: usize,
    ) -> BOOL;
    pub fn WinHttpReceiveResponse(hRequest: HANDLE, lpReserved: LPVOID) -> BOOL;
    pub fn WinHttpQueryDataAvailable(
        hRequest: HANDLE,
        lpdwNumberOfBytesAvailable: *mut DWORD,
    ) -> BOOL;
    pub fn WinHttpReadData(
        hRequest: HANDLE,
        lpBuffer: LPVOID,
        dwNumberOfBytesToRead: DWORD,
        lpdwNumberOfBytesRead: *mut DWORD,
    ) -> BOOL;
    pub fn WinHttpQueryHeaders(
        hRequest: HANDLE,
        dwInfoLevel: DWORD,
        pwszName: LPCWSTR,
        lpBuffer: LPVOID,
        lpdwBufferLength: *mut DWORD,
        lpdwIndex: *mut DWORD,
    ) -> BOOL;
    pub fn WinHttpCloseHandle(hInternet: HANDLE) -> BOOL;
}
