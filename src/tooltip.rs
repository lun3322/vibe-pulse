use windows::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, WPARAM},
        Graphics::Gdi::{CreateRoundRectRgn, DeleteObject, InvalidateRect, SetWindowRgn},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetCursorPos, IDC_ARROW,
            LoadCursorW, RegisterClassW, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            SetWindowLongPtrW, SetWindowPos, ShowWindow, WINDOW_EX_STYLE, WM_ERASEBKGND, WM_PAINT,
            WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    },
    core::{Error, HRESULT, PCWSTR, Result, w},
};

use crate::{
    model::Session,
    tooltip_content::TooltipContent,
    tooltip_paint::{CORNER_RADIUS, HEIGHT, WIDTH},
};

const CLASS_NAME: PCWSTR = w!("VibePulseTooltipWindow");
const CURSOR_OFFSET_X: i32 = 16;
const CURSOR_OFFSET_Y: i32 = 20;
const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);

pub struct Tooltip {
    hwnd: HWND,
    content: Box<TooltipContent>,
}

impl Tooltip {
    pub fn new(owner: HWND) -> Result<Self> {
        register_class()?;
        let mut content = Box::new(TooltipContent::new()?);
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0),
                CLASS_NAME,
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                WIDTH,
                HEIGHT,
                Some(owner),
                None,
                Some(GetModuleHandleW(None)?.into()),
                None,
            )?
        };
        unsafe {
            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                content.as_mut() as *mut TooltipContent as isize,
            );
        }
        if let Err(error) = set_rounded_region(hwnd) {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return Err(error);
        }
        Ok(Self { hwnd, content })
    }

    pub fn show(&mut self, session: &Session, now: std::time::Instant) {
        self.content.update(session, now);
        let mut cursor = POINT::default();
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, false);
            let _ = GetCursorPos(&mut cursor);
            let _ = SetWindowPos(
                self.hwnd,
                None,
                cursor.x + CURSOR_OFFSET_X,
                cursor.y + CURSOR_OFFSET_Y,
                WIDTH,
                HEIGHT,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

impl Drop for Tooltip {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.hwnd);
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
        WM_PAINT => {
            if let Some(content) = unsafe { tooltip_content(hwnd) } {
                unsafe { crate::tooltip_paint::paint(hwnd, content) };
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn register_class() -> Result<()> {
    let instance = unsafe { GetModuleHandleW(None)? };
    let class = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        hInstance: instance.into(),
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

fn set_rounded_region(hwnd: HWND) -> Result<()> {
    let region =
        unsafe { CreateRoundRectRgn(0, 0, WIDTH + 1, HEIGHT + 1, CORNER_RADIUS, CORNER_RADIUS) };
    if region.0.is_null() {
        return Err(Error::new(APP_FAILURE, "无法创建提示卡圆角区域"));
    }
    if unsafe { SetWindowRgn(hwnd, Some(region), true) } == 0 {
        unsafe {
            let _ = DeleteObject(region.into());
        }
        return Err(Error::new(APP_FAILURE, "无法应用提示卡圆角区域"));
    }
    Ok(())
}

unsafe fn tooltip_content(hwnd: HWND) -> Option<&'static TooltipContent> {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const TooltipContent;
    unsafe { pointer.as_ref() }
}
