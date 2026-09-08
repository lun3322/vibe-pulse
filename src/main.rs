#![windows_subsystem = "windows"]

mod drawing;
mod renderer;
mod tray;

use std::time::Instant;

use renderer::{BASE_HEIGHT, BASE_WIDTH, LayeredSurface};
use tray::TrayIcon;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem,
                SetProcessDpiAwarenessContext,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
                HTCAPTION, IDC_ARROW, KillTimer, LoadCursorW, MB_ICONERROR, MB_OK, MSG,
                MessageBoxW, PostQuitMessage, RegisterClassW, SW_SHOWNOACTIVATE, SetTimer,
                SetWindowLongPtrW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WM_DESTROY,
                WM_NCDESTROY, WM_NCHITTEST, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
                WS_EX_TOPMOST, WS_POPUP,
            },
        },
    },
    core::{Error, PCWSTR, Result, w},
};

const CLASS_NAME: PCWSTR = w!("VibePulseSignalWindow");
const WINDOW_MARGIN: f32 = 18.0;
const CONTENT_SCALE: f32 = 0.32;
const TIMER_ID: usize = 1;
const FRAME_INTERVAL_MS: u32 = 33;

struct AppState {
    surface: LayeredSurface,
    tray: Option<TrayIcon>,
    started_at: Instant,
}

impl AppState {
    fn new(scale: f32) -> Result<Self> {
        Ok(Self {
            surface: LayeredSurface::new(scale)?,
            tray: None,
            started_at: Instant::now(),
        })
    }

    fn draw(&mut self, hwnd: HWND) -> Result<()> {
        self.surface.render(self.started_at.elapsed().as_secs_f32());
        self.surface.present(hwnd)
    }
}

fn main() {
    if let Err(error) = run() {
        show_error(&error);
    }
}

fn run() -> Result<()> {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)? };
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

    let dpi_scale = unsafe { GetDpiForSystem() } as f32 / 96.0;
    let render_scale = dpi_scale * CONTENT_SCALE;
    let hwnd = create_window(instance.into(), dpi_scale, render_scale)?;
    if let Err(error) = initialize_window(hwnd, render_scale) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
        }
        return Err(error);
    }
    message_loop()
}

fn create_window(
    instance: windows::Win32::Foundation::HINSTANCE,
    dpi_scale: f32,
    render_scale: f32,
) -> Result<HWND> {
    let extended_style = WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0);
    unsafe {
        CreateWindowExW(
            extended_style,
            CLASS_NAME,
            w!("横排红绿灯"),
            WS_POPUP,
            (WINDOW_MARGIN * dpi_scale).round() as i32,
            (WINDOW_MARGIN * dpi_scale).round() as i32,
            (BASE_WIDTH as f32 * render_scale).round() as i32,
            (BASE_HEIGHT as f32 * render_scale).round() as i32,
            None,
            None,
            Some(instance),
            None,
        )
    }
}

fn initialize_window(hwnd: HWND, scale: f32) -> Result<()> {
    let mut state = Box::new(AppState::new(scale)?);
    state.tray = Some(TrayIcon::new(hwnd)?);
    state.draw(hwnd)?;
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
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
    match message {
        WM_NCHITTEST => LRESULT(HTCAPTION as isize),
        WM_TIMER if wparam.0 == TIMER_ID => {
            if let Some(state) = unsafe { app_state(hwnd) }
                && let Err(error) = state.draw(hwnd)
            {
                show_error(&error);
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }
        tray::CALLBACK_MESSAGE => {
            tray::handle_callback(hwnd, lparam);
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                let _ = KillTimer(Some(hwnd), TIMER_ID);
            }
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_NCDESTROY => {
            let pointer = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) } as *mut AppState;
            if !pointer.is_null() {
                unsafe {
                    drop(Box::from_raw(pointer));
                }
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe fn app_state(hwnd: HWND) -> Option<&'static mut AppState> {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut AppState;
    unsafe { pointer.as_mut() }
}

fn show_error(error: &Error) {
    let text: Vec<u16> = format!("红绿灯启动失败：{error}\0")
        .encode_utf16()
        .collect();
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Vibe Pulse"),
            MB_OK | MB_ICONERROR,
        );
    }
}
