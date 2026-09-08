use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

const SETTINGS_DIRECTORY: &str = "VibePulse";
const SETTINGS_FILE: &str = "settings.json";
const TOKEN_BYTES: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub token: String,
    pub allow_lan: bool,
}

impl Settings {
    pub fn load_or_create() -> Result<Self, String> {
        let path = settings_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("无法读取设置 {}：{error}", path.display()))?;
            return serde_json::from_str(&content)
                .map_err(|error| format!("设置格式无效 {}：{error}", path.display()));
        }
        let settings = Self {
            token: generate_token()?,
            allow_lan: false,
        };
        settings.save()?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = settings_path()?;
        let parent = path.parent().expect("设置路径必须包含父目录");
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建设置目录 {}：{error}", parent.display()))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|error| format!("无法序列化设置：{error}"))?;
        fs::write(&path, content)
            .map_err(|error| format!("无法保存设置 {}：{error}", path.display()))
    }
}

fn settings_path() -> Result<PathBuf, String> {
    let local_app_data =
        env::var_os("LOCALAPPDATA").ok_or_else(|| "环境变量 LOCALAPPDATA 不存在".to_owned())?;
    Ok(PathBuf::from(local_app_data)
        .join(SETTINGS_DIRECTORY)
        .join(SETTINGS_FILE))
}

fn generate_token() -> Result<String, String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| format!("无法生成访问令牌：{error}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::generate_token;

    #[test]
    fn token_has_256_bits_encoded_as_hex() {
        let token = generate_token().unwrap();
        assert_eq!(token.len(), 64);
        assert!(token.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
