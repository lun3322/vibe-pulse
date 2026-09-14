use std::mem::size_of;

use windows::{
    Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
        },
        UI::WindowsAndMessaging::{
            GetWindowRect, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
        },
    },
    core::{Error, Result},
};

pub fn ensure_visible(hwnd: HWND) -> Result<()> {
    let window = window_rect(hwnd)?;
    let work_area = monitor_work_area(hwnd)?;
    let (x, y) = fit_rect(window, work_area);
    if x == window.left && y == window.top {
        return Ok(());
    }
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
    }
    Ok(())
}

pub fn fit_axis(position: i32, size: i32, minimum: i32, maximum: i32) -> i32 {
    position.clamp(minimum, (maximum - size).max(minimum))
}

fn fit_rect(window: RECT, work_area: RECT) -> (i32, i32) {
    let width = window.right - window.left;
    let height = window.bottom - window.top;
    (
        fit_axis(window.left, width, work_area.left, work_area.right),
        fit_axis(window.top, height, work_area.top, work_area.bottom),
    )
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

#[cfg(test)]
mod tests {
    use super::fit_rect;
    use windows::Win32::Foundation::RECT;

    #[test]
    fn keeps_visible_window_position() {
        let window = RECT {
            left: 100,
            top: 120,
            right: 300,
            bottom: 320,
        };
        let work_area = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };

        assert_eq!(fit_rect(window, work_area), (100, 120));
    }

    #[test]
    fn moves_window_inside_nearest_work_area() {
        let window = RECT {
            left: 1900,
            top: 1000,
            right: 2100,
            bottom: 1200,
        };
        let work_area = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };

        assert_eq!(fit_rect(window, work_area), (1720, 840));
    }

    #[test]
    fn supports_negative_monitor_coordinates() {
        let window = RECT {
            left: -2100,
            top: -100,
            right: -1900,
            bottom: 100,
        };
        let work_area = RECT {
            left: -1920,
            top: 0,
            right: 0,
            bottom: 1040,
        };

        assert_eq!(fit_rect(window, work_area), (-1920, 0));
    }

    #[test]
    fn anchors_oversized_window_to_work_area_origin() {
        let window = RECT {
            left: 50,
            top: 50,
            right: 550,
            bottom: 550,
        };
        let work_area = RECT {
            left: 100,
            top: 100,
            right: 400,
            bottom: 400,
        };

        assert_eq!(fit_rect(window, work_area), (100, 100));
    }
}
