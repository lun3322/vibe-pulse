pub use crate::raster::Canvas;
use crate::{
    model::{ClientKind, Session, SessionStatus},
    raster::{Rect, Rgb, scale_color},
};

pub const BASE_WIDTH: i32 = 404;
pub const GROUP_HEIGHT: i32 = 156;
const GROUP_TOP: f32 = 12.0;
const CLOSE_X: f32 = 377.0;
const CLOSE_Y: f32 = 22.0;
const CLOSE_HIT_RADIUS: f32 = 20.0;
const RED: Rgb = Rgb(255, 31, 45);
const YELLOW: Rgb = Rgb(255, 185, 0);
const GREEN: Rgb = Rgb(0, 214, 100);
const QODER_MARKER: Rgb = Rgb(0, 214, 100);
const CLAUDE_MARKER: Rgb = Rgb(237, 120, 55);
const UNKNOWN_MARKER: Rgb = Rgb(142, 150, 151);

impl Canvas {
    pub fn render(&mut self, sessions: &[Session], hovered: Option<usize>, elapsed: f32) {
        self.pixels.fill(0);
        for (index, session) in sessions.iter().enumerate() {
            self.draw_group(session, index, hovered == Some(index), elapsed);
        }
    }

    fn draw_group(&mut self, session: &Session, index: usize, hovered: bool, elapsed: f32) {
        let scale = self.scale;
        let offset = index as f32 * GROUP_HEIGHT as f32 * scale;
        let outer = scaled_rect(
            Rect {
                x: 18.0,
                y: GROUP_TOP,
                width: 368.0,
                height: 132.0,
            },
            scale,
            offset,
        );
        self.draw_rounded_rect(outer, 34.0 * scale, Rgb(55, 62, 63), Rgb(21, 24, 25), 1.0);
        let inner = inset(outer, 5.0 * scale);
        self.draw_rounded_rect(inner, 29.0 * scale, Rgb(39, 44, 45), Rgb(9, 11, 12), 1.0);
        self.draw_sheen(scale, offset);
        self.draw_client_marker(&session.key.client, scale, offset);
        self.draw_signal_lights(session.status, scale, offset, elapsed);
        if hovered {
            self.draw_close_button(scale, offset);
        }
    }

    fn draw_sheen(&mut self, scale: f32, offset: f32) {
        let sheen = Rect {
            x: 39.0 * scale,
            y: 23.0 * scale + offset,
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
    }

    fn draw_client_marker(&mut self, client: &ClientKind, scale: f32, offset: f32) {
        let color = match client {
            ClientKind::Qoder => QODER_MARKER,
            ClientKind::ClaudeCode => CLAUDE_MARKER,
            ClientKind::Unknown => UNKNOWN_MARKER,
        };
        let marker = Rect {
            x: 26.0 * scale,
            y: 49.0 * scale + offset,
            width: 5.0 * scale,
            height: 58.0 * scale,
        };
        self.draw_rounded_rect(marker, 3.0 * scale, color, color, 1.0);
    }

    fn draw_signal_lights(&mut self, status: SessionStatus, scale: f32, offset: f32, elapsed: f32) {
        let active_index = match status {
            SessionStatus::Waiting | SessionStatus::Failed => 0,
            SessionStatus::Working => 1,
            SessionStatus::Idle | SessionStatus::Finishing => 2,
        };
        let pulse = pulse_intensity(elapsed, status);
        for (index, color) in [RED, YELLOW, GREEN].into_iter().enumerate() {
            self.draw_lamp(
                (84.0 + index as f32 * 118.0) * scale,
                78.0 * scale + offset,
                45.0 * scale,
                color,
                if index == active_index { pulse } else { 0.0 },
            );
        }
    }

    fn draw_close_button(&mut self, scale: f32, offset: f32) {
        let cx = CLOSE_X * scale;
        let cy = CLOSE_Y * scale + offset;
        let radius = 15.0 * scale;
        self.draw_circle(cx, cy, radius, Rgb(116, 28, 32), 0.96);
        let arm = 6.0 * scale;
        self.draw_line(
            cx - arm,
            cy - arm,
            cx + arm,
            cy + arm,
            Rgb(255, 235, 235),
            1.5 * scale,
        );
        self.draw_line(
            cx + arm,
            cy - arm,
            cx - arm,
            cy + arm,
            Rgb(255, 235, 235),
            1.5 * scale,
        );
    }

    fn draw_lamp(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, intensity: f32) {
        if intensity > 0.01 {
            self.draw_glow(
                cx,
                cy,
                radius * 0.78,
                radius * 1.35,
                color,
                intensity * 0.45,
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
}

pub fn group_at(y: i32, group_count: usize, scale: f32) -> Option<usize> {
    if y < 0 {
        return None;
    }
    let index = (y as f32 / (GROUP_HEIGHT as f32 * scale)) as usize;
    (index < group_count).then_some(index)
}

pub fn group_top(index: usize, scale: f32) -> i32 {
    ((index as f32 * GROUP_HEIGHT as f32 + GROUP_TOP) * scale).round() as i32
}

pub fn close_at(x: i32, y: i32, group_count: usize, scale: f32) -> Option<usize> {
    let index = group_at(y, group_count, scale)?;
    let offset = index as f32 * GROUP_HEIGHT as f32 * scale;
    let dx = x as f32 - CLOSE_X * scale;
    let dy = y as f32 - (CLOSE_Y * scale + offset);
    (dx * dx + dy * dy <= (CLOSE_HIT_RADIUS * scale).powi(2)).then_some(index)
}

fn pulse_intensity(elapsed: f32, status: SessionStatus) -> f32 {
    if status == SessionStatus::Finishing {
        return 1.0;
    }
    0.78 + 0.22 * (elapsed * std::f32::consts::TAU * 1.5).sin().abs()
}

fn scaled_rect(rect: Rect, scale: f32, offset: f32) -> Rect {
    Rect {
        x: rect.x * scale,
        y: rect.y * scale + offset,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_testing_finds_group_and_close_button() {
        let scale = 0.32;
        assert_eq!(group_at(20, 2, scale), Some(0));
        assert_eq!(group_at(70, 2, scale), Some(1));
        assert_eq!(close_at(121, 7, 2, scale), Some(0));
    }
}
