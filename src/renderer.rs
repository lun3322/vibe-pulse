use std::{ffi::c_void, mem::size_of, ptr::copy_nonoverlapping};

use crate::{
    drawing::{BASE_WIDTH, Canvas, GROUP_HEIGHT},
    model::Session,
};
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, POINT, SIZE},
        Graphics::Gdi::{
            AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
            CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC,
            HBITMAP, HDC, HGDIOBJ, ReleaseDC, SelectObject,
        },
        UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow},
    },
    core::{Error, Result},
};

pub struct LayeredSurface {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u32,
    canvas: Canvas,
    scale: f32,
}

impl LayeredSurface {
    pub fn new(scale: f32) -> Result<Self> {
        let (width, height) = pixel_size(1, scale);
        unsafe {
            let dc = CreateCompatibleDC(None);
            if dc.0.is_null() {
                return Err(Error::from_thread());
            }
            let (bitmap, bits) = match create_bitmap(dc, width, height) {
                Ok(bitmap) => bitmap,
                Err(error) => {
                    let _ = DeleteDC(dc);
                    return Err(error);
                }
            };
            let previous = SelectObject(dc, bitmap.into());
            Ok(Self {
                dc,
                bitmap,
                previous,
                bits,
                canvas: Canvas::new(width, height, scale),
                scale,
            })
        }
    }

    pub fn render(
        &mut self,
        sessions: &[Session],
        hovered: Option<usize>,
        elapsed_seconds: f32,
    ) -> Result<()> {
        self.resize(sessions.len())?;
        self.canvas.render(sessions, hovered, elapsed_seconds);
        unsafe {
            copy_nonoverlapping(
                self.canvas.pixels.as_ptr(),
                self.bits,
                self.canvas.pixels.len(),
            );
        }
        Ok(())
    }

    pub fn present(&self, hwnd: HWND) -> Result<()> {
        let source = POINT::default();
        let size = SIZE {
            cx: self.canvas.width,
            cy: self.canvas.height,
        };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        unsafe {
            let screen = GetDC(None);
            if screen.0.is_null() {
                return Err(Error::from_thread());
            }
            let result = UpdateLayeredWindow(
                hwnd,
                Some(screen),
                None,
                Some(&size),
                Some(self.dc),
                Some(&source),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            ReleaseDC(None, screen);
            result
        }
    }

    fn resize(&mut self, group_count: usize) -> Result<()> {
        let (width, height) = pixel_size(group_count, self.scale);
        if self.canvas.height == height {
            return Ok(());
        }
        unsafe {
            let (bitmap, bits) = create_bitmap(self.dc, width, height)?;
            let old_bitmap = SelectObject(self.dc, bitmap.into());
            let _ = DeleteObject(old_bitmap);
            self.bitmap = bitmap;
            self.bits = bits;
            self.canvas.resize(height);
        }
        Ok(())
    }
}

impl Drop for LayeredSurface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.dc);
        }
    }
}

pub fn pixel_size(group_count: usize, scale: f32) -> (i32, i32) {
    let width = (BASE_WIDTH as f32 * scale).round() as i32;
    let height = (GROUP_HEIGHT as f32 * scale * group_count.max(1) as f32).round() as i32;
    (width, height)
}

unsafe fn create_bitmap(dc: HDC, width: i32, height: i32) -> Result<(HBITMAP, *mut u32)> {
    let info = bitmap_info(width, height);
    let mut bits = std::ptr::null_mut::<c_void>();
    let bitmap = unsafe { CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)? };
    Ok((bitmap, bits.cast()))
}

fn bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    }
}
