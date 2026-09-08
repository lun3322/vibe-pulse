use std::{env, ffi::c_void};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        Graphics::Gdi::{DEFAULT_GUI_FONT, GetStockObject},
        UI::{
            Controls::BST_CHECKED,
            WindowsAndMessaging::{
                BM_SETCHECK, BS_AUTOCHECKBOX, CreateWindowExW, GetWindowTextLengthW,
                GetWindowTextW, HMENU, SendMessageW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_SETFONT,
                WS_BORDER, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::{PCWSTR, Result, w},
};

const ADDRESS_ID: isize = 101;
const HOST_ID: isize = 102;
const LAN_ID: isize = 103;
pub const COPY_QODER_ID: isize = 104;
pub const COPY_CLAUDE_ID: isize = 105;

#[derive(Default)]
pub struct ConfigControls {
    pub address: HWND,
    pub host: HWND,
    pub lan: HWND,
}

struct ControlSpec<'a> {
    class: PCWSTR,
    text: &'a str,
    style: WINDOW_STYLE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
}

pub fn create(parent: HWND, allow_lan: bool) -> Result<ConfigControls> {
    create_label(
        parent,
        "HTTP Hooks 配置仅复制到剪贴板，不会修改任何配置文件。",
        24,
        20,
        590,
    )?;
    let (address, host) = create_identity_controls(parent)?;
    let lan = create_lan_control(parent, allow_lan)?;
    create_action_controls(parent)?;
    Ok(ConfigControls { address, host, lan })
}

fn create_identity_controls(parent: HWND) -> Result<(HWND, HWND)> {
    create_label(parent, "服务地址", 24, 58, 90)?;
    let address = create_control(
        parent,
        ControlSpec {
            class: w!("EDIT"),
            text: "127.0.0.1",
            style: WS_BORDER | WS_TABSTOP,
            x: 118,
            y: 54,
            width: 490,
            height: 26,
            id: ADDRESS_ID,
        },
    )?;
    create_label(parent, "来源主机名", 24, 96, 90)?;
    let host_name = env::var("COMPUTERNAME").unwrap_or_else(|_| "windows-client".to_owned());
    let host = create_control(
        parent,
        ControlSpec {
            class: w!("EDIT"),
            text: &host_name,
            style: WS_BORDER | WS_TABSTOP,
            x: 118,
            y: 92,
            width: 490,
            height: 26,
            id: HOST_ID,
        },
    )?;
    Ok((address, host))
}

fn create_lan_control(parent: HWND, allow_lan: bool) -> Result<HWND> {
    let lan = create_control(
        parent,
        ControlSpec {
            class: w!("BUTTON"),
            text: "允许 WSL2 / 局域网接入（重启软件后生效）",
            style: WINDOW_STYLE(WS_TABSTOP.0 | BS_AUTOCHECKBOX as u32),
            x: 24,
            y: 134,
            width: 430,
            height: 26,
            id: LAN_ID,
        },
    )?;
    if allow_lan {
        send(lan, BM_SETCHECK, BST_CHECKED.0 as usize, 0);
    }
    Ok(lan)
}

fn create_action_controls(parent: HWND) -> Result<()> {
    create_label(
        parent,
        "本机填 127.0.0.1；WSL2 填 Windows 主机地址；其他电脑填本机局域网地址。",
        24,
        174,
        590,
    )?;
    create_control(
        parent,
        ControlSpec {
            class: w!("BUTTON"),
            text: "复制 Qoder 配置",
            style: WS_TABSTOP,
            x: 24,
            y: 218,
            width: 180,
            height: 38,
            id: COPY_QODER_ID,
        },
    )?;
    create_control(
        parent,
        ControlSpec {
            class: w!("BUTTON"),
            text: "复制 Claude Code 配置",
            style: WS_TABSTOP,
            x: 222,
            y: 218,
            width: 210,
            height: 38,
            id: COPY_CLAUDE_ID,
        },
    )?;
    Ok(())
}

pub fn window_text(hwnd: HWND) -> String {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    let mut text = vec![0u16; length as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut text) };
    String::from_utf16_lossy(&text[..copied as usize])
}

fn create_label(parent: HWND, text: &str, x: i32, y: i32, width: i32) -> Result<HWND> {
    create_control(
        parent,
        ControlSpec {
            class: w!("STATIC"),
            text,
            style: WINDOW_STYLE(0),
            x,
            y,
            width,
            height: 24,
            id: 0,
        },
    )
}

fn create_control(parent: HWND, spec: ControlSpec<'_>) -> Result<HWND> {
    let text: Vec<u16> = spec.text.encode_utf16().chain([0]).collect();
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            spec.class,
            PCWSTR(text.as_ptr()),
            WS_CHILD | WS_VISIBLE | spec.style,
            spec.x,
            spec.y,
            spec.width,
            spec.height,
            Some(parent),
            Some(HMENU(spec.id as *mut c_void)),
            None,
            None,
        )?
    };
    let font = unsafe { GetStockObject(DEFAULT_GUI_FONT) };
    send(hwnd, WM_SETFONT, font.0 as usize, 1);
    Ok(hwnd)
}

fn send(hwnd: HWND, message: u32, wparam: usize, lparam: isize) {
    unsafe {
        SendMessageW(hwnd, message, Some(WPARAM(wparam)), Some(LPARAM(lparam)));
    }
}
