use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, EndPaint, HFONT,
            InvalidateRect, PAINTSTRUCT,
        },
        UI::{
            Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled},
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                GetClientRect, GetWindowTextLengthW, GetWindowTextW, WM_ENABLE, WM_ERASEBKGND,
                WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_NCDESTROY, WM_PAINT, WM_SETFOCUS,
            },
        },
    },
    core::{Error, Result},
};

use crate::{
    config_paint::{GREEN, INPUT, MUTED, PANEL, PANEL_EDGE, TEXT},
    config_paint_primitives::{self as primitive, LineSpec, RoundRectSpec, TextSpec},
};

const SUBCLASS_ID: usize = 1;
const ARROW_WIDTH: i32 = 34;
const HORIZONTAL_PADDING: i32 = 8;

pub fn install(hwnd: HWND, font: HFONT) -> Result<()> {
    let installed =
        unsafe { SetWindowSubclass(hwnd, Some(window_proc), SUBCLASS_ID, font.0 as usize) };
    if installed.as_bool() {
        Ok(())
    } else {
        Err(Error::from_thread())
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass_id: usize,
    data: usize,
) -> LRESULT {
    match message {
        WM_PAINT => unsafe { paint(hwnd, HFONT(data as *mut _)) },
        WM_ERASEBKGND => LRESULT(1),
        WM_ENABLE | WM_SETFOCUS | WM_KILLFOCUS | WM_LBUTTONDOWN | WM_LBUTTONUP => {
            let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
            unsafe {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            result
        }
        WM_NCDESTROY => {
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(window_proc), subclass_id);
            }
            unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefSubclassProc(hwnd, message, wparam, lparam) },
    }
}

unsafe fn paint(hwnd: HWND, font: HFONT) -> LRESULT {
    let mut paint = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(hwnd, &mut paint) };
    let mut client = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut client);
        let enabled = IsWindowEnabled(hwnd).as_bool();
        let focused = GetFocus() == hwnd;
        primitive::round_rect(
            dc,
            RoundRectSpec {
                rect: client,
                radius: 4,
                fill: INPUT,
                edge: if focused { GREEN } else { PANEL_EDGE },
            },
        );
        draw_value(dc, client, font, enabled, &window_text(hwnd));
        draw_arrow(dc, client, enabled);
        let _ = EndPaint(hwnd, &paint);
    }
    LRESULT(0)
}

unsafe fn draw_value(
    dc: windows::Win32::Graphics::Gdi::HDC,
    client: RECT,
    font: HFONT,
    enabled: bool,
    value: &str,
) {
    unsafe {
        primitive::text(
            dc,
            TextSpec {
                value,
                rect: RECT {
                    left: HORIZONTAL_PADDING,
                    top: 0,
                    right: client.right - ARROW_WIDTH - HORIZONTAL_PADDING,
                    bottom: client.bottom,
                },
                color: if enabled { TEXT } else { MUTED },
                font,
                flags: DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
            },
        );
    }
}

unsafe fn draw_arrow(dc: windows::Win32::Graphics::Gdi::HDC, client: RECT, enabled: bool) {
    let left = client.right - ARROW_WIDTH;
    let center_x = left + ARROW_WIDTH / 2;
    let center_y = client.bottom / 2;
    let color = if enabled { TEXT } else { MUTED };
    unsafe {
        primitive::fill(
            dc,
            RECT {
                left: left + 1,
                top: 1,
                right: client.right - 1,
                bottom: client.bottom - 1,
            },
            PANEL,
        );
        primitive::line(
            dc,
            LineSpec {
                start: POINT { x: left, y: 1 },
                end: POINT {
                    x: left,
                    y: client.bottom - 1,
                },
                width: 1,
                color: PANEL_EDGE,
            },
        );
        primitive::line(
            dc,
            LineSpec {
                start: POINT {
                    x: center_x - 4,
                    y: center_y - 2,
                },
                end: POINT {
                    x: center_x,
                    y: center_y + 2,
                },
                width: 2,
                color,
            },
        );
        primitive::line(
            dc,
            LineSpec {
                start: POINT {
                    x: center_x,
                    y: center_y + 2,
                },
                end: POINT {
                    x: center_x + 4,
                    y: center_y - 2,
                },
                width: 2,
                color,
            },
        );
    }
}

fn window_text(hwnd: HWND) -> String {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    let mut text = vec![0u16; length as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut text) };
    String::from_utf16_lossy(&text[..copied as usize])
}
