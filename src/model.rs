use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use serde_json::Value;

const COMPLETION_DISPLAY_TIME: Duration = Duration::from_secs(2);
const PROMPT_SUMMARY_LENGTH: usize = 120;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ClientKind {
    Qoder,
    ClaudeCode,
    Unknown,
}

impl ClientKind {
    pub fn from_header(value: Option<&str>) -> Self {
        match value {
            Some("qoder") => Self::Qoder,
            Some("claude-code") => Self::ClaudeCode,
            _ => Self::Unknown,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Qoder => "Qoder",
            Self::ClaudeCode => "Claude Code",
            Self::Unknown => "未知客户端",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SessionKey {
    pub client: ClientKind,
    pub host: String,
    pub session_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionStatus {
    Idle,
    Working,
    Waiting,
    Failed,
    Finishing,
}

impl SessionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "空闲",
            Self::Working => "工作中",
            Self::Waiting => "等待输入或授权",
            Self::Failed => "异常失败",
            Self::Finishing => "已结束",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HookEvent {
    pub key: SessionKey,
    pub name: String,
    pub notification_type: Option<String>,
    pub cwd: Option<String>,
    pub prompt: Option<String>,
}

impl HookEvent {
    pub fn from_json(client: ClientKind, host: String, value: &Value) -> Result<Self, String> {
        let session_id = required_text(value, "session_id")?;
        let name = required_text(value, "hook_event_name")?;
        Ok(Self {
            key: SessionKey {
                client,
                host,
                session_id,
            },
            name,
            notification_type: optional_text(value, "notification_type"),
            cwd: optional_text(value, "cwd"),
            prompt: optional_text(value, "prompt"),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub key: SessionKey,
    pub status: SessionStatus,
    pub cwd: String,
    pub prompt_summary: String,
    pub started_at: Instant,
    pub updated_at: Instant,
    remove_at: Option<Instant>,
}

impl Session {
    pub fn tooltip(&self, now: Instant) -> String {
        let elapsed = now.duration_since(self.started_at).as_secs();
        let prompt = if self.prompt_summary.is_empty() {
            "暂无提示"
        } else {
            &self.prompt_summary
        };
        format!(
            "{} · {}\n{}\n{}\n状态：{} · {}",
            self.key.client.label(),
            self.key.host,
            self.cwd,
            prompt,
            self.status.label(),
            format_duration(elapsed),
        )
    }
}

#[derive(Default)]
pub struct SessionStore {
    sessions: Vec<Session>,
    ended_sessions: HashSet<SessionKey>,
}

impl SessionStore {
    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    pub fn apply(&mut self, event: HookEvent, now: Instant) {
        if self.ended_sessions.contains(&event.key) {
            if event.name != "SessionStart" {
                return;
            }
            self.ended_sessions.remove(&event.key);
            self.sessions.retain(|session| session.key != event.key);
        }
        if event.name == "SessionEnd" {
            self.finish(event.key, now);
            return;
        }
        let session = match self.sessions.iter_mut().find(|item| item.key == event.key) {
            Some(session) => session,
            None => {
                self.sessions.push(new_session(&event, now));
                self.sessions.last_mut().expect("刚插入的会话必须存在")
            }
        };
        apply_event(session, &event, now);
    }

    pub fn dismiss(&mut self, index: usize) {
        if index >= self.sessions.len() {
            return;
        }
        self.sessions.remove(index);
    }

    pub fn remove_finished(&mut self, now: Instant) {
        self.sessions
            .retain(|session| !session.remove_at.is_some_and(|deadline| now >= deadline));
    }

    fn finish(&mut self, key: SessionKey, now: Instant) {
        self.ended_sessions.insert(key.clone());
        let Some(session) = self.sessions.iter_mut().find(|item| item.key == key) else {
            return;
        };
        if session.status == SessionStatus::Failed {
            return;
        }
        session.status = SessionStatus::Finishing;
        session.updated_at = now;
        session.remove_at = Some(now + COMPLETION_DISPLAY_TIME);
    }
}

fn new_session(event: &HookEvent, now: Instant) -> Session {
    Session {
        key: event.key.clone(),
        status: SessionStatus::Idle,
        cwd: event.cwd.clone().unwrap_or_else(|| "未知目录".to_owned()),
        prompt_summary: String::new(),
        started_at: now,
        updated_at: now,
        remove_at: None,
    }
}

fn apply_event(session: &mut Session, event: &HookEvent, now: Instant) {
    if let Some(cwd) = &event.cwd {
        session.cwd.clone_from(cwd);
    }
    if let Some(prompt) = &event.prompt {
        session.prompt_summary = summarize(prompt);
    }
    session.status = status_for_event(event, session.status);
    session.updated_at = now;
    session.remove_at = None;
}

fn status_for_event(event: &HookEvent, current: SessionStatus) -> SessionStatus {
    match event.name.as_str() {
        "SessionStart" | "Stop" => SessionStatus::Idle,
        "UserPromptSubmit" | "PreToolUse" | "PostToolUse" | "PostToolUseFailure"
        | "ElicitationResult" => SessionStatus::Working,
        "PermissionRequest" | "PermissionDenied" | "Elicitation" => SessionStatus::Waiting,
        "StopFailure" => SessionStatus::Failed,
        "Notification" => notification_status(event.notification_type.as_deref(), current),
        _ => current,
    }
}

fn notification_status(notification_type: Option<&str>, current: SessionStatus) -> SessionStatus {
    match notification_type {
        Some("permission_prompt" | "elicitation_dialog" | "agent_needs_input") => {
            SessionStatus::Waiting
        }
        Some("idle_prompt" | "agent_completed") => SessionStatus::Idle,
        _ => current,
    }
}

fn required_text(value: &Value, field: &str) -> Result<String, String> {
    optional_text(value, field).ok_or_else(|| format!("缺少字段 {field}"))
}

fn optional_text(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)?
        .as_str()
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn summarize(prompt: &str) -> String {
    let single_line = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut characters = single_line.chars();
    let summary: String = characters.by_ref().take(PROMPT_SUMMARY_LENGTH).collect();
    if characters.next().is_some() {
        format!("{summary}…")
    } else {
        summary
    }
}

fn format_duration(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = total_seconds % 3600 / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}小时{minutes}分")
    } else if minutes > 0 {
        format!("{minutes}分{seconds}秒")
    } else {
        format!("{seconds}秒")
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
