use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::HFONT,
        UI::WindowsAndMessaging::{
            CB_ADDSTRING, CB_GETCURSEL, CB_GETLBTEXT, CB_GETLBTEXTLEN, CB_SETCURSEL,
            GetWindowTextLengthW, GetWindowTextW, SendMessageW,
        },
    },
    core::{Error, HRESULT, Result},
};

use crate::config_control_factory;

pub const ADDRESS_ID: isize = 101;
pub const HOST_ID: isize = 102;
pub const COPY_QODER_ID: isize = 104;
pub const COPY_CLAUDE_ID: isize = 105;
pub const CLOSE_ID: isize = 106;
pub const WINDOW_WIDTH: i32 = 700;
pub const WINDOW_HEIGHT: i32 = 380;
pub const TITLEBAR_HEIGHT: i32 = 68;
pub const CLOSE_LEFT: i32 = 650;
pub const FIELD_LEFT: i32 = 166;
pub const FIELD_WIDTH: i32 = 490;
pub const ADDRESS_TOP: i32 = 86;
pub const HOST_TOP: i32 = 132;
pub const COPY_TOP: i32 = 316;

const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);
const LOCAL_HOST: &str = "localhost";

#[derive(Default)]
pub struct ConfigControls {
    pub address: HWND,
    pub host: HWND,
}

pub struct ConfigControlInit<'a> {
    pub parent: HWND,
    pub private_addresses: &'a [String],
    pub font: HFONT,
}

struct ControlMessage {
    id: u32,
    wparam: usize,
    lparam: isize,
}

pub fn create(init: ConfigControlInit<'_>) -> Result<ConfigControls> {
    let controls = config_control_factory::create(init.parent, init.font)?;
    address_options(init.private_addresses)
        .for_each(|address| add_combo_item(controls.address, address));
    send(
        controls.address,
        ControlMessage {
            id: CB_SETCURSEL,
            wparam: 0,
            lparam: 0,
        },
    );
    Ok(controls)
}

fn address_options(private_addresses: &[String]) -> impl Iterator<Item = &str> {
    std::iter::once(LOCAL_HOST).chain(private_addresses.iter().map(String::as_str))
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
        return Err(Error::new(APP_FAILURE, "Hook 地址未选择"));
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
        return Err(Error::new(APP_FAILURE, "无法读取 Hook 地址"));
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

#[cfg(test)]
mod tests {
    use super::address_options;

    #[test]
    fn address_options_start_with_localhost() {
        let private_addresses = vec!["10.0.0.8".to_owned(), "192.168.1.9".to_owned()];

        assert_eq!(
            address_options(&private_addresses).collect::<Vec<_>>(),
            vec!["localhost", "10.0.0.8", "192.168.1.9"]
        );
    }
}
