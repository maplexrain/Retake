// Windows-specific helper: Exclude window from screen capture/recording (WDA_EXCLUDEFROMCAPTURE).
// Note: WDA_EXCLUDEFROMCAPTURE requires Windows 10 2004 (Build 19041) or later.

#[cfg(windows)]
mod imp {
    #[link(name = "user32")]
    extern "system" {
        fn SetWindowDisplayAffinity(hwnd: *mut core::ffi::c_void, dw_affinity: u32) -> i32;
    }

    const WDA_NONE: u32 = 0x0000_0000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x0000_0011;

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
}

/// Set or clear the capture exclusion flag for a window (selector, countdown, and HUD windows always exclude; main window depends on settings).
#[cfg(windows)]
pub fn set_window_capture_exclusion(w: &tauri::WebviewWindow, exclude: bool) {
    if let Some(h) = imp::window_hwnd(w) {
        imp::set_capture_exclusion(h, exclude);
    }
}

#[cfg(not(windows))]
pub fn set_window_capture_exclusion(_w: &tauri::WebviewWindow, _exclude: bool) {}
