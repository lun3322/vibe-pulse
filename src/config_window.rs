use std::{mem::size_of, ptr::copy_nonoverlapping};

use windows::{
    Win32::{
        Foundation::{
            ERROR_CLASS_ALREADY_EXISTS, GlobalFree, HANDLE, HWND, LPARAM, LRESULT, WPARAM,
        },
        Graphics::Gdi::{COLOR_WINDOW, GetSysColorBrush},
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            LibraryLoader::GetModuleHandleW,
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
        },
        UI::{
            Controls::BST_CHECKED,
            WindowsAndMessaging::{
                BM_GETCHECK, CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DestroyWindow,
                FindWindowW, GWLP_USERDATA, IDC_ARROW, LoadCursorW, MB_ICONERROR,
                MB_ICONINFORMATION, MB_OK, MessageBoxW, PostMessageW, RegisterClassW, SW_SHOW,
                SendMessageW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow, WM_CLOSE,
                WM_COMMAND, WM_CREATE, WM_NCCREATE, WM_NCDESTROY, WNDCLASSW, WS_CAPTION,
                WS_EX_APPWINDOW, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
            },
        },
    },
    core::{Error, HRESULT, PCWSTR, Result, w},
};

use crate::{
    app::LAN_SETTING_MESSAGE,
    config_controls::{self, COPY_CLAUDE_ID, COPY_QODER_ID, ConfigControls},
    hook_config::{endpoint, generate},
    http_server::PORT,
    settings::Settings,
};

const CLASS_NAME: PCWSTR = w!("VibePulseConfigWindow");
const LAN_ID: isize = 103;
const CLIPBOARD_UNICODE_TEXT: u32 = 13;
const INVALID_ARGUMENT: HRESULT = HRESULT(0x80070057_u32 as i32);

struct ConfigInit<'a> {
    owner: HWND,
    settings: &'a Settings,
}

struct ConfigState {
    owner: HWND,
    token: String,
    allow_lan: bool,
    controls: ConfigControls,
}

pub fn show(owner: HWND, settings: &Settings) -> Result<()> {
    if let Ok(window) = unsafe { FindWindowW(CLASS_NAME, PCWSTR::null()) } {
        unsafe {
            let _ = ShowWindow(window, SW_SHOW);
            let _ = SetForegroundWindow(window);
        }
        return Ok(());
    }
    register_class()?;
    let init = ConfigInit { owner, settings };
    unsafe {
        let _ = CreateWindowExW(
            WS_EX_APPWINDOW,
            CLASS_NAME,
            w!("Vibe Pulse 配置"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            240,
            180,
            650,
            340,
            Some(owner),
            None,
            Some(GetModuleHandleW(None)?.into()),
            Some((&init as *const ConfigInit).cast()),
        )?;
    }
    Ok(())
}

fn register_class() -> Result<()> {
    let instance = unsafe { GetModuleHandleW(None)? };
    let class = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        hInstance: instance.into(),
        hbrBackground: unsafe { GetSysColorBrush(COLOR_WINDOW) },
        lpszClassName: CLASS_NAME,
        lpfnWndProc: Some(window_proc),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } != 0 {
        return Ok(());
    }
    let error = Error::from_thread();
    if error.code().0 as u32 == ERROR_CLASS_ALREADY_EXISTS.0 {
        Ok(())
    } else {
        Err(error)
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => unsafe { initialize_state(hwnd, lparam) },
        WM_CREATE => {
            if let Some(state) = unsafe { state(hwnd) } {
                match config_controls::create(hwnd, state.allow_lan) {
                    Ok(controls) => state.controls = controls,
                    Err(error) => {
                        show_error(hwnd, &error);
                        return LRESULT(-1);
                    }
                }
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            if let Some(state) = unsafe { state(hwnd) } {
                handle_command(hwnd, state, wparam.0 & 0xffff);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_NCDESTROY => unsafe { destroy_state(hwnd, message, wparam, lparam) },
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe fn initialize_state(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
    let init = unsafe { &*(create.lpCreateParams as *const ConfigInit) };
    let state = Box::new(ConfigState {
        owner: init.owner,
        token: init.settings.token.clone(),
        allow_lan: init.settings.allow_lan,
        controls: ConfigControls::default(),
    });
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
    }
    LRESULT(1)
}

unsafe fn destroy_state(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let pointer = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) } as *mut ConfigState;
    if !pointer.is_null() {
        unsafe {
            drop(Box::from_raw(pointer));
        }
    }
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

fn handle_command(hwnd: HWND, state: &ConfigState, command: usize) {
    let result = match command as isize {
        COPY_QODER_ID => copy_config(state, "qoder"),
        COPY_CLAUDE_ID => copy_config(state, "claude-code"),
        LAN_ID => update_lan_setting(state),
        _ => return,
    };
    match result {
        Ok(()) if command as isize != LAN_ID => show_info(hwnd, "配置已复制到剪贴板。"),
        Ok(()) => show_info(hwnd, "局域网设置已保存，重启 Vibe Pulse 后生效。"),
        Err(error) => show_error(hwnd, &error),
    }
}

fn copy_config(state: &ConfigState, client: &str) -> Result<()> {
    let address = config_controls::window_text(state.controls.address);
    let host = config_controls::window_text(state.controls.host);
    if address.trim().is_empty() || host.trim().is_empty() {
        return Err(Error::new(INVALID_ARGUMENT, "服务地址和来源主机名不能为空"));
    }
    let config = generate(
        client,
        &endpoint(address.trim(), PORT),
        &state.token,
        host.trim(),
    );
    copy_to_clipboard(state.owner, &config)
}

fn update_lan_setting(state: &ConfigState) -> Result<()> {
    let checked = unsafe {
        SendMessageW(
            state.controls.lan,
            BM_GETCHECK,
            Some(WPARAM(0)),
            Some(LPARAM(0)),
        )
    };
    unsafe {
        PostMessageW(
            Some(state.owner),
            LAN_SETTING_MESSAGE,
            WPARAM((checked.0 as u32 == BST_CHECKED.0) as usize),
            LPARAM(0),
        )
    }
}

fn copy_to_clipboard(owner: HWND, text: &str) -> Result<()> {
    let text: Vec<u16> = text.encode_utf16().chain([0]).collect();
    unsafe {
        let memory = GlobalAlloc(GMEM_MOVEABLE, text.len() * size_of::<u16>())?;
        let target = GlobalLock(memory).cast::<u16>();
        if target.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err(Error::from_thread());
        }
        copy_nonoverlapping(text.as_ptr(), target, text.len());
        let _ = GlobalUnlock(memory);
        if let Err(error) = write_clipboard(owner, HANDLE(memory.0)) {
            let _ = GlobalFree(Some(memory));
            return Err(error);
        }
    }
    Ok(())
}

unsafe fn write_clipboard(owner: HWND, memory: HANDLE) -> Result<()> {
    unsafe { OpenClipboard(Some(owner))? };
    let result = unsafe {
        EmptyClipboard()
            .and_then(|_| SetClipboardData(CLIPBOARD_UNICODE_TEXT, Some(memory)).map(|_| ()))
    };
    let close_result = unsafe { CloseClipboard() };
    result.and(close_result)
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
