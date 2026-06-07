use crate::features::database::{chrono_now, get_config_i64, with_db};
use crate::features::events::emit_reminder_fired;
use chrono::Timelike;
use cron::Schedule;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: i64,
    pub title: String,
    pub message: String,
    pub cron_expr: String,
    pub enabled: bool,
    pub category: String,
    pub last_fired: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReminderInput {
    pub title: String,
    pub message: String,
    pub cron_expr: String,
    pub category: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReminderInput {
    pub id: i64,
    pub title: String,
    pub message: String,
    pub cron_expr: String,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PetMood {
    Happy,
    Neutral,
    Sad,
    Excited,
    Sleepy,
}

impl PetMood {
    fn as_str(self) -> &'static str {
        match self {
            Self::Happy => "happy",
            Self::Neutral => "neutral",
            Self::Sad => "sad",
            Self::Excited => "excited",
            Self::Sleepy => "sleepy",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "neutral" => Self::Neutral,
            "sad" => Self::Sad,
            "excited" => Self::Excited,
            "sleepy" => Self::Sleepy,
            _ => Self::Happy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoodState {
    pub mood: PetMood,
    pub last_interaction: i64,
    pub excitement_until: Option<i64>,
}

pub struct InteractionState {
    pub last_interaction: AtomicI64,
    pub idle_emitted: AtomicBool,
}

impl InteractionState {
    pub fn new(now: i64) -> Self {
        Self {
            last_interaction: AtomicI64::new(now),
            idle_emitted: AtomicBool::new(false),
        }
    }

    pub fn touch(&self) {
        self.last_interaction
            .store(chrono_now(), Ordering::Relaxed);
        self.idle_emitted.store(false, Ordering::Relaxed);
    }
}

fn map_reminder(row: &rusqlite::Row<'_>) -> rusqlite::Result<Reminder> {
    Ok(Reminder {
        id: row.get(0)?,
        title: row.get(1)?,
        message: row.get(2)?,
        cron_expr: row.get(3)?,
        enabled: row.get::<_, i64>(4)? == 1,
        category: row.get(5)?,
        last_fired: row.get(6)?,
    })
}

fn reminder_is_due(reminder: &Reminder, now: i64) -> bool {
    if !reminder.enabled {
        return false;
    }

    if reminder.cron_expr.starts_with("@every:") {
        let minutes = reminder
            .cron_expr
            .trim_start_matches("@every:")
            .parse::<i64>()
            .unwrap_or(30);
        let interval_secs = minutes * 60;
        match reminder.last_fired {
            None => true,
            Some(last) => now.saturating_sub(last) >= interval_secs,
        }
    } else {
        let Ok(schedule) = Schedule::from_str(&reminder.cron_expr) else {
            return false;
        };

        let now_dt = chrono::DateTime::from_timestamp(now, 0)
            .map(|dt| dt.with_timezone(&chrono::Local))
            .unwrap_or_else(chrono::Local::now);

        if let Some(last) = reminder.last_fired {
            let last_dt = chrono::DateTime::from_timestamp(last, 0)
                .map(|dt| dt.with_timezone(&chrono::Local))
                .unwrap_or_else(chrono::Local::now);
            if let Some(next) = schedule.after(&last_dt).next() {
                return next <= now_dt;
            }
            false
        } else {
            schedule.after(&now_dt).next().is_some()
        }
    }
}

#[tauri::command]
pub fn get_reminders(app: AppHandle) -> Result<Vec<Reminder>, String> {
    with_db(&app, |connection| {
        let mut statement = connection
            .prepare(
                "SELECT id, title, message, cron_expr, enabled, category, last_fired
                 FROM reminders
                 ORDER BY id ASC",
            )
            .map_err(|error| error.to_string())?;

        let reminders = statement
            .query_map([], map_reminder)
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(reminders)
    })
}

pub fn replace_reminders(app: &AppHandle, reminders: Vec<Reminder>) -> Result<(), String> {
    with_db(app, |connection| {
        connection
            .execute("DELETE FROM reminders", [])
            .map_err(|error| error.to_string())?;

        for reminder in reminders {
            connection
                .execute(
                    "INSERT INTO reminders (title, message, cron_expr, enabled, category, last_fired)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        reminder.title,
                        reminder.message,
                        reminder.cron_expr,
                        if reminder.enabled { 1 } else { 0 },
                        reminder.category,
                        reminder.last_fired
                    ],
                )
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    })
}

#[tauri::command]
pub fn create_reminder(app: AppHandle, input: CreateReminderInput) -> Result<Reminder, String> {
    with_db(&app, |connection| {
        connection
            .execute(
                "INSERT INTO reminders (title, message, cron_expr, enabled, category)
                 VALUES (?1, ?2, ?3, 1, ?4)",
                params![input.title, input.message, input.cron_expr, input.category],
            )
            .map_err(|error| error.to_string())?;

        let id = connection.last_insert_rowid();
        connection
            .query_row(
                "SELECT id, title, message, cron_expr, enabled, category, last_fired
                 FROM reminders WHERE id = ?1",
                [id],
                map_reminder,
            )
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn update_reminder(app: AppHandle, input: UpdateReminderInput) -> Result<Reminder, String> {
    with_db(&app, |connection| {
        connection
            .execute(
                "UPDATE reminders
                 SET title = ?1, message = ?2, cron_expr = ?3, category = ?4, enabled = ?5
                 WHERE id = ?6",
                params![
                    input.title,
                    input.message,
                    input.cron_expr,
                    input.category,
                    if input.enabled { 1 } else { 0 },
                    input.id
                ],
            )
            .map_err(|error| error.to_string())?;

        connection
            .query_row(
                "SELECT id, title, message, cron_expr, enabled, category, last_fired
                 FROM reminders WHERE id = ?1",
                [input.id],
                map_reminder,
            )
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn delete_reminder(app: AppHandle, id: i64) -> Result<(), String> {
    with_db(&app, |connection| {
        connection
            .execute("DELETE FROM reminders WHERE id = ?1", [id])
            .map_err(|error| error.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn toggle_reminder(app: AppHandle, id: i64) -> Result<Reminder, String> {
    with_db(&app, |connection| {
        connection
            .execute(
                "UPDATE reminders SET enabled = CASE enabled WHEN 1 THEN 0 ELSE 1 END WHERE id = ?1",
                [id],
            )
            .map_err(|error| error.to_string())?;

        connection
            .query_row(
                "SELECT id, title, message, cron_expr, enabled, category, last_fired
                 FROM reminders WHERE id = ?1",
                [id],
                map_reminder,
            )
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn get_mood(app: AppHandle) -> Result<MoodState, String> {
    with_db(&app, |connection| {
        connection
            .query_row(
                "SELECT mood, last_interaction, excitement_until FROM pet_state WHERE id = 1",
                [],
                |row| {
                    Ok(MoodState {
                        mood: PetMood::from_str(row.get::<_, String>(0)?.as_str()),
                        last_interaction: row.get(1)?,
                        excitement_until: row.get(2)?,
                    })
                },
            )
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn update_mood(app: AppHandle, mood: PetMood) -> Result<MoodState, String> {
    with_db(&app, |connection| {
        connection
            .execute(
                "UPDATE pet_state SET mood = ?1 WHERE id = 1",
                [mood.as_str()],
            )
            .map_err(|error| error.to_string())?;

        connection
            .query_row(
                "SELECT mood, last_interaction, excitement_until FROM pet_state WHERE id = 1",
                [],
                |row| {
                    Ok(MoodState {
                        mood: PetMood::from_str(row.get::<_, String>(0)?.as_str()),
                        last_interaction: row.get(1)?,
                        excitement_until: row.get(2)?,
                    })
                },
            )
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn record_interaction(
    app: AppHandle,
    interaction: State<'_, Arc<InteractionState>>,
) -> Result<(), String> {
    interaction.touch();

    with_db(&app, |connection| {
        connection
            .execute(
                "UPDATE pet_state SET last_interaction = ?1 WHERE id = 1",
                [chrono_now()],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    })?;

    Ok(())
}

pub fn start_scheduler(app: AppHandle, interaction: Arc<InteractionState>) {
    tauri::async_runtime::spawn(async move {
        let mut idle_tick = tokio::time::interval(Duration::from_secs(30));
        let mut reminder_tick = tokio::time::interval(Duration::from_secs(60));
        let mut random_deadline = next_random_deadline(&app);

        loop {
            tokio::select! {
                _ = idle_tick.tick() => {
                    let _ = check_idle(&app, &interaction);
                }
                _ = reminder_tick.tick() => {
                    let _ = poll_reminders(&app);
                }
                _ = tokio::time::sleep_until(random_deadline) => {
                    let _ = crate::features::events::emit_random_message(&app);
                    random_deadline = next_random_deadline(&app);
                }
            }
        }
    });
}

fn next_random_deadline(app: &AppHandle) -> tokio::time::Instant {
    let min_minutes = with_db(app, |connection| {
        get_config_i64(connection, "random_message_min_minutes", 10)
    })
    .unwrap_or(10);
    let max_minutes = with_db(app, |connection| {
        get_config_i64(connection, "random_message_max_minutes", 20)
    })
    .unwrap_or(20);

    let span = max_minutes.saturating_sub(min_minutes).max(1);
    let jitter = (chrono_now() as u64 % span as u64) as i64;
    let minutes = min_minutes + jitter;
    tokio::time::Instant::now() + Duration::from_secs((minutes * 60) as u64)
}

fn check_idle(app: &AppHandle, interaction: &InteractionState) -> Result<(), String> {
    let idle_minutes = with_db(app, |connection| {
        get_config_i64(connection, "idle_timeout_minutes", 5)
    })?;

    let elapsed_secs = chrono_now() - interaction.last_interaction.load(Ordering::Relaxed);
    if elapsed_secs >= idle_minutes * 60 && !interaction.idle_emitted.load(Ordering::Relaxed) {
        interaction.idle_emitted.store(true, Ordering::Relaxed);
        crate::features::events::emit_idle_detected(app)?;
    }

    Ok(())
}

fn poll_reminders(app: &AppHandle) -> Result<(), String> {
    let now = chrono_now();
    let due = with_db(app, |connection| {
        let mut statement = connection
            .prepare(
                "SELECT id, title, message, cron_expr, enabled, category, last_fired
                 FROM reminders
                 WHERE enabled = 1",
            )
            .map_err(|error| error.to_string())?;

        let reminders = statement
            .query_map([], map_reminder)
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(reminders
            .into_iter()
            .filter(|reminder| reminder_is_due(reminder, now))
            .collect::<Vec<_>>())
    })?;

    for reminder in due {
        emit_reminder_fired(
            app,
            reminder.id,
            reminder.title.clone(),
            reminder.message.clone(),
            reminder.category.clone(),
        )?;

        with_db(app, |connection| {
            connection
                .execute(
                    "UPDATE reminders SET last_fired = ?1 WHERE id = ?2",
                    params![now, reminder.id],
                )
                .map_err(|error| error.to_string())?;
            Ok(())
        })?;

        let _ = set_mood_internal(app, PetMood::Excited, Some(now + 300));
    }

    Ok(())
}

fn set_mood_internal(
    app: &AppHandle,
    mood: PetMood,
    excitement_until: Option<i64>,
) -> Result<(), String> {
    with_db(app, |connection| {
        connection
            .execute(
                "UPDATE pet_state SET mood = ?1, excitement_until = ?2 WHERE id = 1",
                params![mood.as_str(), excitement_until],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn reconcile_mood(app: AppHandle) -> Result<MoodState, String> {
    reconcile_mood_internal(&app)
}

fn reconcile_mood_internal(app: &AppHandle) -> Result<MoodState, String> {
    with_db(app, |connection| {
        let mut state = connection
            .query_row(
                "SELECT mood, last_interaction, excitement_until FROM pet_state WHERE id = 1",
                [],
                |row| {
                    Ok(MoodState {
                        mood: PetMood::from_str(row.get::<_, String>(0)?.as_str()),
                        last_interaction: row.get(1)?,
                        excitement_until: row.get(2)?,
                    })
                },
            )
            .map_err(|error| error.to_string())?;

        let now = chrono_now();

        if let Some(until) = state.excitement_until {
            if now >= until && state.mood == PetMood::Excited {
                state.mood = PetMood::Happy;
                connection
                    .execute(
                        "UPDATE pet_state SET mood = 'happy', excitement_until = NULL WHERE id = 1",
                        [],
                    )
                    .map_err(|error| error.to_string())?;
            }
        }

        let hour = chrono::Local::now().hour();
        let inactive_secs = now.saturating_sub(state.last_interaction);
        let next_mood = if (hour >= 23 || hour < 6) && inactive_secs > 900 {
            PetMood::Sleepy
        } else if inactive_secs > 1800 {
            PetMood::Sad
        } else if inactive_secs > 600 {
            PetMood::Neutral
        } else if state.mood == PetMood::Excited {
            PetMood::Excited
        } else {
            PetMood::Happy
        };

        if next_mood != state.mood && state.mood != PetMood::Excited {
            connection
                .execute(
                    "UPDATE pet_state SET mood = ?1 WHERE id = 1",
                    [next_mood.as_str()],
                )
                .map_err(|error| error.to_string())?;
            state.mood = next_mood;
        }

        Ok(state)
    })
}
