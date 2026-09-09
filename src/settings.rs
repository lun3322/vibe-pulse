use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

const SETTINGS_FILE: &str = "vibe-pulse.settings.json";
const LEGACY_DIRECTORY: &str = "VibePulse";
const LEGACY_FILE: &str = "settings.json";
const TOKEN_BYTES: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub token: String,
    pub allow_lan: bool,
    #[serde(default = "default_address")]
    pub selected_address: String,
}

impl Settings {
    pub fn load_or_create() -> Result<Self, String> {
        let path = settings_path()?;
        if path.exists() {
            return read(&path);
        }
        if let Some(legacy_path) = legacy_settings_path()
            && legacy_path.exists()
        {
            let settings = read(&legacy_path)?;
            settings.save()?;
            return Ok(settings);
        }
        let settings = Self {
            token: generate_token()?,
            allow_lan: false,
            selected_address: default_address(),
        };
        settings.save()?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), String> {
        validate_token(&self.token)?;
        let path = settings_path()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|error| format!("无法序列化设置：{error}"))?;
        fs::write(&path, content)
            .map_err(|error| format!("无法保存设置 {}：{error}", path.display()))
    }

    pub fn path_display() -> Result<String, String> {
        Ok(settings_path()?.display().to_string())
    }
}

fn read(path: &PathBuf) -> Result<Settings, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("无法读取设置 {}：{error}", path.display()))?;
    let settings: Settings = serde_json::from_str(&content)
        .map_err(|error| format!("设置格式无效 {}：{error}", path.display()))?;
    validate_token(&settings.token)?;
    Ok(settings)
}

fn settings_path() -> Result<PathBuf, String> {
    let executable = env::current_exe().map_err(|error| format!("无法获取程序路径：{error}"))?;
    let directory = executable
        .parent()
        .ok_or_else(|| "程序路径不包含父目录".to_owned())?;
    Ok(directory.join(SETTINGS_FILE))
}

fn legacy_settings_path() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA").map(|directory| {
        PathBuf::from(directory)
            .join(LEGACY_DIRECTORY)
            .join(LEGACY_FILE)
    })
}

fn generate_token() -> Result<String, String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| format!("无法生成访问令牌：{error}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn validate_token(token: &str) -> Result<(), String> {
    if token.len() == TOKEN_BYTES * 2 && token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("设置中的访问令牌无效，应为 64 位十六进制字符串".to_owned())
    }
}

fn default_address() -> String {
    "127.0.0.1".to_owned()
}

#[cfg(test)]
mod tests {
    use super::{generate_token, validate_token};

    #[test]
    fn token_has_256_bits_encoded_as_hex() {
        let token = generate_token().unwrap();
        assert_eq!(token.len(), 64);
        assert!(token.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert!(validate_token(&token).is_ok());
    }
}
