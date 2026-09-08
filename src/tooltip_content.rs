use windows::{
    Win32::{
        Foundation::COLORREF,
        Graphics::Gdi::{
            CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DEFAULT_PITCH,
            DeleteObject, FF_DONTCARE, FW_NORMAL, FW_SEMIBOLD, HFONT, OUT_DEFAULT_PRECIS,
        },
    },
    core::{Error, HRESULT, Result, w},
};

use crate::model::{ClientKind, Session, SessionStatus};

const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);

pub struct TooltipContent {
    pub title: String,
    pub cwd: String,
    pub prompt: String,
    pub status: String,
    pub accent: COLORREF,
    pub status_color: COLORREF,
    pub header_font: HFONT,
    pub body_font: HFONT,
}

impl TooltipContent {
    pub fn new() -> Result<Self> {
        let header_font = create_font(-16, FW_SEMIBOLD.0 as i32)?;
        let body_font = match create_font(-14, FW_NORMAL.0 as i32) {
            Ok(font) => font,
            Err(error) => {
                unsafe {
                    let _ = DeleteObject(header_font.into());
                }
                return Err(error);
            }
        };
        Ok(Self {
            title: String::new(),
            cwd: String::new(),
            prompt: String::new(),
            status: String::new(),
            accent: rgb(142, 150, 151),
            status_color: rgb(142, 150, 151),
            header_font,
            body_font,
        })
    }

    pub fn update(&mut self, session: &Session, now: std::time::Instant) {
        self.title = format!("{}  ·  {}", session.key.client.label(), session.key.host);
        self.cwd.clone_from(&session.cwd);
        self.prompt = if session.prompt_summary.is_empty() {
            "暂无任务摘要".to_owned()
        } else {
            session.prompt_summary.clone()
        };
        self.status = session
            .tooltip(now)
            .lines()
            .last()
            .unwrap_or(session.status.label())
            .to_owned();
        self.accent = client_color(&session.key.client);
        self.status_color = status_color(session.status);
    }
}

impl Drop for TooltipContent {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.header_font.into());
            let _ = DeleteObject(self.body_font.into());
        }
    }
}

fn create_font(height: i32, weight: i32) -> Result<HFONT> {
    let font = unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
            w!("Segoe UI Variable Text"),
        )
    };
    if font.0.is_null() {
        Err(Error::new(APP_FAILURE, "无法创建提示文字字体"))
    } else {
        Ok(font)
    }
}

fn client_color(client: &ClientKind) -> COLORREF {
    match client {
        ClientKind::Qoder => rgb(0, 214, 100),
        ClientKind::ClaudeCode => rgb(237, 120, 55),
        ClientKind::Unknown => rgb(142, 150, 151),
    }
}

fn status_color(status: SessionStatus) -> COLORREF {
    match status {
        SessionStatus::Idle | SessionStatus::Finishing => rgb(0, 214, 100),
        SessionStatus::Working => rgb(255, 185, 0),
        SessionStatus::Waiting | SessionStatus::Failed => rgb(255, 31, 45),
    }
}

pub const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF(red as u32 | ((green as u32) << 8) | ((blue as u32) << 16))
}
