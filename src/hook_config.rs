use serde_json::{Map, json};

const HOOK_EVENTS: [&str; 13] = [
    "SessionStart",
    "SessionEnd",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionRequest",
    "PermissionDenied",
    "Notification",
    "Stop",
    "StopFailure",
    "Elicitation",
    "ElicitationResult",
];

pub fn generate(client: &str, endpoint: &str, token: &str, host: &str) -> String {
    let hook = json!({
        "type": "http",
        "url": endpoint,
        "headers": {
            "Authorization": format!("Bearer {token}"),
            "X-Vibe-Client": client,
            "X-Vibe-Host": host,
        },
        "timeout": 10,
    });
    let hooks = HOOK_EVENTS
        .into_iter()
        .fold(Map::new(), |mut hooks, event| {
            hooks.insert(event.to_owned(), json!([{ "hooks": [hook.clone()] }]));
            hooks
        });
    serde_json::to_string_pretty(&json!({ "hooks": hooks })).expect("Hook 配置仅包含可序列化字段")
}

pub fn endpoint(host: &str, port: u16) -> String {
    format!("http://{host}:{port}/hooks")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn generated_config_contains_identity_and_token() {
        let config = generate(
            "claude-code",
            "http://127.0.0.1:17321/hooks",
            "secret",
            "desk",
        );
        let value: Value = serde_json::from_str(&config).unwrap();
        let hook = &value["hooks"]["SessionStart"][0]["hooks"][0];
        assert_eq!(hook["headers"]["Authorization"], "Bearer secret");
        assert_eq!(hook["headers"]["X-Vibe-Client"], "claude-code");
        assert_eq!(hook["headers"]["X-Vibe-Host"], "desk");
    }
}
