use windows::Win32::{
    Foundation::{COLORREF, POINT, RECT},
    Graphics::Gdi::{
        CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, Ellipse, FillRect, HDC, HFONT,
        LineTo, MoveToEx, PS_SOLID, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    },
};

pub struct TextSpec<'a> {
    pub value: &'a str,
    pub rect: RECT,
    pub color: COLORREF,
    pub font: HFONT,
    pub flags: windows::Win32::Graphics::Gdi::DRAW_TEXT_FORMAT,
}

pub struct RoundRectSpec {
    pub rect: RECT,
    pub radius: i32,
    pub fill: COLORREF,
    pub edge: COLORREF,
}

pub struct CircleSpec {
    pub center: POINT,
    pub radius: i32,
    pub color: COLORREF,
}

pub struct LineSpec {
    pub start: POINT,
    pub end: POINT,
    pub width: i32,
    pub color: COLORREF,
}

pub unsafe fn fill(dc: HDC, rect: RECT, color: COLORREF) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(dc, &rect, brush);
        let _ = DeleteObject(brush.into());
    }
}

pub unsafe fn circle(dc: HDC, spec: CircleSpec) {
    unsafe {
        let brush = CreateSolidBrush(spec.color);
        let pen = CreatePen(PS_SOLID, 1, spec.color);
        let old_brush = SelectObject(dc, brush.into());
        let old_pen = SelectObject(dc, pen.into());
        let _ = Ellipse(
            dc,
            spec.center.x - spec.radius,
            spec.center.y - spec.radius,
            spec.center.x + spec.radius,
            spec.center.y + spec.radius,
        );
        SelectObject(dc, old_pen);
        SelectObject(dc, old_brush);
        let _ = DeleteObject(pen.into());
        let _ = DeleteObject(brush.into());
    }
}

pub unsafe fn round_rect(dc: HDC, spec: RoundRectSpec) {
    unsafe {
        let brush = CreateSolidBrush(spec.fill);
        let pen = CreatePen(PS_SOLID, 1, spec.edge);
        let old_brush = SelectObject(dc, brush.into());
        let old_pen = SelectObject(dc, pen.into());
        let _ = RoundRect(
            dc,
            spec.rect.left,
            spec.rect.top,
            spec.rect.right,
            spec.rect.bottom,
            spec.radius,
            spec.radius,
        );
        SelectObject(dc, old_pen);
        SelectObject(dc, old_brush);
        let _ = DeleteObject(pen.into());
        let _ = DeleteObject(brush.into());
    }
}

pub unsafe fn line(dc: HDC, spec: LineSpec) {
    unsafe {
        let pen = CreatePen(PS_SOLID, spec.width, spec.color);
        let old_pen = SelectObject(dc, pen.into());
        let mut previous = POINT::default();
        let _ = MoveToEx(dc, spec.start.x, spec.start.y, Some(&mut previous));
        let _ = LineTo(dc, spec.end.x, spec.end.y);
        SelectObject(dc, old_pen);
        let _ = DeleteObject(pen.into());
    }
}

pub unsafe fn text(dc: HDC, mut spec: TextSpec<'_>) {
    let mut value: Vec<u16> = spec.value.encode_utf16().collect();
    unsafe {
        let old_font = SelectObject(dc, spec.font.into());
        let _ = SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, spec.color);
        DrawTextW(dc, &mut value, &mut spec.rect, spec.flags);
        SelectObject(dc, old_font);
    }
}
