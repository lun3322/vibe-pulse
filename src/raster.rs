#[derive(Clone, Copy)]
pub(crate) struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Copy)]
pub(crate) struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Canvas {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>,
    pub(crate) scale: f32,
}

impl Canvas {
    pub fn new(width: i32, height: i32, scale: f32) -> Self {
        Self {
            width,
            height,
            scale,
            pixels: vec![0; (width * height) as usize],
        }
    }

    pub fn resize(&mut self, height: i32) {
        self.height = height;
        self.pixels.resize((self.width * height) as usize, 0);
    }

    pub(crate) fn draw_rounded_rect(
        &mut self,
        rect: Rect,
        radius: f32,
        top: Rgb,
        bottom: Rgb,
        opacity: f32,
    ) {
        let bounds = self.bounds(
            rect.x - 1.0,
            rect.y - 1.0,
            rect.x + rect.width + 1.0,
            rect.y + rect.height + 1.0,
        );
        for y in bounds.1..bounds.3 {
            let mix = ((y as f32 - rect.y) / rect.height).clamp(0.0, 1.0);
            let color = mix_color(top, bottom, mix);
            for x in bounds.0..bounds.2 {
                let distance = rounded_rect_distance(x as f32 + 0.5, y as f32 + 0.5, rect, radius);
                self.blend(x, y, color, opacity * (0.7 - distance).clamp(0.0, 1.0));
            }
        }
    }

    pub(crate) fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, opacity: f32) {
        self.draw_ellipse(cx, cy, radius, radius, color, opacity);
    }

    pub(crate) fn draw_ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        color: Rgb,
        opacity: f32,
    ) {
        let bounds = self.bounds(cx - rx, cy - ry, cx + rx, cy + ry);
        for y in bounds.1..bounds.3 {
            for x in bounds.0..bounds.2 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                let coverage =
                    ((1.0 - (dx * dx + dy * dy).sqrt()) * rx.min(ry) + 0.7).clamp(0.0, 1.0);
                self.blend(x, y, color, opacity * coverage);
            }
        }
    }

    pub(crate) fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Rgb, width: f32) {
        let steps = (x2 - x1).abs().max((y2 - y1).abs()).ceil() as i32;
        for step in 0..=steps {
            let amount = step as f32 / steps.max(1) as f32;
            self.draw_circle(
                x1 + (x2 - x1) * amount,
                y1 + (y2 - y1) * amount,
                width,
                color,
                1.0,
            );
        }
    }

    pub(crate) fn blend(&mut self, x: i32, y: i32, color: Rgb, opacity: f32) {
        if opacity <= 0.0 {
            return;
        }
        let index = (y * self.width + x) as usize;
        let destination = self.pixels[index];
        let source_alpha = opacity.clamp(0.0, 1.0);
        let inverse = 1.0 - source_alpha;
        let destination_alpha = ((destination >> 24) & 0xff) as f32 / 255.0;
        let red = color.0 as f32 * source_alpha + ((destination >> 16) & 0xff) as f32 * inverse;
        let green = color.1 as f32 * source_alpha + ((destination >> 8) & 0xff) as f32 * inverse;
        let blue = color.2 as f32 * source_alpha + (destination & 0xff) as f32 * inverse;
        let alpha = (source_alpha + destination_alpha * inverse) * 255.0;
        self.pixels[index] =
            ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32;
    }

    pub(crate) fn bounds(
        &self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) -> (i32, i32, i32, i32) {
        (
            left.floor().max(0.0) as i32,
            top.floor().max(0.0) as i32,
            right.ceil().min(self.width as f32) as i32,
            bottom.ceil().min(self.height as f32) as i32,
        )
    }
}

pub(crate) fn scale_color(color: Rgb, brightness: f32) -> Rgb {
    Rgb(
        (color.0 as f32 * brightness).clamp(0.0, 255.0) as u8,
        (color.1 as f32 * brightness).clamp(0.0, 255.0) as u8,
        (color.2 as f32 * brightness).clamp(0.0, 255.0) as u8,
    )
}

fn rounded_rect_distance(px: f32, py: f32, rect: Rect, radius: f32) -> f32 {
    let dx = (px - (rect.x + rect.width * 0.5)).abs() - (rect.width * 0.5 - radius);
    let dy = (py - (rect.y + rect.height * 0.5)).abs() - (rect.height * 0.5 - radius);
    dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius
}

fn mix_color(start: Rgb, end: Rgb, amount: f32) -> Rgb {
    Rgb(
        (start.0 as f32 + (end.0 as f32 - start.0 as f32) * amount) as u8,
        (start.1 as f32 + (end.1 as f32 - start.1 as f32) * amount) as u8,
        (start.2 as f32 + (end.2 as f32 - start.2 as f32) * amount) as u8,
    )
}
