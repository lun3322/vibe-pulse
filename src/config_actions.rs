use windows::{
    Win32::Foundation::{HWND, LPARAM, WPARAM},
    core::{Error, HRESULT, Result},
};

use crate::{
    clipboard,
    config_controls::{
        self, CLOSE_ID, COPY_CLAUDE_ID, COPY_QODER_ID, ConfigControlInit, ConfigControls,
    },
    config_theme::ConfigTheme,
    hook_config::{endpoint, generate},
    http_server::PORT,
    network,
    settings::Settings,
};

const INVALID_ARGUMENT: HRESULT = HRESULT(0x80070057_u32 as i32);

pub struct Command {
    pub hwnd: HWND,
    pub identifier: isize,
}

pub enum CommandOutcome {
    None,
    Copied,
    Close,
}

pub struct ConfigState {
    settings: Settings,
    settings_path: String,
    controls: ConfigControls,
    theme: Option<ConfigTheme>,
}

impl ConfigState {
    pub fn new(settings: Settings, settings_path: String) -> Self {
        Self {
            settings,
            settings_path,
            controls: ConfigControls::default(),
            theme: None,
        }
    }

    pub fn initialize_content(&mut self, hwnd: HWND) -> Result<()> {
        let theme = ConfigTheme::new(hwnd)?;
        let private_addresses = network::private_ipv4_addresses()?;
        self.controls = config_controls::create(ConfigControlInit {
            parent: hwnd,
            private_addresses: &private_addresses,
            font: theme.body_font(),
        })?;
        self.theme = Some(theme);
        Ok(())
    }

    pub fn handle(&self, command: Command) -> Result<CommandOutcome> {
        match command.identifier {
            COPY_QODER_ID => {
                self.copy_config(command.hwnd, "qoder")?;
                Ok(CommandOutcome::Copied)
            }
            COPY_CLAUDE_ID => {
                self.copy_config(command.hwnd, "claude-code")?;
                Ok(CommandOutcome::Copied)
            }
            CLOSE_ID => Ok(CommandOutcome::Close),
            _ => Ok(CommandOutcome::None),
        }
    }

    pub unsafe fn paint(&self, hwnd: HWND) {
        if let Some(theme) = &self.theme {
            unsafe { theme.paint(hwnd, &self.token_fingerprint()) };
        }
    }

    pub unsafe fn draw_item(&self, lparam: LPARAM) {
        if let Some(theme) = &self.theme {
            unsafe { theme.draw_button(lparam) };
        }
    }

    pub unsafe fn control_color(&self, wparam: WPARAM, edit: bool) -> isize {
        self.theme
            .as_ref()
            .map(|theme| unsafe {
                if edit {
                    theme.color_edit(wparam)
                } else {
                    theme.color_button(wparam)
                }
            })
            .unwrap_or_default()
    }

    fn copy_config(&self, hwnd: HWND, client: &str) -> Result<()> {
        let address = config_controls::selected_address(&self.controls)?;
        let host = config_controls::window_text(self.controls.host);
        if host.trim().is_empty() {
            return Err(Error::new(INVALID_ARGUMENT, "来源主机名不能为空"));
        }
        let config = generate(
            client,
            &endpoint(address.trim(), PORT),
            &self.settings.token,
            host.trim(),
        );
        clipboard::copy_text(hwnd, &config)
    }

    fn token_fingerprint(&self) -> String {
        let prefix: String = self.settings.token.chars().take(8).collect();
        let suffix: String = self
            .settings
            .token
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        format!(
            "Bearer 令牌指纹：{prefix}…{suffix}  ·  {}",
            self.settings_path
        )
    }
}
