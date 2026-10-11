//! 锁屏壁纸预览窗口。
//!
//! 因为设置锁屏后无法直接查看效果，所以先显示预览，
//! 用户确认满意后再设为锁屏壁纸。

#![allow(unsafe_op_in_unsafe_fn)] // FFI 密集模块，unsafe 操作是常态

use super::*;
use core::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

// ---- 窗口/消息常量 ----
pub const WM_CREATE: UINT = 0x0001;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_SETFONT: UINT = 0x0030;

pub const DT_CENTER: UINT = 0x0000_0001;
pub const DT_VCENTER: UINT = 0x0000_0004;
pub const DT_SINGLELINE: UINT = 0x0000_0020;

/// GetDeviceCaps 索引：每逻辑英寸像素数（X 轴 DPI）。
pub const LOGPIXELSX: c_int = 88;

// CreateFontW 参数
pub const FW_NORMAL: c_int = 400;
pub const DEFAULT_CHARSET: c_int = 1;
pub const OUT_TT_PRECIS: c_int = 4;
pub const CLEARTYPE_QUALITY: c_int = 5;
pub const DEFAULT_PITCH: c_int = 0;

pub const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF_0000;
pub const WS_VISIBLE: DWORD = 0x1000_0000;
pub const WS_CHILD: DWORD = 0x4000_0000;
pub const BS_PUSHBUTTON: DWORD = 0x0000_0000;
pub const CS_HREDRAW: UINT = 0x0002;
pub const CS_VREDRAW: UINT = 0x0001;
pub const CW_USEDEFAULT: c_int = -2147483648i32 as c_int;
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;
pub const IDI_APPLICATION: LPCWSTR = 32512usize as LPCWSTR;

pub const BTN_SET: usize = 1001;
pub const BTN_NEXT: usize = 1002;

// ---- COM 类型 ----
pub type HRESULT = i32;
pub type ULONG = c_ulong;
pub type LONG = i32;
pub type SHORT = i16;
pub type OLE_HANDLE = c_uint;
pub type OLE_XSIZE_HIMETRIC = LONG;
pub type OLE_YSIZE_HIMETRIC = LONG;
pub type OLE_XPOS_HIMETRIC = LONG;
pub type OLE_YPOS_HIMETRIC = LONG;

#[repr(C)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
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

pub type HDC = HANDLE;
pub type HFONT = HANDLE;
pub type HICON = HANDLE;
pub type HCURSOR = HANDLE;
pub type HBRUSH = HANDLE;
pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;
pub type HWND = HANDLE;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;

#[repr(C)]
pub struct IID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

// IID_IPicture: {7BF80980-BF32-101A-8BBB-00AA00300CAB}
pub const IID_IPICTURE: IID = IID {
    data1: 0x7BF8_0980,
    data2: 0xBF32,
    data3: 0x101A,
    data4: [0x8B, 0xBB, 0x00, 0xAA, 0x00, 0x30, 0x0C, 0xAB],
};

