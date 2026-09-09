use windows::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::{DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HFONT},
    UI::Controls::{DRAWITEMSTRUCT, ODS_SELECTED},
};

use crate::{
    config_controls::{CLOSE_ID, COPY_CLAUDE_ID, LAN_ID},
    config_paint::{BACKGROUND, GREEN, INPUT, MUTED, ORANGE, PANEL, PANEL_EDGE, TEXT, rgb},
    config_paint_primitives::{self as primitive, CircleSpec, LineSpec, RoundRectSpec, TextSpec},
};

pub unsafe fn draw(item: &DRAWITEMSTRUCT, font: HFONT, allow_lan: bool) {
    match item.CtlID as isize {
        LAN_ID => unsafe { draw_scope(item, font, allow_lan) },
        CLOSE_ID => unsafe { draw_close(item) },
        _ => unsafe { draw_copy(item, font) },
    }
}

unsafe fn draw_copy(item: &DRAWITEMSTRUCT, font: HFONT) {
    let claude = item.CtlID == COPY_CLAUDE_ID as u32;
    let accent = if claude { ORANGE } else { GREEN };
    let label = if claude {
        "复制 Claude Code 配置"
    } else {
        "复制 Qoder 配置"
    };
    let fill = if selected(item) {
        rgb(37, 44, 45)
    } else {
        PANEL
    };
    unsafe {
        primitive::round_rect(
            item.hDC,
            RoundRectSpec {
                rect: item.rcItem,
                radius: 12,
                fill,
                edge: accent,
            },
        );
        primitive::text(
            item.hDC,
            TextSpec {
                value: label,
                rect: item.rcItem,
                color: accent,
                font,
                flags: DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOPREFIX,
            },
        );
    }
}

unsafe fn draw_scope(item: &DRAWITEMSTRUCT, font: HFONT, allow_lan: bool) {
    let fill = if selected(item) {
        rgb(30, 36, 37)
    } else {
        INPUT
    };
    let switch_color = if allow_lan { GREEN } else { PANEL_EDGE };
    let knob_x = if allow_lan { 42 } else { 20 };
    unsafe {
        primitive::round_rect(
            item.hDC,
            RoundRectSpec {
                rect: item.rcItem,
                radius: 10,
                fill,
                edge: PANEL_EDGE,
            },
        );
        primitive::round_rect(
            item.hDC,
            RoundRectSpec {
                rect: RECT {
                    left: 10,
                    top: 8,
                    right: 56,
                    bottom: 28,
                },
                radius: 18,
                fill: switch_color,
                edge: switch_color,
            },
        );
        primitive::circle(
            item.hDC,
            CircleSpec {
                center: POINT { x: knob_x, y: 18 },
                radius: 8,
                color: TEXT,
            },
        );
        primitive::text(
            item.hDC,
            TextSpec {
                value: "允许 WSL2 / 局域网接入（重启软件后生效）",
                rect: RECT {
                    left: 72,
                    top: 0,
                    right: item.rcItem.right - 12,
                    bottom: item.rcItem.bottom,
                },
                color: TEXT,
                font,
                flags: DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            },
        );
    }
}

unsafe fn draw_close(item: &DRAWITEMSTRUCT) {
    let fill = if selected(item) {
        rgb(78, 32, 35)
    } else {
        BACKGROUND
    };
    unsafe {
        primitive::fill(item.hDC, item.rcItem, fill);
        primitive::line(
            item.hDC,
            LineSpec {
                start: POINT { x: 11, y: 10 },
                end: POINT { x: 23, y: 22 },
                width: 2,
                color: MUTED,
            },
        );
        primitive::line(
            item.hDC,
            LineSpec {
                start: POINT { x: 23, y: 10 },
                end: POINT { x: 11, y: 22 },
                width: 2,
                color: MUTED,
            },
        );
    }
}

fn selected(item: &DRAWITEMSTRUCT) -> bool {
    item.itemState.0 & ODS_SELECTED.0 != 0
}
