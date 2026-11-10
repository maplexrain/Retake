// Windows-specific helper: window enumeration, capture exclusion, and bounds helpers.
// Note: WDA_EXCLUDEFROMCAPTURE requires Windows 10 2004 (Build 19041) or later.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppWindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub is_minimized: bool,
}

#[cfg(windows)]
mod imp {
    use super::AppWindowInfo;

    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn SetWindowDisplayAffinity(hwnd: *mut core::ffi::c_void, dw_affinity: u32) -> i32;
        fn EnumWindows(lpEnumFunc: unsafe extern "system" fn(*mut core::ffi::c_void, isize) -> i32, lParam: isize) -> i32;
        fn IsWindowVisible(hwnd: *mut core::ffi::c_void) -> i32;
        fn IsIconic(hwnd: *mut core::ffi::c_void) -> i32;
        fn GetWindowTextLengthW(hwnd: *mut core::ffi::c_void) -> i32;
        fn GetWindowTextW(hwnd: *mut core::ffi::c_void, lpString: *mut u16, nMaxCount: i32) -> i32;
        fn GetClassNameW(hwnd: *mut core::ffi::c_void, lpClassName: *mut u16, nMaxCount: i32) -> i32;
        fn GetWindowRect(hwnd: *mut core::ffi::c_void, lpRect: *mut RECT) -> i32;
        fn GetWindowLongW(hwnd: *mut core::ffi::c_void, nIndex: i32) -> i32;
        fn GetWindowThreadProcessId(hwnd: *mut core::ffi::c_void, lpdwProcessId: *mut u32) -> u32;
        fn GetCurrentProcessId() -> u32;
    }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmGetWindowAttribute(hwnd: *mut core::ffi::c_void, dwAttribute: u32, pvAttribute: *mut core::ffi::c_void, cbAttribute: u32) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut core::ffi::c_void;
        fn QueryFullProcessImageNameW(hProcess: *mut core::ffi::c_void, dwFlags: u32, lpExeName: *mut u16, lpdwSize: *mut u32) -> i32;
        fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
    }

    const WDA_NONE: u32 = 0x0000_0000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x0000_0011;
    const DWMWA_EXTENDED_FRAME_BOUNDS: u32 = 9;
    const GWL_STYLE: i32 = -16;
    const GWL_EXSTYLE: i32 = -20;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_APPWINDOW: u32 = 0x0004_0000;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    pub fn set_capture_exclusion(hwnd: isize, exclude: bool) {
        unsafe {
            SetWindowDisplayAffinity(
                hwnd as usize as *mut core::ffi::c_void,
                if exclude { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE },
            );
        }
    }

    pub fn window_hwnd(w: &tauri::WebviewWindow) -> Option<isize> {
        w.hwnd().ok().map(|h| h.0 as usize as isize)
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: *mut core::ffi::c_void, lparam: isize) -> i32 {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == GetCurrentProcessId() {
            return 1;
        }

        let text_len = GetWindowTextLengthW(hwnd);
        if text_len <= 0 {
            return 1;
        }

        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
        if len <= 0 {
            return 1;
        }
        let title = String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string();
        if title.is_empty() {
            return 1;
        }

        let mut cls_buf = [0u16; 256];
        let clen = GetClassNameW(hwnd, cls_buf.as_mut_ptr(), 256);
        let class_name = if clen > 0 {
            String::from_utf16_lossy(&cls_buf[..clen as usize])
        } else {
            String::new()
        };

        // Filter out common desktop & shell background windows
        if class_name == "Progman"
            || class_name == "WorkerW"
            || class_name == "Shell_TrayWnd"
            || class_name == "Shell_SecondaryTrayWnd"
            || class_name == "SidekickOverlayWindow"
        {
            return 1;
        }

        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (style & WS_VISIBLE) == 0 {
            return 1;
        }
        if (ex_style & WS_EX_TOOLWINDOW) != 0 && (ex_style & WS_EX_APPWINDOW) == 0 {
            return 1;
        }

        let is_minimized = IsIconic(hwnd) != 0;
        let mut rect = RECT::default();
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut core::ffi::c_void,
            std::mem::size_of::<RECT>() as u32,
        );
        if hr != 0 {
            GetWindowRect(hwnd, &mut rect);
        }

        let w = rect.right - rect.left;
        let h = rect.bottom - rect.top;
        if !is_minimized && (w < 32 || h < 32) {
            return 1;
        }

        let mut process_name = String::new();
        if pid > 0 {
            let hp = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !hp.is_null() {
                let mut exe_buf = [0u16; 512];
                let mut size = 512u32;
                if QueryFullProcessImageNameW(hp, 0, exe_buf.as_mut_ptr(), &mut size) != 0 && size > 0 {
                    let path_str = String::from_utf16_lossy(&exe_buf[..size as usize]);
                    if let Some(name) = std::path::Path::new(&path_str).file_name() {
                        process_name = name.to_string_lossy().to_string();
                    }
                }
                CloseHandle(hp);
            }
        }

        let list = &mut *(lparam as *mut Vec<AppWindowInfo>);
        list.push(AppWindowInfo {
            hwnd: hwnd as usize as isize,
            title,
            process_name,
            pid,
            x: rect.left,
            y: rect.top,
            w,
            h,
            is_minimized,
        });

        1
    }

    pub fn list_windows() -> Vec<AppWindowInfo> {
        let mut list: Vec<AppWindowInfo> = Vec::new();
        unsafe {
            EnumWindows(enum_windows_callback, &mut list as *mut _ as isize);
        }
        list
    }

    #[allow(dead_code)]
pub fn get_window_bounds(hwnd: isize) -> Option<(i32, i32, i32, i32)> {
        unsafe {
            let h = hwnd as usize as *mut core::ffi::c_void;
            if IsWindowVisible(h) == 0 {
                return None;
            }
            let mut rect = RECT::default();
            let hr = DwmGetWindowAttribute(
                h,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut rect as *mut _ as *mut core::ffi::c_void,
                std::mem::size_of::<RECT>() as u32,
            );
            if hr != 0 {
                if GetWindowRect(h, &mut rect) == 0 {
                    return None;
                }
            }
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w > 0 && h > 0 {
                Some((rect.left, rect.top, w, h))
            } else {
                None
            }
        }
    }
}

/// Set or clear the capture exclusion flag for a window.
#[cfg(windows)]
pub fn set_window_capture_exclusion(w: &tauri::WebviewWindow, exclude: bool) {
    if let Some(h) = imp::window_hwnd(w) {
        imp::set_capture_exclusion(h, exclude);
    }
}

#[cfg(windows)]
pub fn list_windows() -> Vec<AppWindowInfo> {
    imp::list_windows()
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn get_window_bounds(hwnd: isize) -> Option<(i32, i32, i32, i32)> {
    imp::get_window_bounds(hwnd)
}

#[cfg(not(windows))]
pub fn set_window_capture_exclusion(_w: &tauri::WebviewWindow, _exclude: bool) {}

#[cfg(not(windows))]
pub fn list_windows() -> Vec<AppWindowInfo> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn get_window_bounds(_hwnd: isize) -> Option<(i32, i32, i32, i32)> {
    None
}
