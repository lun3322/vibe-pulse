use windows::Win32::{
    Foundation::POINT,
    Graphics::Gdi::{DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HFONT},
    UI::Controls::{DRAWITEMSTRUCT, ODS_SELECTED},
};

use crate::{
    config_controls::{CLOSE_ID, COPY_CLAUDE_ID},
    config_paint::{BACKGROUND, GREEN, MUTED, ORANGE, PANEL, rgb},
    config_paint_primitives::{self as primitive, LineSpec, RoundRectSpec, TextSpec},
};

pub unsafe fn draw(item: &DRAWITEMSTRUCT, font: HFONT) {
    match item.CtlID as isize {
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
