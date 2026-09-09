use std::sync::OnceLock;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        Graphics::Gdi::ScreenToClient,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowW,
            GWLP_USERDATA, HTCAPTION, HTCLIENT, IDC_ARROW, LoadCursorW, MB_ICONERROR,
            MB_ICONINFORMATION, MB_OK, MessageBoxW, RegisterClassW, SW_SHOW, SetForegroundWindow,
            SetWindowLongPtrW, ShowWindow, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN,
            WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_DRAWITEM, WM_ERASEBKGND, WM_NCCREATE,
            WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WNDCLASSW, WS_EX_APPWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
    core::{Error, HRESULT, PCWSTR, Result, w},
};

use crate::{
    config_actions::{Command, CommandOutcome, ConfigState},
    config_controls::{CLOSE_LEFT, TITLEBAR_HEIGHT, WINDOW_HEIGHT, WINDOW_WIDTH},
    settings::Settings,
};

const CLASS_NAME: PCWSTR = w!("VibePulseConfigWindow");
const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);
static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

struct ConfigInit<'a> {
    settings: &'a Settings,
    settings_path: &'a str,
}

pub fn show(owner: HWND) -> Result<()> {
    if let Ok(window) = unsafe { FindWindowW(CLASS_NAME, PCWSTR::null()) } {
        unsafe {
            let _ = ShowWindow(window, SW_SHOW);
            let _ = SetForegroundWindow(window);
        }
        return Ok(());
    }
    register_class()?;
    let settings = Settings::load_or_create().map_err(app_error)?;
    let settings_path = Settings::path_display().map_err(app_error)?;
    let init = ConfigInit {
        settings: &settings,
        settings_path: &settings_path,
    };
    unsafe {
        let _ = CreateWindowExW(
            WS_EX_APPWINDOW,
            CLASS_NAME,
            w!("Vibe Pulse · Hook 配置"),
            WS_POPUP | WS_VISIBLE,
            240,
            160,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            Some(owner),
            None,
            Some(GetModuleHandleW(None)?.into()),
            Some((&init as *const ConfigInit).cast()),
        )?;
    }
    Ok(())
}

fn register_class() -> Result<()> {
    if CLASS_REGISTERED.get().is_some() {
        return Ok(());
    }
    let instance = unsafe { GetModuleHandleW(None)? };
    let class = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        hInstance: instance.into(),
        lpszClassName: CLASS_NAME,
        lpfnWndProc: Some(window_proc),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(Error::from_thread());
    }
    let _ = CLASS_REGISTERED.set(());
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => unsafe { initialize_state(hwnd, lparam) },
        WM_CREATE => initialize_content(hwnd),
        WM_COMMAND => handle_window_command(hwnd, wparam),
        WM_PAINT => paint(hwnd),
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => unsafe { control_color(hwnd, wparam, true) },
        WM_CTLCOLORBTN => unsafe { control_color(hwnd, wparam, false) },
        WM_DRAWITEM => draw_item(hwnd, lparam),
        WM_NCHITTEST => unsafe { hit_test(hwnd, lparam) },
        WM_ERASEBKGND => LRESULT(1),
        WM_CLOSE => close(hwnd),
        WM_NCDESTROY => {
            unsafe { destroy_state(hwnd) };
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn initialize_content(hwnd: HWND) -> LRESULT {
    let result = unsafe { state(hwnd) }
        .ok_or_else(Error::from_thread)
        .and_then(|state| state.initialize_content(hwnd));
    match result {
        Ok(()) => LRESULT(0),
        Err(error) => {
            show_error(hwnd, &error);
            LRESULT(-1)
        }
    }
}

fn handle_window_command(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    let result = unsafe { state(hwnd) }.map(|state| {
        state.handle(Command {
            hwnd,
            identifier: (wparam.0 & 0xffff) as isize,
            notification: ((wparam.0 >> 16) & 0xffff) as u32,
        })
    });
    match result {
        Some(Ok(CommandOutcome::Copied)) => show_info(hwnd, "配置已复制到剪贴板。"),
        Some(Ok(CommandOutcome::Close)) => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        Some(Err(error)) => show_error(hwnd, &error),
        _ => {}
    }
    LRESULT(0)
}

fn paint(hwnd: HWND) -> LRESULT {
    if let Some(state) = unsafe { state(hwnd) } {
        unsafe { state.paint(hwnd) };
    }
    LRESULT(0)
}

fn draw_item(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    if let Some(state) = unsafe { state(hwnd) } {
        unsafe { state.draw_item(lparam) };
    }
    LRESULT(1)
}

unsafe fn hit_test(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let mut point = point_from_lparam(lparam);
    unsafe {
        let _ = ScreenToClient(hwnd, &mut point);
    }
    if point.y < TITLEBAR_HEIGHT && point.x < CLOSE_LEFT {
        LRESULT(HTCAPTION as isize)
    } else {
        LRESULT(HTCLIENT as isize)
    }
}

fn close(hwnd: HWND) -> LRESULT {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
    LRESULT(0)
}

unsafe fn initialize_state(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
    let init = unsafe { &*(create.lpCreateParams as *const ConfigInit) };
    let state = Box::new(ConfigState::new(
        init.settings.clone(),
        init.settings_path.to_owned(),
    ));
    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize) };
    LRESULT(1)
}

unsafe fn destroy_state(hwnd: HWND) {
    let pointer = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) } as *mut ConfigState;
    if !pointer.is_null() {
        unsafe { drop(Box::from_raw(pointer)) };
    }
}

unsafe fn control_color(hwnd: HWND, wparam: WPARAM, edit: bool) -> LRESULT {
    let brush = unsafe { state(hwnd) }
        .map(|state| unsafe { state.control_color(wparam, edit) })
        .unwrap_or_default();
    LRESULT(brush)
}

fn point_from_lparam(lparam: LPARAM) -> POINT {
    let value = lparam.0 as u32;
    POINT {
        x: (value as u16 as i16) as i32,
        y: ((value >> 16) as u16 as i16) as i32,
    }
}

unsafe fn state(hwnd: HWND) -> Option<&'static mut ConfigState> {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut ConfigState;
    unsafe { pointer.as_mut() }
}

fn show_info(hwnd: HWND, message: &str) {
    show_message(hwnd, message, MB_OK | MB_ICONINFORMATION);
}

fn show_error(hwnd: HWND, error: &Error) {
    show_message(hwnd, &error.to_string(), MB_OK | MB_ICONERROR);
}

fn show_message(
    hwnd: HWND,
    message: &str,
    style: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
) {
    let message: Vec<u16> = message.encode_utf16().chain([0]).collect();
    unsafe {
        let _ = MessageBoxW(
            Some(hwnd),
            PCWSTR(message.as_ptr()),
            w!("Vibe Pulse"),
            style,
        );
    }
}

fn app_error(message: String) -> Error {
    Error::new(APP_FAILURE, message)
}
