use windows::Win32::{
    Foundation::{COLORREF, HWND, RECT},
    Graphics::Gdi::{
        BeginPaint, CreatePen, CreateSolidBrush, DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE,
        DT_WORDBREAK, DeleteObject, DrawTextW, EndPaint, FillRect, HFONT, PAINTSTRUCT, PS_SOLID,
        RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    },
};

use crate::tooltip_content::{TooltipContent, rgb};

pub const WIDTH: i32 = 460;
pub const HEIGHT: i32 = 126;
pub const CORNER_RADIUS: i32 = 18;

type Hdc = windows::Win32::Graphics::Gdi::HDC;
type TextFormat = windows::Win32::Graphics::Gdi::DRAW_TEXT_FORMAT;

pub unsafe fn paint(hwnd: HWND, content: &TooltipContent) {
    let mut paint = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(hwnd, &mut paint) };
    unsafe {
        draw_card(dc, content);
        let _ = EndPaint(hwnd, &paint);
    }
}

unsafe fn draw_card(dc: Hdc, content: &TooltipContent) {
    unsafe {
        draw_round_rect(
            dc,
            RECT {
                left: 0,
                top: 0,
                right: WIDTH,
                bottom: HEIGHT,
            },
            CORNER_RADIUS,
            rgb(21, 24, 25),
            rgb(61, 69, 71),
        );
        draw_inner_panel(dc);
        draw_round_rect(
            dc,
            RECT {
                left: 14,
                top: 16,
                right: 19,
                bottom: HEIGHT - 16,
            },
            4,
            content.accent,
            content.accent,
        );
        draw_status_dot(dc, content.status_color);
        draw_content(dc, content);
    }
}

unsafe fn draw_inner_panel(dc: Hdc) {
    unsafe {
        draw_round_rect(
            dc,
            RECT {
                left: 3,
                top: 3,
                right: WIDTH - 3,
                bottom: HEIGHT - 3,
            },
            CORNER_RADIUS - 3,
            rgb(12, 15, 16),
            rgb(33, 39, 40),
        );
        fill(
            dc,
            RECT {
                left: 24,
                top: 5,
                right: WIDTH - 24,
                bottom: 7,
            },
            rgb(73, 82, 84),
        );
        draw_round_rect(
            dc,
            RECT {
                left: 30,
                top: 43,
                right: WIDTH - 18,
                bottom: 44,
            },
            1,
            rgb(43, 50, 51),
            rgb(43, 50, 51),
        );
    }
}

unsafe fn draw_content(dc: Hdc, content: &TooltipContent) {
    unsafe {
        let _ = SetBkMode(dc, TRANSPARENT);
        draw_text(
            dc,
            &content.title,
            RECT {
                left: 31,
                top: 13,
                right: WIDTH - 18,
                bottom: 36,
            },
            content.accent,
            content.header_font,
            DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        draw_text(
            dc,
            &content.cwd,
            RECT {
                left: 31,
                top: 49,
                right: WIDTH - 18,
                bottom: 68,
            },
            rgb(147, 158, 160),
            content.body_font,
            DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        draw_text(
            dc,
            &content.prompt,
            RECT {
                left: 31,
                top: 70,
                right: WIDTH - 18,
                bottom: 99,
            },
            rgb(229, 234, 234),
            content.body_font,
            DT_WORDBREAK | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        draw_text(
            dc,
            &content.status,
            RECT {
                left: 47,
                top: 103,
                right: WIDTH - 18,
                bottom: 121,
            },
            content.status_color,
            content.body_font,
            DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
    }
}

unsafe fn draw_round_rect(
    dc: Hdc,
    rect: RECT,
    radius: i32,
    fill_color: COLORREF,
    border_color: COLORREF,
) {
    unsafe {
        let brush = CreateSolidBrush(fill_color);
        let pen = CreatePen(PS_SOLID, 1, border_color);
        let previous_brush = SelectObject(dc, brush.into());
        let previous_pen = SelectObject(dc, pen.into());
        let _ = RoundRect(
            dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
        SelectObject(dc, previous_pen);
        SelectObject(dc, previous_brush);
        let _ = DeleteObject(pen.into());
        let _ = DeleteObject(brush.into());
    }
}

unsafe fn draw_status_dot(dc: Hdc, color: COLORREF) {
    unsafe {
        draw_round_rect(
            dc,
            RECT {
                left: 31,
                top: 106,
                right: 39,
                bottom: 114,
            },
            8,
            color,
            color,
        );
    }
}

unsafe fn fill(dc: Hdc, rect: RECT, color: COLORREF) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(dc, &rect, brush);
        let _ = DeleteObject(brush.into());
    }
}

unsafe fn draw_text(
    dc: Hdc,
    text: &str,
    mut rect: RECT,
    color: COLORREF,
    font: HFONT,
    format: TextFormat,
) {
    let mut text: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let previous = SelectObject(dc, font.into());
        SetTextColor(dc, color);
        DrawTextW(dc, &mut text, &mut rect, format);
        SelectObject(dc, previous);
    }
}
