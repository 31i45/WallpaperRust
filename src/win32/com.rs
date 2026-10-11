//! COM 类型定义与 OLE 函数声明（供预览窗口加载图片）。
//!
//! 与 Win32 窗口 API 分离：这里只包含 COM/OLE 相关类型，
//! 窗口逻辑在 preview.rs 中。

#![allow(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::c_void;

// ---- COM 基础类型 ----
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
