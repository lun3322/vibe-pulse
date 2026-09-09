use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{HFONT, InvalidateRect},
        UI::{
            Input::KeyboardAndMouse::EnableWindow,
            WindowsAndMessaging::{
                CB_ADDSTRING, CB_GETCURSEL, CB_GETLBTEXT, CB_GETLBTEXTLEN, CB_RESETCONTENT,
                CB_SETCURSEL, GetWindowTextLengthW, GetWindowTextW, SendMessageW,
            },
        },
    },
    core::{Error, HRESULT, Result},
};

use crate::config_control_factory;

pub const ADDRESS_ID: isize = 101;
pub const HOST_ID: isize = 102;
pub const LAN_ID: isize = 103;
pub const COPY_QODER_ID: isize = 104;
pub const COPY_CLAUDE_ID: isize = 105;
pub const CLOSE_ID: isize = 106;
pub const WINDOW_WIDTH: i32 = 700;
pub const WINDOW_HEIGHT: i32 = 432;
pub const TITLEBAR_HEIGHT: i32 = 68;
pub const CLOSE_LEFT: i32 = 650;
pub const FIELD_LEFT: i32 = 166;
pub const FIELD_WIDTH: i32 = 490;
pub const ADDRESS_TOP: i32 = 86;
pub const HOST_TOP: i32 = 132;
pub const LAN_TOP: i32 = 196;
pub const COPY_TOP: i32 = 368;

const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);
const LOCAL_ADDRESS: &str = "127.0.0.1";

#[derive(Default)]
pub struct ConfigControls {
    pub address: HWND,
    pub host: HWND,
    pub lan: HWND,
}

pub struct ConfigControlInit<'a> {
    pub parent: HWND,
    pub allow_lan: bool,
    pub selected_address: &'a str,
    pub private_addresses: &'a [String],
    pub font: HFONT,
}

pub struct ScopeSelection<'a> {
    pub allow_lan: bool,
    pub selected_address: &'a str,
    pub private_addresses: &'a [String],
}

struct ControlMessage {
    id: u32,
    wparam: usize,
    lparam: isize,
}

pub fn create(init: ConfigControlInit<'_>) -> Result<ConfigControls> {
    let controls = config_control_factory::create(init.parent, init.font)?;
    let target = scope_address(
        init.allow_lan,
        init.selected_address,
        init.private_addresses,
    )?;
    apply_scope(
        &controls,
        ScopeSelection {
            allow_lan: init.allow_lan,
            selected_address: &target,
            private_addresses: init.private_addresses,
        },
    );
    Ok(controls)
}

pub fn scope_address(
    allow_lan: bool,
    preferred: &str,
    private_addresses: &[String],
) -> Result<String> {
    if !allow_lan {
        return Ok(LOCAL_ADDRESS.to_owned());
    }
    private_addresses
        .iter()
        .find(|address| address.as_str() == preferred)
        .or_else(|| private_addresses.first())
        .cloned()
        .ok_or_else(|| Error::new(APP_FAILURE, "未检测到可用的内网 IPv4 地址"))
}

pub fn apply_scope(controls: &ConfigControls, selection: ScopeSelection<'_>) {
    reset_combo(controls.address);
    let selected = if selection.allow_lan {
        selection
            .private_addresses
            .iter()
            .for_each(|address| add_combo_item(controls.address, address));
        selection
            .private_addresses
            .iter()
            .position(|address| address == selection.selected_address)
            .unwrap_or_default()
    } else {
        add_combo_item(controls.address, LOCAL_ADDRESS);
        0
    };
    send(
        controls.address,
        ControlMessage {
            id: CB_SETCURSEL,
            wparam: selected,
            lparam: 0,
        },
    );
    unsafe {
        let _ = EnableWindow(controls.address, selection.allow_lan);
        let _ = InvalidateRect(Some(controls.lan), None, true);
    }
}

pub fn selected_address(controls: &ConfigControls) -> Result<String> {
    let index = send(
        controls.address,
        ControlMessage {
            id: CB_GETCURSEL,
            wparam: 0,
            lparam: 0,
        },
    )
    .0;
    if index < 0 {
        return Err(Error::new(APP_FAILURE, "服务地址未选择"));
    }
    let length = send(
        controls.address,
        ControlMessage {
            id: CB_GETLBTEXTLEN,
            wparam: index as usize,
            lparam: 0,
        },
    )
    .0;
    if length < 0 {
        return Err(Error::new(APP_FAILURE, "无法读取服务地址"));
    }
    let mut text = vec![0u16; length as usize + 1];
    send(
        controls.address,
        ControlMessage {
            id: CB_GETLBTEXT,
            wparam: index as usize,
            lparam: text.as_mut_ptr() as isize,
        },
    );
    Ok(String::from_utf16_lossy(&text[..length as usize]))
}

pub fn window_text(hwnd: HWND) -> String {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    let mut text = vec![0u16; length as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut text) };
    String::from_utf16_lossy(&text[..copied as usize])
}

fn reset_combo(hwnd: HWND) {
    send(
        hwnd,
        ControlMessage {
            id: CB_RESETCONTENT,
            wparam: 0,
            lparam: 0,
        },
    );
}

fn add_combo_item(hwnd: HWND, value: &str) {
    let text: Vec<u16> = value.encode_utf16().chain([0]).collect();
    send(
        hwnd,
        ControlMessage {
            id: CB_ADDSTRING,
            wparam: 0,
            lparam: text.as_ptr() as isize,
        },
    );
}

fn send(hwnd: HWND, message: ControlMessage) -> LRESULT {
    unsafe {
        SendMessageW(
            hwnd,
            message.id,
            Some(WPARAM(message.wparam)),
            Some(LPARAM(message.lparam)),
        )
    }
}
