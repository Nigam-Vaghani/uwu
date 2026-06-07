use crate::features::config::{read_config, AppConfig};
use crate::features::events::PetEventPayload;
use crate::features::reminders::PetMood;
use crate::features::system_monitor::SystemMonitorState;
use chrono::Local;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

const MAX_RECENT_EVENTS: usize = 5;

pub struct RecentEventsState(pub Mutex<VecDeque<String>>);

impl RecentEventsState {
    pub fn new() -> Self {
        Self(Mutex::new(VecDeque::new()))
    }

    pub fn push(&self, event_type: &str) {
        if let Ok(mut events) = self.0.lock() {
            events.push_back(event_type.to_string());
            while events.len() > MAX_RECENT_EVENTS {
                events.pop_front();
            }
        }
    }

    pub fn snapshot(&self) -> Vec<String> {
        self.0
            .lock()
            .map(|events| events.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiContext {
    pub pet_name: String,
    pub personality: String,
    pub mood: String,
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub battery_level: Option<f32>,
    pub active_app: Option<String>,
    pub coding_session_duration: u64,
    pub recent_events: Vec<String>,
    pub time_of_day: String,
    pub day_of_week: String,
}

fn mood_label(mood: PetMood) -> &'static str {
    match mood {
        PetMood::Happy => "happy",
        PetMood::Neutral => "neutral",
        PetMood::Sad => "sad",
        PetMood::Excited => "excited",
        PetMood::Sleepy => "sleepy",
    }
}

fn personality_tone(personality: &str) -> &'static str {
    match personality.trim().to_lowercase().as_str() {
        "calm" => "Calm, gentle, and reassuring",
        "sassy" => "Playfully sarcastic but caring",
        "motivational" => "Direct, energizing, and goal-focused",
        _ => "Upbeat, encouraging, and cheerful",
    }
}

pub fn assemble_context(
    app: &AppHandle,
    _payload: &PetEventPayload,
) -> Result<(AiContext, AppConfig), String> {
    let config = read_config(app)?;
    let mood = crate::features::reminders::get_mood(app.clone())?.mood;
    let stats = app
        .try_state::<SystemMonitorState>()
        .ok_or_else(|| "system monitor unavailable".to_string())?
        .snapshot()?;
    let recent_events = app
        .try_state::<RecentEventsState>()
        .map(|state| state.snapshot())
        .unwrap_or_default();

    let now = Local::now();
    let context = AiContext {
        pet_name: config.pet_name.clone(),
        personality: config.personality.clone(),
        mood: mood_label(mood).to_string(),
        cpu_usage: stats.cpu_percent,
        ram_usage: stats.ram_percent,
        battery_level: stats.battery_percent,
        active_app: stats.active_app.clone(),
        coding_session_duration: stats.coding_session_duration_secs,
        recent_events,
        time_of_day: now.format("%H:%M").to_string(),
        day_of_week: now.format("%A").to_string(),
    };

    Ok((context, config))
}

pub fn build_prompt(context: &AiContext, payload: &PetEventPayload) -> String {
    let personality_tone = personality_tone(&context.personality);
    let battery = context
        .battery_level
        .map(|level| format!("{level:.0}%"))
        .unwrap_or_else(|| "unknown".to_string());
    let active_app = context
        .active_app
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let recent = if context.recent_events.is_empty() {
        "none".to_string()
    } else {
        context.recent_events.join(", ")
    };

    let trigger_detail = match payload.event_type {
        crate::features::events::PetEventType::HighCpu => {
            format!("CPU usage is high at {:.0}%", payload.value.unwrap_or(context.cpu_usage))
        }
        crate::features::events::PetEventType::HighRam => {
            format!("RAM usage is high at {:.0}%", payload.value.unwrap_or(context.ram_usage))
        }
        crate::features::events::PetEventType::LowBattery => {
            format!("Battery is low at {:.0}%", payload.value.unwrap_or(0.0))
        }
        crate::features::events::PetEventType::CriticalBattery => {
            format!("Battery is critical at {:.0}%", payload.value.unwrap_or(0.0))
        }
        crate::features::events::PetEventType::CodingSessionStarted => {
            format!(
                "User started coding in {}",
                payload
                    .app_name
                    .clone()
                    .unwrap_or_else(|| "their editor".to_string())
            )
        }
        crate::features::events::PetEventType::CodingSessionEnded => {
            format!(
                "User ended a coding session in {}",
                payload
                    .app_name
                    .clone()
                    .unwrap_or_else(|| "their editor".to_string())
            )
        }
        crate::features::events::PetEventType::ReminderFired => {
            format!(
                "Reminder fired: {} — {}",
                payload.title.clone().unwrap_or_else(|| "Reminder".to_string()),
                payload.message.clone().unwrap_or_default()
            )
        }
        crate::features::events::PetEventType::IdleDetected => {
            "User has been idle for a while".to_string()
        }
        crate::features::events::PetEventType::UserInteraction => {
            "User clicked on the pet".to_string()
        }
        crate::features::events::PetEventType::RandomMessage => {
            "Send a spontaneous check-in message".to_string()
        }
        crate::features::events::PetEventType::LevelUp => {
            format!(
                "User leveled up to level {:.0}",
                payload.value.unwrap_or(1.0)
            )
        }
        crate::features::events::PetEventType::AchievementUnlocked => {
            format!(
                "User unlocked achievement: {}",
                payload.title.clone().unwrap_or_else(|| "Achievement".to_string())
            )
        }
        crate::features::events::PetEventType::ObjectiveCompleted => {
            format!(
                "User completed objective: {}",
                payload.title.clone().unwrap_or_else(|| "Daily objective".to_string())
            )
        }
        crate::features::events::PetEventType::StreakMilestone => {
            format!(
                "User hit a {:.0}-day streak milestone",
                payload.value.unwrap_or(0.0)
            )
        }
    };

    format!(
        "You are {pet_name}, a desktop pet companion with a {personality} personality.\n\
Tone: {personality_tone}. Current mood: {mood}.\n\
Context: CPU {cpu:.0}%, RAM {ram:.0}%, battery {battery}, active app \"{active_app}\", \
coding session {coding_secs}s, time {time_of_day} on {day_of_week}, recent events: {recent}.\n\
Rules: Reply in plain text only, no markdown, at most 2 casual sentences, under 80 tokens, stay in character.\n\
Event: {event_type}. {trigger_detail}\n\
Respond as {pet_name}:",
        pet_name = context.pet_name,
        personality = context.personality,
        personality_tone = personality_tone,
        mood = context.mood,
        cpu = context.cpu_usage,
        ram = context.ram_usage,
        battery = battery,
        active_app = active_app,
        coding_secs = context.coding_session_duration,
        time_of_day = context.time_of_day,
        day_of_week = context.day_of_week,
        recent = recent,
        event_type = format!("{:?}", payload.event_type),
        trigger_detail = trigger_detail,
    )
}

pub fn record_event(recent_events: State<'_, RecentEventsState>, event_type: &str) {
    recent_events.push(event_type);
}
