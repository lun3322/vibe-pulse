use std::time::{Duration, Instant};

use serde_json::json;

use super::{ClientKind, HookEvent, SessionStatus, SessionStore};

fn event(name: &str) -> HookEvent {
    HookEvent::from_json(
        ClientKind::Qoder,
        "desk".to_owned(),
        &json!({"session_id": "session-1", "hook_event_name": name}),
    )
    .unwrap()
}

#[test]
fn later_event_creates_session_without_start() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("UserPromptSubmit"), now);
    assert_eq!(store.sessions().len(), 1);
    assert_eq!(store.sessions()[0].status, SessionStatus::Working);
}

#[test]
fn duplicate_events_update_one_session() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("SessionStart"), now);
    store.apply(event("UserPromptSubmit"), now);
    assert_eq!(store.sessions().len(), 1);
}

#[test]
fn end_without_start_ignores_late_events_until_new_start() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("SessionEnd"), now);
    store.apply(event("UserPromptSubmit"), now);
    assert!(store.sessions().is_empty());
    store.apply(event("SessionStart"), now);
    assert_eq!(store.sessions().len(), 1);
}

#[test]
fn manual_close_allows_later_events_to_recreate_session() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("SessionStart"), now);
    store.dismiss(0);
    store.apply(event("UserPromptSubmit"), now);
    assert_eq!(store.sessions().len(), 1);
    assert_eq!(store.sessions()[0].status, SessionStatus::Working);
}

#[test]
fn successful_end_disappears_after_two_seconds() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("SessionStart"), now);
    store.apply(event("SessionEnd"), now);
    assert_eq!(store.sessions()[0].status, SessionStatus::Finishing);
    store.remove_finished(now + Duration::from_secs(2));
    assert!(store.sessions().is_empty());
}

#[test]
fn completed_session_ignores_late_events_until_new_start() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("SessionStart"), now);
    store.apply(event("SessionEnd"), now);
    store.apply(event("UserPromptSubmit"), now);
    assert_eq!(store.sessions()[0].status, SessionStatus::Finishing);
    store.remove_finished(now + Duration::from_secs(2));
    assert!(store.sessions().is_empty());
    store.apply(event("UserPromptSubmit"), now);
    assert!(store.sessions().is_empty());
    store.apply(event("SessionStart"), now);
    assert_eq!(store.sessions().len(), 1);
    assert_eq!(store.sessions()[0].status, SessionStatus::Idle);
}

#[test]
fn failure_remains_after_session_end() {
    let now = Instant::now();
    let mut store = SessionStore::default();
    store.apply(event("StopFailure"), now);
    store.apply(event("SessionEnd"), now);
    store.remove_finished(now + Duration::from_secs(10));
    assert_eq!(store.sessions()[0].status, SessionStatus::Failed);
}