#[repr(C)]
pub struct IPictureVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IPicture, *const IID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IPicture) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IPicture) -> ULONG,
    pub get_Handle: unsafe extern "system" fn(*mut IPicture, *mut OLE_HANDLE) -> HRESULT,
    pub get_hPal: unsafe extern "system" fn(*mut IPicture, *mut OLE_HANDLE) -> HRESULT,
    pub get_Type: unsafe extern "system" fn(*mut IPicture, *mut SHORT) -> HRESULT,
    pub get_Width: unsafe extern "system" fn(*mut IPicture, *mut OLE_XSIZE_HIMETRIC) -> HRESULT,
    pub get_Height: unsafe extern "system" fn(*mut IPicture, *mut OLE_YSIZE_HIMETRIC) -> HRESULT,
    pub Render: unsafe extern "system" fn(
        *mut IPicture,
        HDC,
        c_int,
        c_int,
        c_int,
        c_int,
        OLE_XPOS_HIMETRIC,
        OLE_YPOS_HIMETRIC,
        OLE_XSIZE_HIMETRIC,
        OLE_YSIZE_HIMETRIC,
        *const RECT,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IPicture {
    pub lpVtbl: *const IPictureVtbl,
}

#[link(name = "user32")]
unsafe extern "system" {
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
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetProcessDPIAware() -> BOOL;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn GetDeviceCaps(hdc: HDC, index: c_int) -> c_int;
    pub fn CreateFontW(
        cHeight: c_int,
        cWidth: c_int,
        cEscapement: c_int,
        cOrientation: c_int,
        cWeight: c_int,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: c_int,
        iOutPrecision: c_int,
        iClipPrecision: c_int,
        iQuality: c_int,
        iPitchAndFamily: c_int,
        pszFaceName: LPCWSTR,
    ) -> HFONT;
    pub fn SelectObject(hdc: HDC, h: HANDLE) -> HANDLE;
    pub fn DeleteObject(ho: HANDLE) -> BOOL;
}

pub type ATOM = u16;

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
pub struct POINT {
    pub x: c_int,
    pub y: c_int,
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn OleInitialize(pvReserved: LPVOID) -> HRESULT;
    pub fn OleUninitialize();
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    pub fn OleLoadPicturePath(
        szURL: LPCWSTR,
        pCaller: LPVOID,
        dwReserved: DWORD,
        clrReserved: c_uint,
        riid: *const IID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT;
}

/// 图片获取器：返回新下载的图片临时路径（None 表示失败）。
///
/// 由调用方注入，使预览窗口（机制层）不依赖下载编排（编排层），
/// 既消除层间反向依赖，也便于在测试中替换为假数据源。
pub type Fetcher = dyn FnMut() -> Option<String>;

/// 将 96 DPI 基准下的像素值缩放到实际 DPI（纯逻辑，可测试）。
///
/// 采用 Win32 MulDiv 的语义（四舍五入到最近像素）：(v * dpi + 48) / 96。
pub fn scale_px(value: c_int, dpi: u32) -> c_int {
    ((value as i64 * dpi as i64 + 48) / 96) as c_int
}

/// 预览窗口状态。
pub struct PreviewState {
    pub image_path: Option<String>,
    pub picture: Option<*mut IPicture>,
    pub loading: bool,
    pub fetch: Box<Fetcher>,
    /// 窗口所在显示器 DPI（96/120/144…），所有尺寸按它缩放。
    pub dpi: u32,
    /// 按 DPI 缩放的 UI 字体（退出时释放）。
    pub font: HFONT,
}

/// 用户是否点击了"设为锁屏"。
static CONFIRMED: AtomicBool = AtomicBool::new(false);

/// 运行预览窗口，返回用户确认时的图片路径。
/// None = 用户关闭/取消。
///
/// `fetch` 负责提供图片（首次加载与每次"换一张"都会调用）。
///
/// 体验优化：先创建窗口显示"正在下载..."，再下载图片，
/// 避免用户双击后数秒无任何反馈。
pub fn show_preview<F>(fetch: F) -> Option<String>
where
    F: FnMut() -> Option<String> + 'static,
{
    CONFIRMED.store(false, Ordering::SeqCst);
    unsafe {
        OleInitialize(core::ptr::null_mut());
        // 声明系统级 DPI 感知：否则在缩放显示器（如 150%）上整个窗口
        // 会被系统位图拉伸，图片和文字都发虚。必须在创建窗口/取 DC 前调用。
        SetProcessDPIAware();
    }

    // 读取系统 DPI（96 = 100%，144 = 150%），全部 UI 尺寸按它缩放
    let dpi = unsafe {
        let dc = GetDC(core::ptr::null_mut());
        let mut d = 96u32;
        if !dc.is_null() {
            let v = GetDeviceCaps(dc, LOGPIXELSX);
            if v >= 96 {
                d = v as u32;
            }
            ReleaseDC(core::ptr::null_mut(), dc);
        }
        d
    };

    let state = Box::new(PreviewState {
        image_path: None,
        picture: None,
        loading: true,
        fetch: Box::new(fetch),
        dpi,
        font: core::ptr::null_mut(),
    });
    let state_ptr = Box::into_raw(state);

    let class_name: Vec<u16> = "WallpaperRustPreview"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let hinstance = unsafe { GetModuleHandleW(core::ptr::null()) };

    let wc = WNDCLASSEXW {
        cbSize: core::mem::size_of::<WNDCLASSEXW>() as UINT,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: wnd_proc,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: unsafe { LoadIconW(core::ptr::null_mut(), IDI_APPLICATION) },
        hCursor: unsafe { LoadCursorW(core::ptr::null_mut(), IDC_ARROW) },
        hbrBackground: core::ptr::null_mut(),
        lpszMenuName: core::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: core::ptr::null_mut(),
    };

    unsafe {
        RegisterClassExW(&wc);
    }

    let title: Vec<u16> = "锁屏壁纸预览".encode_utf16().chain(Some(0)).collect();

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            scale_px(900, dpi),
            scale_px(700, dpi),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            hinstance,
            state_ptr as LPVOID,
        )
    };

    if hwnd.is_null() {
        unsafe {
            drop(Box::from_raw(state_ptr));
            OleUninitialize();
        }
        return None;
    }

    unsafe {
        ShowWindow(hwnd, 5); // SW_SHOW
        UpdateWindow(hwnd);
    }

    // 先显示"正在下载..."，再下载（窗口已可见，用户有反馈）
    let fetched = unsafe { ((*state_ptr).fetch)() };
    if let Some(path) = fetched {
        unsafe {
            let state = &mut *state_ptr;
            state.image_path = Some(path);
            state.loading = false;
            // 下载完成，启用按钮
            let btn_set = GetDlgItem(hwnd, BTN_SET as c_int);
            let btn_next = GetDlgItem(hwnd, BTN_NEXT as c_int);
            EnableWindow(btn_set, 1);
            EnableWindow(btn_next, 1);
            InvalidateRect(hwnd, core::ptr::null(), 1);
        }
    } else {
        // 下载失败：恢复标题，但按钮保持禁用（无图可设）
        let fail_title: Vec<u16> = "下载失败，请关闭重试"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        unsafe {
            SetWindowTextW(hwnd, fail_title.as_ptr());
        }
    }

    let mut msg: MSG = unsafe { core::mem::zeroed() };
    loop {
        let ret = unsafe { GetMessageW(&mut msg, core::ptr::null_mut(), 0, 0) };
        if ret == 0 || ret == -1 {
            break;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // 清理并返回结果
    let result = if CONFIRMED.load(Ordering::SeqCst) {
        unsafe { (*state_ptr).image_path.clone() }
    } else {
        None
    };

    unsafe {
        let state = Box::from_raw(state_ptr);
        if let Some(pic) = state.picture {
            ((*(*pic).lpVtbl).Release)(pic);
        }
        if !state.font.is_null() {
            DeleteObject(state.font);
        }
        OleUninitialize();
    }

    result
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = lparam as *const c_void as *const CREATESTRUCTW;
            let state_ptr = (*create_struct).lpCreateParams as *mut PreviewState;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

            let dpi = (*state_ptr).dpi;

            // 按 DPI 创建 UI 字体（9 磅 Segoe UI），供按钮与提示文字共用
            let face: Vec<u16> = "Segoe UI".encode_utf16().chain(Some(0)).collect();
            let font = CreateFontW(
                -(((9 * dpi as i64) + 36) / 72) as c_int, // -MulDiv(9, dpi, 72)
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_TT_PRECIS,
                0,
                CLEARTYPE_QUALITY,
                DEFAULT_PITCH,
                face.as_ptr(),
            );
            (*state_ptr).font = font;

            // 创建按钮（尺寸按 DPI 缩放；位置由 WM_SIZE 统一布局）
            let btn_class: Vec<u16> = "BUTTON".encode_utf16().chain(Some(0)).collect();
            let set_text: Vec<u16> = "设为锁屏壁纸".encode_utf16().chain(Some(0)).collect();
            let next_text: Vec<u16> = "换一张".encode_utf16().chain(Some(0)).collect();
            let btn_set = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                set_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                0,
                0,
                scale_px(150, dpi),
                scale_px(40, dpi),
                hwnd,
                BTN_SET as HANDLE,
                GetModuleHandleW(core::ptr::null()),
                core::ptr::null_mut(),
            );
            let btn_next = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                next_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                0,
                0,
                scale_px(120, dpi),
                scale_px(40, dpi),
                hwnd,
                BTN_NEXT as HANDLE,
                GetModuleHandleW(core::ptr::null()),
                core::ptr::null_mut(),
            );
            SendMessageW(btn_set, WM_SETFONT, font as WPARAM, 1);
            SendMessageW(btn_next, WM_SETFONT, font as WPARAM, 1);
            // 初始禁用按钮（下载中）
            EnableWindow(btn_set, 0);
            EnableWindow(btn_next, 0);
            0
        }
        WM_SIZE => {
            // 窗口大小变化时，按 DPI 把按钮重新布局到底部居中
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PreviewState;
            let dpi = if state_ptr.is_null() {
                96
            } else {
                (*state_ptr).dpi
            };
            let btn_set = GetDlgItem(hwnd, BTN_SET as c_int);
            let btn_next = GetDlgItem(hwnd, BTN_NEXT as c_int);
            let width = (lparam & 0xFFFF) as c_int;
            let height = ((lparam >> 16) & 0xFFFF) as c_int;
            let set_w = scale_px(150, dpi);
            let next_w = scale_px(120, dpi);
            let btn_h = scale_px(40, dpi);
            let gap = scale_px(20, dpi);
            let btn_y = height - scale_px(70, dpi);
            let start_x = (width - set_w - gap - next_w) / 2;
            MoveWindow(btn_set, start_x, btn_y, set_w, btn_h, 1);
            MoveWindow(btn_next, start_x + set_w + gap, btn_y, next_w, btn_h, 1);
            0
        }
        WM_PAINT => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PreviewState;
            if state_ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            let state = &mut *state_ptr;

            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            // 提示文字使用按 DPI 缩放的字体（图片 Render 不受字体影响）
            let old_obj = if state.font.is_null() {
                core::ptr::null_mut()
            } else {
                SelectObject(hdc, state.font)
            };
            let dpi = state.dpi;

            if state.loading || state.image_path.is_none() {
                // 显示"正在下载..."
                let mut client: RECT = core::mem::zeroed();
                GetClientRect(hwnd, &mut client);
                let text: Vec<u16> = "正在下载壁纸...".encode_utf16().chain(Some(0)).collect();
                DrawTextW(
                    hdc,
                    text.as_ptr(),
                    -1,
                    &mut client,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            } else if let Some(path) = state.image_path.as_ref() {
                // 加载图片
                if state.picture.is_none() {
                    let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
                    let mut ppv: LPVOID = core::ptr::null_mut();
                    let hr = OleLoadPicturePath(
                        wide_path.as_ptr(),
                        core::ptr::null_mut(),
                        0,
                        0,
                        &IID_IPICTURE,
                        &mut ppv,
                    );
                    if hr >= 0 && !ppv.is_null() {
                        state.picture = Some(ppv as *mut IPicture);
                    }
                }

                // 绘制图片（居中缩放）
                if let Some(pic) = state.picture {
                    let vtbl = &*(*pic).lpVtbl;
                    let mut hm_w: OLE_XSIZE_HIMETRIC = 0;
                    let mut hm_h: OLE_YSIZE_HIMETRIC = 0;
                    (vtbl.get_Width)(pic, &mut hm_w);
                    (vtbl.get_Height)(pic, &mut hm_h);

                    // HIMETRIC → 像素：2540 HIMETRIC = 1 英寸
                    let img_w = (hm_w as i64 * dpi as i64 / 2540) as c_int;
                    let img_h = (hm_h as i64 * dpi as i64 / 2540) as c_int;

                    let mut client: RECT = core::mem::zeroed();
                    GetClientRect(hwnd, &mut client);
                    let cli_w = client.right - client.left;
                    let cli_h = client.bottom - client.top - scale_px(70, dpi);

                    let scale = (cli_w as f64 / img_w as f64).min(cli_h as f64 / img_h as f64);
                    let draw_w = (img_w as f64 * scale) as c_int;
                    let draw_h = (img_h as f64 * scale) as c_int;
                    let draw_x = (cli_w - draw_w) / 2;
                    let draw_y = (cli_h - draw_h) / 2;

                    (vtbl.Render)(
                        pic,
                        hdc,
                        draw_x,
                        draw_y,
                        draw_w,
                        draw_h,
                        0,
                        hm_h,
                        hm_w,
                        -hm_h,
                        core::ptr::null(),
                    );
                } else {
                    // 有路径但解码失败（文件损坏等罕见情况）：给出可操作提示
                    let mut client: RECT = core::mem::zeroed();
                    GetClientRect(hwnd, &mut client);
                    let text: Vec<u16> = "图片加载失败，请换一张"
                        .encode_utf16()
                        .chain(Some(0))
                        .collect();
                    DrawTextW(
                        hdc,
                        text.as_ptr(),
                        -1,
                        &mut client,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                }
            }

            if !old_obj.is_null() {
                SelectObject(hdc, old_obj);
            }
            EndPaint(hwnd, &ps);
            0
        }
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            match id {
                BTN_SET => {
                    // 用户确认设为锁屏，设置标志并关闭窗口
                    CONFIRMED.store(true, Ordering::SeqCst);
                    DestroyWindow(hwnd);
                }
                BTN_NEXT => {
                    // 换一张：禁用按钮显示加载，先下载新图，
                    // 成功后再释放并删除旧图；失败则保留当前图供重试。
                    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PreviewState;
                    if !state_ptr.is_null() {
                        let state = &mut *state_ptr;

                        let btn_set = GetDlgItem(hwnd, BTN_SET as c_int);
                        let btn_next = GetDlgItem(hwnd, BTN_NEXT as c_int);
                        EnableWindow(btn_set, 0);
                        EnableWindow(btn_next, 0);
                        let loading_title: Vec<u16> =
                            "正在下载...".encode_utf16().chain(Some(0)).collect();
                        SetWindowTextW(hwnd, loading_title.as_ptr());
                        InvalidateRect(hwnd, core::ptr::null(), 1);

                        // 获取下一张（阻塞 UI，但有加载提示）
                        match (state.fetch)() {
                            Some(new_path) => {
                                // 新图就绪，再替换旧资源
                                if let Some(pic) = state.picture.take() {
                                    ((*(*pic).lpVtbl).Release)(pic);
                                }
                                if let Some(old) = state.image_path.take() {
                                    let _ = std::fs::remove_file(&old);
                                }
                                state.image_path = Some(new_path);
                                state.loading = false;
                                let title: Vec<u16> =
                                    "锁屏壁纸预览".encode_utf16().chain(Some(0)).collect();
                                SetWindowTextW(hwnd, title.as_ptr());
                            }
                            None => {
                                // 下载失败：保留当前图片，用户可重试
                                let fail_title: Vec<u16> = "下载失败，请再试一次"
                                    .encode_utf16()
                                    .chain(Some(0))
                                    .collect();
                                SetWindowTextW(hwnd, fail_title.as_ptr());
                            }
                        }

                        EnableWindow(btn_set, 1);
                        EnableWindow(btn_next, 1);
                        InvalidateRect(hwnd, core::ptr::null(), 1);
                    }
                }
                _ => {}
            }
            0
        }
        WM_CLOSE | WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[repr(C)]
