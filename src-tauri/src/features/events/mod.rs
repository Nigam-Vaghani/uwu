use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetEventPayload {
    pub event_type: PetEventType,
    pub title: Option<String>,
    pub message: Option<String>,
    pub category: Option<String>,
    pub reminder_id: Option<i64>,
    pub value: Option<f32>,
    pub app_name: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PetEventType {
    IdleDetected,
    ReminderFired,
    RandomMessage,
    UserInteraction,
    HighCpu,
    HighRam,
    LowBattery,
    CriticalBattery,
    CodingSessionStarted,
    CodingSessionEnded,
    LevelUp,
    AchievementUnlocked,
    ObjectiveCompleted,
    StreakMilestone,
}

pub const PET_EVENT: &str = "pet:event";
pub const SYSTEM_STATS_EVENT: &str = "system:stats";

pub fn emit_event(app: &AppHandle, payload: PetEventPayload) -> Result<(), String> {
    app.emit(PET_EVENT, payload)
        .map_err(|error| error.to_string())
}

pub fn emit_system_stats<T: Serialize + Clone>(app: &AppHandle, stats: T) -> Result<(), String> {
    app.emit(SYSTEM_STATS_EVENT, stats)
        .map_err(|error| error.to_string())
}

pub fn emit_user_interaction(app: &AppHandle) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::UserInteraction,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: None,
            app_name: None,
        },
    )
}

#[tauri::command]
pub fn emit_pet_interaction(app: AppHandle) -> Result<(), String> {
    emit_user_interaction(&app)
}

pub fn emit_idle_detected(app: &AppHandle) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::IdleDetected,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: None,
            app_name: None,
        },
    )
}

pub fn emit_random_message(app: &AppHandle) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::RandomMessage,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: None,
            app_name: None,
        },
    )
}

pub fn emit_reminder_fired(
    app: &AppHandle,
    reminder_id: i64,
    title: String,
    message: String,
    category: String,
) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::ReminderFired,
            title: Some(title),
            message: Some(message),
            category: Some(category),
            reminder_id: Some(reminder_id),
            value: None,
            app_name: None,
        },
    )
}

pub fn emit_high_cpu(app: &AppHandle, cpu_percent: f32) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::HighCpu,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: Some(cpu_percent),
            app_name: None,
        },
    )
}

pub fn emit_high_ram(app: &AppHandle, ram_percent: f32) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::HighRam,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: Some(ram_percent),
            app_name: None,
        },
    )
}

pub fn emit_low_battery(app: &AppHandle, battery_percent: f32) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::LowBattery,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: Some(battery_percent),
            app_name: None,
        },
    )
}

pub fn emit_critical_battery(app: &AppHandle, battery_percent: f32) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::CriticalBattery,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: Some(battery_percent),
            app_name: None,
        },
    )
}

pub fn emit_coding_session_started(app: &AppHandle, app_name: String) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::CodingSessionStarted,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: None,
            app_name: Some(app_name),
        },
    )
}

pub fn emit_coding_session_ended(app: &AppHandle, app_name: String) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::CodingSessionEnded,
            title: None,
            message: None,
            category: None,
            reminder_id: None,
            value: None,
            app_name: Some(app_name),
        },
    )
}

pub fn emit_level_up(app: &AppHandle, level: i64) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::LevelUp,
            title: Some(format!("Level {level}")),
            message: None,
            category: None,
            reminder_id: None,
            value: Some(level as f32),
            app_name: None,
        },
    )
}

pub fn emit_achievement_unlocked(
    app: &AppHandle,
    achievement_id: String,
    name: String,
    icon: String,
) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::AchievementUnlocked,
            title: Some(name),
            message: Some(icon),
            category: Some(achievement_id),
            reminder_id: None,
            value: None,
            app_name: None,
        },
    )
}

pub fn emit_objective_completed(app: &AppHandle, title: String) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::ObjectiveCompleted,
            title: Some(title),
            message: None,
            category: None,
            reminder_id: None,
            value: None,
            app_name: None,
        },
    )
}

pub fn emit_streak_milestone(app: &AppHandle, days: i64) -> Result<(), String> {
    emit_event(
        app,
        PetEventPayload {
            event_type: PetEventType::StreakMilestone,
            title: Some(format!("{days}-day streak")),
            message: None,
            category: None,
            reminder_id: None,
            value: Some(days as f32),
            app_name: None,
        },
    )
}
