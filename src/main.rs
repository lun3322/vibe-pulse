#![windows_subsystem = "windows"]

mod app;
mod clipboard;
mod config_actions;
mod config_button_paint;
mod config_combo_box;
mod config_control_factory;
mod config_controls;
mod config_paint;
mod config_paint_primitives;
mod config_theme;
mod config_window;
mod drawing;
mod hook_config;
mod http_server;
mod model;
mod network;
mod raster;
mod renderer;
mod settings;
mod tooltip;
mod tooltip_content;
mod tooltip_paint;
mod tray;

use app::{AppState, HOOK_EVENT_MESSAGE};
use renderer::pixel_size;
use tray::TrayAction;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::WM_MOUSELEAVE,
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem,
                SetProcessDpiAwarenessContext,
            },
            Input::KeyboardAndMouse::ReleaseCapture,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
                GetMessageW, HTCAPTION, IDC_ARROW, KillTimer, LoadCursorW, MB_ICONERROR, MB_OK,
                MSG, MessageBoxW, PostQuitMessage, RegisterClassW, SendMessageW, SetTimer,
                SetWindowLongPtrW, TranslateMessage, WINDOW_EX_STYLE, WM_DESTROY, WM_LBUTTONDOWN,
                WM_MOUSEMOVE, WM_NCDESTROY, WM_NCLBUTTONDOWN, WM_TIMER, WNDCLASSW, WS_EX_LAYERED,
                WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
            },
        },
    },
    core::{Error, HRESULT, PCWSTR, Result, w},
};

const CLASS_NAME: PCWSTR = w!("VibePulseSignalWindow");
const WINDOW_MARGIN: f32 = 18.0;
const CONTENT_SCALE: f32 = 0.32;
const TIMER_ID: usize = 1;
const FRAME_INTERVAL_MS: u32 = 33;
const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);

fn main() {
    if let Err(error) = run() {
        show_error(&error);
    }
}

fn run() -> Result<()> {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
        .map_err(|error| win_error("无法启用 DPI 感知", error))?;
    let instance =
        unsafe { GetModuleHandleW(None) }.map_err(|error| win_error("无法获取应用模块", error))?;
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }
        .map_err(|error| win_error("无法加载鼠标指针", error))?;
    let class = WNDCLASSW {
        hCursor: cursor,
        hInstance: instance.into(),
        lpszClassName: CLASS_NAME,
        lpfnWndProc: Some(window_proc),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(win_error("无法注册主窗口", Error::from_thread()));
    }
    let dpi_scale = unsafe { GetDpiForSystem() } as f32 / 96.0;
    let render_scale = dpi_scale * CONTENT_SCALE;
    let hwnd = create_window(instance.into(), dpi_scale, render_scale)
        .map_err(|error| win_error("无法创建主窗口", error))?;
    if let Err(error) = initialize_window(hwnd, render_scale) {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
        return Err(win_error("无法初始化应用状态", error));
    }
    message_loop().map_err(|error| win_error("主消息循环异常", error))
}

fn create_window(
    instance: windows::Win32::Foundation::HINSTANCE,
    dpi_scale: f32,
    render_scale: f32,
) -> Result<HWND> {
    let (width, height) = pixel_size(1, render_scale);
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0),
            CLASS_NAME,
            w!("Vibe Pulse"),
            WS_POPUP,
            (WINDOW_MARGIN * dpi_scale).round() as i32,
            (WINDOW_MARGIN * dpi_scale).round() as i32,
            width,
            height,
            None,
            None,
            Some(instance),
            None,
        )
    }
}

fn initialize_window(hwnd: HWND, scale: f32) -> Result<()> {
    let state = Box::new(AppState::new(hwnd, scale)?);
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
        if SetTimer(Some(hwnd), TIMER_ID, FRAME_INTERVAL_MS, None) == 0 {
            return Err(Error::from_thread());
        }
    }
    Ok(())
}

fn message_loop() -> Result<()> {
    let mut message = MSG::default();
    loop {
        let status = unsafe { GetMessageW(&mut message, None, 0, 0) }.0;
        if status == -1 {
            return Err(Error::from_thread());
        }
        if status == 0 {
            return Ok(());
        }
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let result = match message {
        HOOK_EVENT_MESSAGE => unsafe { app_state(hwnd) }.map(|state| state.receive_hooks(hwnd)),
        WM_TIMER if wparam.0 == TIMER_ID => {
            unsafe { app_state(hwnd) }.map(|state| state.tick(hwnd))
        }
        WM_MOUSEMOVE => unsafe { app_state(hwnd) }.map(|state| {
            let (_, y) = cursor_position(lparam);
            state.mouse_move(hwnd, y)
        }),
        WM_MOUSELEAVE => unsafe { app_state(hwnd) }.map(|state| state.mouse_leave(hwnd)),
        WM_LBUTTONDOWN => unsafe { app_state(hwnd) }.map(|state| handle_click(hwnd, state, lparam)),
        tray::CALLBACK_MESSAGE => {
            handle_tray_action(hwnd, lparam);
            return LRESULT(0);
        }
        WM_DESTROY => {
            unsafe {
                let _ = KillTimer(Some(hwnd), TIMER_ID);
                PostQuitMessage(0);
            }
            return LRESULT(0);
        }
        WM_NCDESTROY => {
            let pointer = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) } as *mut AppState;
            if !pointer.is_null() {
                unsafe {
                    drop(Box::from_raw(pointer));
                }
            }
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        }
        _ => return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    };
    if let Some(Err(error)) = result {
        show_error(&error);
    }
    LRESULT(0)
}

fn handle_click(hwnd: HWND, state: &mut AppState, lparam: LPARAM) -> Result<()> {
    let (x, y) = cursor_position(lparam);
    if !state.click(hwnd, x, y)? {
        unsafe {
            let _ = ReleaseCapture();
            SendMessageW(
                hwnd,
                WM_NCLBUTTONDOWN,
                Some(WPARAM(HTCAPTION as usize)),
                Some(LPARAM(0)),
            );
        }
    }
    Ok(())
}

fn handle_tray_action(hwnd: HWND, lparam: LPARAM) {
    match tray::handle_callback(hwnd, lparam) {
        Some(TrayAction::OpenConfig) => {
            if let Err(error) = config_window::show(hwnd) {
                show_error(&error);
            }
        }
        Some(TrayAction::Exit) => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        None => {}
    }
}

unsafe fn app_state(hwnd: HWND) -> Option<&'static mut AppState> {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut AppState;
    unsafe { pointer.as_mut() }
}

fn cursor_position(lparam: LPARAM) -> (i32, i32) {
    let value = lparam.0 as u32;
    (
        (value as u16 as i16) as i32,
        ((value >> 16) as u16 as i16) as i32,
    )
}

fn win_error(context: &str, error: Error) -> Error {
    let code = if error.code().is_ok() {
        APP_FAILURE
    } else {
        error.code()
    };
    Error::new(code, format!("{context}：{error}"))
}

fn show_error(error: &Error) {
    let text: Vec<u16> = format!("Vibe Pulse：{error}\0").encode_utf16().collect();
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Vibe Pulse"),
            MB_OK | MB_ICONERROR,
        );
    }
}