struct CREATESTRUCTW {
    lpCreateParams: LPVOID,
    hInstance: HMODULE,
    hMenu: HANDLE,
    hwndParent: HWND,
    cy: c_int,
    cx: c_int,
    y: c_int,
    x: c_int,
    style: LONG,
    lpszName: LPCWSTR,
    lpszClass: LPCWSTR,
    dwExStyle: DWORD,
}

const GWLP_USERDATA: c_int = -21;

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: isize) -> isize;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> isize;
}

#[cfg(test)]
mod tests {
    use super::scale_px;

    #[test]
    fn scale_px_identity_at_96_dpi() {
        assert_eq!(scale_px(0, 96), 0);
        assert_eq!(scale_px(120, 96), 120);
        assert_eq!(scale_px(900, 96), 900);
    }

    #[test]
    fn scale_px_150_percent_on_144_dpi() {
        assert_eq!(scale_px(150, 144), 225);
        assert_eq!(scale_px(120, 144), 180);
        assert_eq!(scale_px(40, 144), 60);
        assert_eq!(scale_px(20, 144), 30);
        assert_eq!(scale_px(70, 144), 105);
        assert_eq!(scale_px(900, 144), 1350);
        assert_eq!(scale_px(700, 144), 1050);
    }

    #[test]
    fn scale_px_125_percent_on_120_dpi() {
        assert_eq!(scale_px(9, 120), 11); // 四舍五入：11.25 → 11
        assert_eq!(scale_px(150, 120), 188); // 187.5 → 188
    }
}
