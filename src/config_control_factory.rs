use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        Graphics::Gdi::HFONT,
        UI::WindowsAndMessaging::{
            BS_OWNERDRAW, CBS_DROPDOWNLIST, CreateWindowExW, HMENU, SendMessageW, WINDOW_EX_STYLE,
            WINDOW_STYLE, WM_SETFONT, WS_BORDER, WS_CHILD, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
        },
    },
    core::{PCWSTR, Result, w},
};

use crate::{
    config_combo_box,
    config_controls::{
        ADDRESS_ID, ADDRESS_TOP, CLOSE_ID, CLOSE_LEFT, COPY_CLAUDE_ID, COPY_QODER_ID, COPY_TOP,
        ConfigControls, FIELD_LEFT, FIELD_WIDTH, HOST_ID, HOST_TOP, LAN_ID, LAN_TOP,
    },
};

const FIELD_HEIGHT: i32 = 32;
const COMBO_HEIGHT: i32 = 200;
const LAN_HEIGHT: i32 = 36;
const COPY_LEFT: i32 = 36;
const COPY_SECOND_LEFT: i32 = 360;
const COPY_WIDTH: i32 = 304;
const COPY_HEIGHT: i32 = 46;
const CLOSE_TOP: i32 = 12;
const CLOSE_WIDTH: i32 = 34;
const CLOSE_HEIGHT: i32 = 32;

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

struct ButtonSpec<'a> {
    text: &'a str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
}

pub fn create(parent: HWND, font: HFONT) -> Result<ConfigControls> {
    let address = create_address(parent, font)?;
    let host = create_host(parent, font)?;
    let lan = create_owner_button(
        parent,
        font,
        ButtonSpec {
            text: "访问范围",
            x: FIELD_LEFT,
            y: LAN_TOP,
            width: FIELD_WIDTH,
            height: LAN_HEIGHT,
            id: LAN_ID,
        },
    )?;
    create_owner_button(
        parent,
        font,
        ButtonSpec {
            text: "关闭",
            x: CLOSE_LEFT,
            y: CLOSE_TOP,
            width: CLOSE_WIDTH,
            height: CLOSE_HEIGHT,
            id: CLOSE_ID,
        },
    )?;
    create_copy_buttons(parent, font)?;
    Ok(ConfigControls { address, host, lan })
}

fn create_address(parent: HWND, font: HFONT) -> Result<HWND> {
    let address = create_control(
        parent,
        font,
        ControlSpec {
            class: w!("COMBOBOX"),
            text: "",
            style: WINDOW_STYLE(WS_TABSTOP.0 | WS_VSCROLL.0 | CBS_DROPDOWNLIST as u32),
            x: FIELD_LEFT,
            y: ADDRESS_TOP,
            width: FIELD_WIDTH,
            height: COMBO_HEIGHT,
            id: ADDRESS_ID,
        },
    )?;
    config_combo_box::install(address, font)?;
    Ok(address)
}

fn create_host(parent: HWND, font: HFONT) -> Result<HWND> {
    let host_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "windows-client".to_owned());
    create_control(
        parent,
        font,
        ControlSpec {
            class: w!("EDIT"),
            text: &host_name,
            style: WS_BORDER | WS_TABSTOP,
            x: FIELD_LEFT,
            y: HOST_TOP,
            width: FIELD_WIDTH,
            height: FIELD_HEIGHT,
            id: HOST_ID,
        },
    )
}

fn create_copy_buttons(parent: HWND, font: HFONT) -> Result<()> {
    for spec in [
        ButtonSpec {
            text: "复制 Qoder 配置",
            x: COPY_LEFT,
            y: COPY_TOP,
            width: COPY_WIDTH,
            height: COPY_HEIGHT,
            id: COPY_QODER_ID,
        },
        ButtonSpec {
            text: "复制 Claude Code 配置",
            x: COPY_SECOND_LEFT,
            y: COPY_TOP,
            width: COPY_WIDTH,
            height: COPY_HEIGHT,
            id: COPY_CLAUDE_ID,
        },
    ] {
        create_owner_button(parent, font, spec)?;
    }
    Ok(())
}

fn create_owner_button(parent: HWND, font: HFONT, spec: ButtonSpec<'_>) -> Result<HWND> {
    create_control(
        parent,
        font,
        ControlSpec {
            class: w!("BUTTON"),
            text: spec.text,
            style: WINDOW_STYLE(WS_TABSTOP.0 | BS_OWNERDRAW as u32),
            x: spec.x,
            y: spec.y,
            width: spec.width,
            height: spec.height,
            id: spec.id,
        },
    )
}

fn create_control(parent: HWND, font: HFONT, spec: ControlSpec<'_>) -> Result<HWND> {
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
    unsafe {
        SendMessageW(
            hwnd,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
    Ok(hwnd)
}
