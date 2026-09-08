use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, POINT},
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreateIcon, CreatePopupMenu, DestroyIcon, DestroyMenu, GetCursorPos,
                HICON, MF_SEPARATOR, MF_STRING, SetForegroundWindow, TPM_BOTTOMALIGN,
                TPM_RETURNCMD, TPM_RIGHTALIGN, TPM_RIGHTBUTTON, TrackPopupMenu, WM_CONTEXTMENU,
                WM_LBUTTONDBLCLK, WM_RBUTTONUP,
            },
        },
    },
    core::{Error, Result, w},
};

pub const CALLBACK_MESSAGE: u32 = 0x8001;
const ICON_ID: u32 = 1;
const OPEN_CONFIG_COMMAND: usize = 100;
const EXIT_COMMAND: usize = 101;
const ICON_SIZE: usize = 32;

pub enum TrayAction {
    OpenConfig,
    Exit,
}

pub struct TrayIcon {
    hwnd: HWND,
    icon: HICON,
}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let icon = create_signal_icon()?;
        let mut data = notification_data(hwnd, icon);
        let tooltip: Vec<u16> = "红绿灯\0".encode_utf16().collect();
        data.szTip[..tooltip.len()].copy_from_slice(&tooltip);
        if unsafe { Shell_NotifyIconW(NIM_ADD, &data) }.as_bool() {
            Ok(Self { hwnd, icon })
        } else {
            let error = Error::from_thread();
            unsafe { DestroyIcon(icon)? };
            Err(error)
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let data = notification_data(self.hwnd, self.icon);
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &data);
            let _ = DestroyIcon(self.icon);
        }
    }
}

pub fn handle_callback(hwnd: HWND, lparam: LPARAM) -> Option<TrayAction> {
    match lparam.0 as u32 {
        WM_LBUTTONDBLCLK => Some(TrayAction::OpenConfig),
        WM_RBUTTONUP | WM_CONTEXTMENU => unsafe { show_context_menu(hwnd) },
        _ => None,
    }
}

unsafe fn show_context_menu(hwnd: HWND) -> Option<TrayAction> {
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return None;
    };
    let menu_result = unsafe {
        AppendMenuW(menu, MF_STRING, OPEN_CONFIG_COMMAND, w!("配置"))
            .and_then(|_| AppendMenuW(menu, MF_SEPARATOR, 0, None))
            .and_then(|_| AppendMenuW(menu, MF_STRING, EXIT_COMMAND, w!("退出")))
    };
    if menu_result.is_err() {
        unsafe {
            let _ = DestroyMenu(menu);
        }
        return None;
    }
    let mut cursor = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut cursor);
        let _ = SetForegroundWindow(hwnd);
    }
    let command = unsafe {
        TrackPopupMenu(
            menu,
            TPM_BOTTOMALIGN | TPM_RIGHTALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
            cursor.x,
            cursor.y,
            Some(0),
            hwnd,
            None,
        )
    };
    unsafe {
        let _ = DestroyMenu(menu);
    }
    match command.0 as usize {
        OPEN_CONFIG_COMMAND => Some(TrayAction::OpenConfig),
        EXIT_COMMAND => Some(TrayAction::Exit),
        _ => None,
    }
}

fn notification_data(hwnd: HWND, icon: HICON) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: CALLBACK_MESSAGE,
        hIcon: icon,
        ..Default::default()
    }
}

fn create_signal_icon() -> Result<HICON> {
    let and_mask = [0u8; ICON_SIZE * ICON_SIZE / 8];
    let mut color = [0u8; ICON_SIZE * ICON_SIZE * 4];
    for y in 7..25 {
        for x in 2..30 {
            let rounded_corner = !(6..=25).contains(&x) && !(11..=20).contains(&y);
            if !rounded_corner {
                set_pixel(&mut color, x, y, (27, 31, 32, 255));
            }
        }
    }
    for (center, rgb) in [(8, (255, 31, 45)), (16, (255, 185, 0)), (24, (0, 214, 100))] {
        for y in 11..21 {
            for x in center - 5..center + 5 {
                let dx = x as i32 - center as i32;
                let dy = y as i32 - 16;
                if dx * dx + dy * dy <= 20 {
                    set_pixel(&mut color, x, y, (rgb.0, rgb.1, rgb.2, 255));
                }
            }
        }
    }
    unsafe {
        CreateIcon(
            None,
            ICON_SIZE as i32,
            ICON_SIZE as i32,
            1,
            32,
            and_mask.as_ptr(),
            color.as_ptr(),
        )
    }
}

fn set_pixel(buffer: &mut [u8], x: usize, y: usize, color: (u8, u8, u8, u8)) {
    let row = ICON_SIZE - 1 - y;
    let index = (row * ICON_SIZE + x) * 4;
    buffer[index] = color.2;
    buffer[index + 1] = color.1;
    buffer[index + 2] = color.0;
    buffer[index + 3] = color.3;
}
