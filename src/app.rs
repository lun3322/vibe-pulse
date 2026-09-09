use std::{
    sync::mpsc::{self, Receiver},
    time::Instant,
};

use windows::{
    Win32::{
        Foundation::HWND,
        UI::{
            Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent},
            WindowsAndMessaging::{SW_HIDE, SW_SHOWNOACTIVATE, ShowWindow},
        },
    },
    core::{Error, HRESULT, Result},
};

use crate::{
    drawing::{close_at, group_at},
    http_server::HookServer,
    model::{HookEvent, SessionStore},
    renderer::LayeredSurface,
    settings::Settings,
    tooltip::Tooltip,
    tray::TrayIcon,
};

const APP_FAILURE: HRESULT = HRESULT(0x80004005_u32 as i32);
pub const HOOK_EVENT_MESSAGE: u32 = 0x8002;

pub struct AppState {
    _server: HookServer,
    surface: LayeredSurface,
    _tray: TrayIcon,
    tooltip: Tooltip,
    events: Receiver<HookEvent>,
    sessions: SessionStore,
    hovered: Option<usize>,
    started_at: Instant,
    scale: f32,
}

impl AppState {
    pub fn new(hwnd: HWND, scale: f32) -> Result<Self> {
        let settings = Settings::load_or_create().map_err(app_error)?;
        let (sender, events) = mpsc::channel();
        let server = HookServer::start(
            hwnd,
            HOOK_EVENT_MESSAGE,
            settings.token.clone(),
            settings.allow_lan,
            sender,
        )
        .map_err(app_error)?;
        Ok(Self {
            _server: server,
            surface: LayeredSurface::new(scale)
                .map_err(|error| win_error("无法创建绘制表面", error))?,
            _tray: TrayIcon::new(hwnd).map_err(|error| win_error("无法创建托盘图标", error))?,
            tooltip: Tooltip::new(hwnd).map_err(|error| win_error("无法创建悬停提示", error))?,
            events,
            sessions: SessionStore::default(),
            hovered: None,
            started_at: Instant::now(),
            scale,
        })
    }

    pub fn receive_hooks(&mut self, hwnd: HWND) -> Result<()> {
        while let Ok(event) = self.events.try_recv() {
            self.sessions.apply(event, Instant::now());
        }
        if let Some(index) = self.hovered
            && let Some(session) = self.sessions.sessions().get(index)
        {
            self.tooltip.show(session, Instant::now());
        }
        self.draw(hwnd)
    }

    pub fn tick(&mut self, hwnd: HWND) -> Result<()> {
        self.sessions.remove_finished(Instant::now());
        if self
            .hovered
            .is_some_and(|index| index >= self.sessions.sessions().len())
        {
            self.hovered = None;
            self.tooltip.hide();
        }
        self.draw(hwnd)
    }

    pub fn mouse_move(&mut self, hwnd: HWND, y: i32) -> Result<()> {
        let hovered = group_at(y, self.sessions.sessions().len(), self.scale);
        if hovered != self.hovered {
            self.hovered = hovered;
            match hovered {
                Some(index) => self
                    .tooltip
                    .show(&self.sessions.sessions()[index], Instant::now()),
                None => self.tooltip.hide(),
            }
            self.draw(hwnd)?;
        }
        let mut tracking = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            ..Default::default()
        };
        unsafe { TrackMouseEvent(&mut tracking)? };
        Ok(())
    }

    pub fn mouse_leave(&mut self, hwnd: HWND) -> Result<()> {
        self.hovered = None;
        self.tooltip.hide();
        self.draw(hwnd)
    }

    pub fn click(&mut self, hwnd: HWND, x: i32, y: i32) -> Result<bool> {
        if let Some(index) = close_at(x, y, self.sessions.sessions().len(), self.scale) {
            self.sessions.dismiss(index);
            self.hovered = None;
            self.tooltip.hide();
            self.draw(hwnd)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn draw(&mut self, hwnd: HWND) -> Result<()> {
        let sessions = self.sessions.sessions();
        if sessions.is_empty() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
            return Ok(());
        }
        self.surface.render(
            sessions,
            self.hovered,
            self.started_at.elapsed().as_secs_f32(),
        )?;
        self.surface.present(hwnd)?;
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        Ok(())
    }
}

fn app_error(message: String) -> Error {
    Error::new(APP_FAILURE, message)
}

fn win_error(context: &str, error: Error) -> Error {
    let code = if error.code().is_ok() {
        APP_FAILURE
    } else {
        error.code()
    };
    Error::new(code, format!("{context}：{error}"))
}
