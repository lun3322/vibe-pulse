use std::mem::size_of;

use windows::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            CreateRoundRectRgn, DeleteObject, GetMonitorInfoW, InvalidateRect,
            MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, SetWindowRgn,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetWindowRect,
            IDC_ARROW, LoadCursorW, RegisterClassW, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            SetWindowLongPtrW, SetWindowPos, ShowWindow, WINDOW_EX_STYLE, WM_ERASEBKGND, WM_PAINT,
            WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    },
    core::{Error, HRESULT, PCWSTR, Result, w},
};

use crate::{
    drawing::group_top,
    model::Session,
    tooltip_content::TooltipContent,
    tooltip_paint::{CORNER_RADIUS, HEIGHT, WIDTH},
};

const CLASS_NAME: PCWSTR = w!("VibePulseTooltipWindow");
const WINDOW_GAP: i32 = 14;
const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);

struct TooltipPosition {
    x: i32,
    y: i32,
}

pub struct Tooltip {
    hwnd: HWND,
    owner: HWND,
    scale: f32,
    content: Box<TooltipContent>,
}

impl Tooltip {
    pub fn new(owner: HWND, scale: f32) -> Result<Self> {
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
        Ok(Self {
            hwnd,
            owner,
            scale,
            content,
        })
    }

    pub fn show(
        &mut self,
        session: &Session,
        now: std::time::Instant,
        group_index: usize,
    ) -> Result<()> {
        self.content.update(session, now);
        let position = self.position(group_index)?;
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, false);
            SetWindowPos(
                self.hwnd,
                None,
                position.x,
                position.y,
                WIDTH,
                HEIGHT,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        }
        Ok(())
    }

    fn position(&self, group_index: usize) -> Result<TooltipPosition> {
        let owner = window_rect(self.owner)?;
        let work = monitor_work_area(self.owner)?;
        let right = owner.right + WINDOW_GAP;
        let left = owner.left - WINDOW_GAP - WIDTH;
        let preferred_x = if right + WIDTH <= work.right {
            right
        } else {
            left
        };
        Ok(TooltipPosition {
            x: fit_axis(preferred_x, WIDTH, work.left, work.right),
            y: fit_axis(
                owner.top + group_top(group_index, self.scale),
                HEIGHT,
                work.top,
                work.bottom,
            ),
        })
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

fn window_rect(hwnd: HWND) -> Result<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect)? };
    Ok(rect)
}

fn monitor_work_area(hwnd: HWND) -> Result<RECT> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        Ok(info.rcWork)
    } else {
        Err(Error::from_thread())
    }
}

fn fit_axis(position: i32, size: i32, minimum: i32, maximum: i32) -> i32 {
    position.clamp(minimum, (maximum - size).max(minimum))
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
