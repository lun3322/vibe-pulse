#[derive(Clone, Copy)]
struct Rgb(u8, u8, u8);

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

const RED: Rgb = Rgb(255, 31, 45);
const YELLOW: Rgb = Rgb(255, 185, 0);
const GREEN: Rgb = Rgb(0, 214, 100);

pub struct Canvas {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>,
    scale: f32,
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

    pub fn render(&mut self, elapsed: f32) {
        self.pixels.fill(0);
        let scale = self.scale;
        let outer = scaled_rect(
            Rect {
                x: 18.0,
                y: 12.0,
                width: 368.0,
                height: 132.0,
            },
            scale,
        );
        self.draw_rounded_rect(outer, 34.0 * scale, Rgb(55, 62, 63), Rgb(21, 24, 25), 1.0);
        let inner = inset(outer, 5.0 * scale);
        self.draw_rounded_rect(inner, 29.0 * scale, Rgb(39, 44, 45), Rgb(9, 11, 12), 1.0);
        let sheen = Rect {
            x: 39.0 * scale,
            y: 23.0 * scale,
            width: 326.0 * scale,
            height: 19.0 * scale,
        };
        self.draw_rounded_rect(
            sheen,
            10.0 * scale,
            Rgb(255, 255, 255),
            Rgb(100, 110, 112),
            0.075,
        );
        for (index, color) in [RED, YELLOW, GREEN].into_iter().enumerate() {
            let center_x = (84.0 + index as f32 * 118.0) * scale;
            self.draw_lamp(
                center_x,
                78.0 * scale,
                45.0 * scale,
                color,
                signal_intensity(elapsed, index),
            );
        }
    }

    fn draw_lamp(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, intensity: f32) {
        if intensity > 0.01 {
            self.draw_glow(
                cx,
                cy,
                radius * 0.78,
                radius * 1.55,
                color,
                intensity * 0.58,
            );
        }
        self.draw_circle(cx, cy, radius + 5.0 * self.scale, Rgb(3, 5, 5), 0.98);
        self.draw_circle(cx, cy, radius + 1.5 * self.scale, Rgb(20, 23, 23), 1.0);
        self.draw_lens(cx, cy, radius, color, intensity);
        self.draw_ellipse(
            cx - radius * 0.19,
            cy - radius * 0.27,
            radius * 0.46,
            radius * 0.25,
            Rgb(255, 255, 255),
            0.10 + intensity * 0.27,
        );
    }

    fn draw_lens(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, intensity: f32) {
        let bounds = self.bounds(cx - radius, cy - radius, cx + radius, cy + radius);
        for y in bounds.1..bounds.3 {
            for x in bounds.0..bounds.2 {
                let dx = (x as f32 + 0.5 - cx) / radius;
                let dy = (y as f32 + 0.5 - cy) / radius;
                let distance = (dx * dx + dy * dy).sqrt();
                let coverage = (radius * (1.0 - distance) + 0.7).clamp(0.0, 1.0);
                if coverage <= 0.0 {
                    continue;
                }
                let edge = (1.0 - distance).clamp(0.0, 1.0).powf(0.45);
                let brightness = 0.18 + intensity * 0.82;
                let directional =
                    (0.72 + 0.28 * (-dx * 0.45 - dy * 0.9 + 0.5).clamp(0.0, 1.0)) * edge;
                self.blend(x, y, scale_color(color, brightness * directional), coverage);
            }
        }
    }

    fn draw_glow(&mut self, cx: f32, cy: f32, inner: f32, outer: f32, color: Rgb, opacity: f32) {
        let bounds = self.bounds(cx - outer, cy - outer, cx + outer, cy + outer);
        for y in bounds.1..bounds.3 {
            for x in bounds.0..bounds.2 {
                let distance =
                    ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let fade = (1.0 - ((distance - inner) / (outer - inner)).clamp(0.0, 1.0)).powi(2);
                self.blend(x, y, color, opacity * fade);
            }
        }
    }

    fn draw_rounded_rect(&mut self, rect: Rect, radius: f32, top: Rgb, bottom: Rgb, opacity: f32) {
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

    fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, opacity: f32) {
        self.draw_ellipse(cx, cy, radius, radius, color, opacity);
    }

    fn draw_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgb, opacity: f32) {
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

    fn blend(&mut self, x: i32, y: i32, color: Rgb, opacity: f32) {
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

    fn bounds(&self, left: f32, top: f32, right: f32, bottom: f32) -> (i32, i32, i32, i32) {
        (
            left.floor().max(0.0) as i32,
            top.floor().max(0.0) as i32,
            right.ceil().min(self.width as f32) as i32,
            bottom.ceil().min(self.height as f32) as i32,
        )
    }
}

fn signal_intensity(elapsed: f32, index: usize) -> f32 {
    let local = (elapsed - index as f32 * 2.0).rem_euclid(6.0);
    if local >= 2.0 {
        return 0.0;
    }
    let fade_in = smoothstep(0.0, 0.16, local);
    let fade_out = 1.0 - smoothstep(1.62, 1.88, local);
    let pulse = 0.88 + 0.12 * (local * std::f32::consts::TAU * 1.5).sin().abs();
    fade_in * fade_out * pulse
}

fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let value = ((value - start) / (end - start)).clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn rounded_rect_distance(px: f32, py: f32, rect: Rect, radius: f32) -> f32 {
    let dx = (px - (rect.x + rect.width * 0.5)).abs() - (rect.width * 0.5 - radius);
    let dy = (py - (rect.y + rect.height * 0.5)).abs() - (rect.height * 0.5 - radius);
    dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius
}

fn scaled_rect(rect: Rect, scale: f32) -> Rect {
    Rect {
        x: rect.x * scale,
        y: rect.y * scale,
        width: rect.width * scale,
        height: rect.height * scale,
    }
}

fn inset(rect: Rect, amount: f32) -> Rect {
    Rect {
        x: rect.x + amount,
        y: rect.y + amount,
        width: rect.width - amount * 2.0,
        height: rect.height - amount * 2.0,
    }
}

fn mix_color(start: Rgb, end: Rgb, amount: f32) -> Rgb {
    Rgb(
        (start.0 as f32 + (end.0 as f32 - start.0 as f32) * amount) as u8,
        (start.1 as f32 + (end.1 as f32 - start.1 as f32) * amount) as u8,
        (start.2 as f32 + (end.2 as f32 - start.2 as f32) * amount) as u8,
    )
}

fn scale_color(color: Rgb, brightness: f32) -> Rgb {
    Rgb(
        (color.0 as f32 * brightness).clamp(0.0, 255.0) as u8,
        (color.1 as f32 * brightness).clamp(0.0, 255.0) as u8,
        (color.2 as f32 * brightness).clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::signal_intensity;

    #[test]
    fn lights_activate_in_sequence() {
        for (active_index, elapsed) in [0.5, 2.5, 4.5].into_iter().enumerate() {
            for index in 0..3 {
                let intensity = signal_intensity(elapsed, index);
                assert_eq!(intensity > 0.8, index == active_index);
            }
        }
    }

    #[test]
    fn active_phase_ends_with_a_dark_gap() {
        assert_eq!(signal_intensity(1.95, 0), 0.0);
    }
}
