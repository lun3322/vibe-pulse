use std::{ffi::c_void, mem::size_of};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        Graphics::{
            Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute},
            Gdi::{
                CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DC_BRUSH, DEFAULT_CHARSET,
                DEFAULT_PITCH, DeleteObject, FF_DONTCARE, FW_NORMAL, FW_SEMIBOLD, GetStockObject,
                HDC, HFONT, OUT_DEFAULT_PRECIS, SetBkColor, SetDCBrushColor, SetTextColor,
            },
        },
        UI::Controls::DRAWITEMSTRUCT,
    },
    core::{Error, HRESULT, Result, w},
};

use crate::{
    config_button_paint,
    config_paint::{self, INPUT, PANEL, TEXT},
};

const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);

pub struct ConfigTheme {
    title_font: HFONT,
    body_font: HFONT,
}

impl ConfigTheme {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let enabled = 1i32;
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                (&enabled as *const i32).cast(),
                size_of::<i32>() as u32,
            )?;
        }
        let title_font = create_font(-19, FW_SEMIBOLD.0 as i32)?;
        let body_font = match create_font(-15, FW_NORMAL.0 as i32) {
            Ok(font) => font,
            Err(error) => {
                unsafe {
                    let _ = DeleteObject(title_font.into());
                }
                return Err(error);
            }
        };
        Ok(Self {
            title_font,
            body_font,
        })
    }

    pub fn body_font(&self) -> HFONT {
        self.body_font
    }

    pub unsafe fn paint(&self, hwnd: HWND, token_fingerprint: &str) {
        unsafe {
            config_paint::paint_window(config_paint::WindowPaint {
                hwnd,
                token_fingerprint,
                title_font: self.title_font,
                body_font: self.body_font,
            })
        };
    }

    pub unsafe fn color_edit(&self, wparam: WPARAM) -> isize {
        unsafe { color_control(wparam, INPUT) }
    }

    pub unsafe fn color_button(&self, wparam: WPARAM) -> isize {
        unsafe { color_control(wparam, PANEL) }
    }

    pub unsafe fn draw_button(&self, lparam: LPARAM, allow_lan: bool) {
        let item = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
        unsafe { config_button_paint::draw(item, self.body_font, allow_lan) };
    }
}

impl Drop for ConfigTheme {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.title_font.into());
            let _ = DeleteObject(self.body_font.into());
        }
    }
}

unsafe fn color_control(wparam: WPARAM, background: windows::Win32::Foundation::COLORREF) -> isize {
    let dc = HDC(wparam.0 as *mut c_void);
    unsafe {
        SetTextColor(dc, TEXT);
        SetBkColor(dc, background);
        SetDCBrushColor(dc, background);
        GetStockObject(DC_BRUSH).0 as isize
    }
}

fn create_font(height: i32, weight: i32) -> Result<HFONT> {
    let font = unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
            w!("Segoe UI Variable Text"),
        )
    };
    if font.0.is_null() {
        Err(Error::new(APP_FAILURE, "无法创建配置窗口字体"))
    } else {
        Ok(font)
    }
}
