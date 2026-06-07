#[cfg(target_os = "windows")]
pub fn get_active_window_title() -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut buffer = [0u16; 512];
        let length = GetWindowTextW(hwnd, &mut buffer);
        if length == 0 {
            return None;
        }

        Some(String::from_utf16_lossy(
            &buffer[..length as usize],
        ))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_window_title() -> Option<String> {
    None
}
