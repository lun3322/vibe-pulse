use windows::Win32::{
    Foundation::{COLORREF, HWND, POINT, RECT},
    Graphics::Gdi::{
        BeginPaint, DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE, EndPaint, HDC, HFONT, PAINTSTRUCT,
    },
    UI::WindowsAndMessaging::GetClientRect,
};

use crate::{
    config_controls::{FIELD_LEFT, WINDOW_WIDTH},
    config_paint_primitives::{self as primitive, CircleSpec, RoundRectSpec, TextSpec},
};

pub const PANEL: COLORREF = rgb(23, 27, 28);
pub const INPUT: COLORREF = rgb(13, 17, 18);
pub const TEXT: COLORREF = rgb(229, 234, 234);
pub const BACKGROUND: COLORREF = rgb(9, 12, 13);
pub const PANEL_EDGE: COLORREF = rgb(55, 63, 64);
pub const MUTED: COLORREF = rgb(142, 153, 155);
pub const GREEN: COLORREF = rgb(0, 214, 100);
pub const ORANGE: COLORREF = rgb(237, 120, 55);
const RED: COLORREF = rgb(255, 54, 69);
const YELLOW: COLORREF = rgb(255, 185, 0);
const LABEL_LEFT: i32 = 40;
const LABEL_RIGHT: i32 = 150;
const CONTENT_RIGHT: i32 = 656;

pub struct WindowPaint<'a> {
    pub hwnd: HWND,
    pub token_fingerprint: &'a str,
    pub title_font: HFONT,
    pub body_font: HFONT,
}

struct LabelSpec<'a> {
    value: &'a str,
    top: i32,
    font: HFONT,
}

pub unsafe fn paint_window(context: WindowPaint<'_>) {
    let mut paint = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(context.hwnd, &mut paint) };
    let mut client = RECT::default();
    unsafe {
        let _ = GetClientRect(context.hwnd, &mut client);
        primitive::fill(dc, client, BACKGROUND);
        draw_window_edge(dc, client);
        draw_panel(
            dc,
            RECT {
                left: 20,
                top: 76,
                right: client.right - 20,
                bottom: 250,
            },
        );
        draw_panel(
            dc,
            RECT {
                left: 20,
                top: 262,
                right: client.right - 20,
                bottom: 360,
            },
        );
        draw_titlebar(dc, context.title_font);
        draw_service_labels(dc, context.body_font);
        draw_authorization(dc, context.body_font, context.token_fingerprint);
        let _ = EndPaint(context.hwnd, &paint);
    }
}

unsafe fn draw_titlebar(dc: HDC, font: HFONT) {
    unsafe {
        draw_signal_icon(dc);
        primitive::text(
            dc,
            TextSpec {
                value: "VIBE PULSE",
                rect: RECT {
                    left: 70,
                    top: 10,
                    right: 260,
                    bottom: 32,
                },
                color: GREEN,
                font,
                flags: DT_SINGLELINE | DT_NOPREFIX,
            },
        );
        primitive::text(
            dc,
            TextSpec {
                value: "HOOK BRIDGE / CONFIGURATION",
                rect: RECT {
                    left: 70,
                    top: 34,
                    right: 520,
                    bottom: 61,
                },
                color: MUTED,
                font,
                flags: DT_SINGLELINE | DT_NOPREFIX,
            },
        );
        primitive::fill(
            dc,
            RECT {
                left: 20,
                top: 67,
                right: WINDOW_WIDTH - 20,
                bottom: 68,
            },
            PANEL_EDGE,
        );
    }
}

unsafe fn draw_signal_icon(dc: HDC) {
    unsafe {
        primitive::round_rect(
            dc,
            RoundRectSpec {
                rect: RECT {
                    left: 22,
                    top: 16,
                    right: 60,
                    bottom: 43,
                },
                radius: 10,
                fill: PANEL,
                edge: PANEL_EDGE,
            },
        );
        draw_light(dc, POINT { x: 32, y: 29 }, RED);
        draw_light(dc, POINT { x: 41, y: 29 }, YELLOW);
        draw_light(dc, POINT { x: 50, y: 29 }, GREEN);
    }
}

unsafe fn draw_light(dc: HDC, center: POINT, color: COLORREF) {
    unsafe {
        primitive::circle(
            dc,
            CircleSpec {
                center,
                radius: 5,
                color,
            },
        )
    };
}

unsafe fn draw_service_labels(dc: HDC, font: HFONT) {
    unsafe {
        draw_label(
            dc,
            LabelSpec {
                value: "服务地址",
                top: 92,
                font,
            },
        );
        draw_label(
            dc,
            LabelSpec {
                value: "来源主机",
                top: 138,
                font,
            },
        );
        primitive::text(
            dc,
            TextSpec {
                value: "用于区分发出 Hook 的电脑或 WSL 实例，并显示在悬停信息中。",
                rect: RECT {
                    left: FIELD_LEFT,
                    top: 167,
                    right: CONTENT_RIGHT,
                    bottom: 187,
                },
                color: MUTED,
                font,
                flags: DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            },
        );
        draw_label(
            dc,
            LabelSpec {
                value: "访问范围",
                top: 203,
                font,
            },
        );
    }
}

unsafe fn draw_authorization(dc: HDC, font: HFONT, token_fingerprint: &str) {
    unsafe {
        primitive::text(
            dc,
            TextSpec {
                value: "AUTHORIZATION",
                rect: RECT {
                    left: LABEL_LEFT,
                    top: 277,
                    right: 210,
                    bottom: 298,
                },
                color: ORANGE,
                font,
                flags: DT_SINGLELINE | DT_NOPREFIX,
            },
        );
        primitive::text(
            dc,
            TextSpec {
                value: "首次运行生成 256 位随机令牌并持久保存；打开窗口不会重新生成。",
                rect: RECT {
                    left: LABEL_LEFT,
                    top: 307,
                    right: CONTENT_RIGHT,
                    bottom: 327,
                },
                color: TEXT,
                font,
                flags: DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            },
        );
        primitive::text(
            dc,
            TextSpec {
                value: token_fingerprint,
                rect: RECT {
                    left: LABEL_LEFT,
                    top: 329,
                    right: CONTENT_RIGHT,
                    bottom: 346,
                },
                color: MUTED,
                font,
                flags: DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            },
        );
    }
}

unsafe fn draw_label(dc: HDC, spec: LabelSpec<'_>) {
    unsafe {
        primitive::text(
            dc,
            TextSpec {
                value: spec.value,
                rect: RECT {
                    left: LABEL_LEFT,
                    top: spec.top,
                    right: LABEL_RIGHT,
                    bottom: spec.top + 22,
                },
                color: MUTED,
                font: spec.font,
                flags: DT_SINGLELINE | DT_NOPREFIX,
            },
        );
    }
}

unsafe fn draw_window_edge(dc: HDC, client: RECT) {
    unsafe {
        primitive::round_rect(
            dc,
            RoundRectSpec {
                rect: client,
                radius: 1,
                fill: BACKGROUND,
                edge: PANEL_EDGE,
            },
        )
    };
}

unsafe fn draw_panel(dc: HDC, rect: RECT) {
    unsafe {
        primitive::round_rect(
            dc,
            RoundRectSpec {
                rect,
                radius: 18,
                fill: PANEL,
                edge: PANEL_EDGE,
            },
        )
    };
}

pub const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF(red as u32 | ((green as u32) << 8) | ((blue as u32) << 16))
}
