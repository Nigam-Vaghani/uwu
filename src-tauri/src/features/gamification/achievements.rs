use super::types::AchievementStatus;
use crate::features::database::{chrono_now, with_db};
use rusqlite::{params, Connection};
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct AchievementDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}

pub const ACHIEVEMENTS: &[AchievementDefinition] = &[
    AchievementDefinition {
        id: "first_session",
        name: "First Steps",
        description: "Complete your first coding session",
        icon: "🚀",
    },
    AchievementDefinition {
        id: "hydration_hero",
        name: "Hydration Hero",
        description: "Acknowledge 10 hydration reminders",
        icon: "💧",
    },
    AchievementDefinition {
        id: "streak_3",
        name: "Getting Started",
        description: "Maintain a 3-day streak",
        icon: "🔥",
    },
    AchievementDefinition {
        id: "streak_7",
        name: "Week Warrior",
        description: "Maintain a 7-day streak",
        icon: "⭐",
    },
    AchievementDefinition {
        id: "streak_30",
        name: "Monthly Master",
        description: "Maintain a 30-day streak",
        icon: "💎",
    },
    AchievementDefinition {
        id: "level_5",
        name: "Rising Star",
        description: "Reach level 5",
        icon: "🌟",
    },
    AchievementDefinition {
        id: "level_10",
        name: "Companion Veteran",
        description: "Reach level 10",
        icon: "🏆",
    },
    AchievementDefinition {
        id: "night_owl",
        name: "Night Owl",
        description: "Code past midnight",
        icon: "🌙",
    },
    AchievementDefinition {
        id: "marathon",
        name: "Marathon Coder",
        description: "Complete a 4+ hour coding session",
        icon: "🏃",
    },
    AchievementDefinition {
        id: "goal_getter",
        name: "Goal Getter",
        description: "Complete your first daily objective",
        icon: "🎯",
    },
    AchievementDefinition {
        id: "break_master",
        name: "Break Master",
        description: "Take 10 breaks on schedule",
        icon: "☕",
    },
    AchievementDefinition {
        id: "century_coder",
        name: "Century Coder",
        description: "Log 100 hours of coding",
        icon: "💻",
    },
];

fn is_unlocked(connection: &Connection, id: &str) -> Result<bool, String> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM achievements WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(count > 0)
}

fn unlock(connection: &Connection, id: &str) -> Result<i64, String> {
    let now = chrono_now();
    connection
        .execute(
            "INSERT OR IGNORE INTO achievements (id, unlocked_at) VALUES (?1, ?2)",
            params![id, now],
        )
        .map_err(|error| error.to_string())?;
    Ok(connection.changes() as i64)
}

fn condition_met(connection: &Connection, id: &str) -> Result<bool, String> {
    match id {
        "first_session" => {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE ended_at IS NOT NULL",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(count >= 1)
        }
        "hydration_hero" => {
            let total: i64 = connection
                .query_row(
                    "SELECT COALESCE(SUM(hydration_acknowledged), 0) FROM daily_activity",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(total >= 10)
        }
        "streak_3" | "streak_7" | "streak_30" => {
            let required = match id {
                "streak_3" => 3,
                "streak_7" => 7,
                _ => 30,
            };
            let streak: i64 = connection
                .query_row("SELECT streak FROM pet_state WHERE id = 1", [], |row| row.get(0))
                .map_err(|error| error.to_string())?;
            Ok(streak >= required)
        }
        "level_5" | "level_10" => {
            let required = if id == "level_5" { 5 } else { 10 };
            let level: i64 = connection
                .query_row("SELECT level FROM pet_state WHERE id = 1", [], |row| row.get(0))
                .map_err(|error| error.to_string())?;
            Ok(level >= required)
        }
        "night_owl" => {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions
                     WHERE CAST(strftime('%H', started_at, 'unixepoch', 'localtime') AS INTEGER) >= 0
                       AND CAST(strftime('%H', started_at, 'unixepoch', 'localtime') AS INTEGER) < 5",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(count >= 1)
        }
        "marathon" => {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE duration_secs >= 14400",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(count >= 1)
        }
        "goal_getter" => {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM daily_objectives WHERE completed = 1",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(count >= 1)
        }
        "break_master" => {
            let total: i64 = connection
                .query_row(
                    "SELECT COALESCE(SUM(breaks_taken), 0) FROM daily_activity",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(total >= 10)
        }
        "century_coder" => {
            let total: i64 = connection
                .query_row(
                    "SELECT COALESCE(SUM(duration_secs), 0) FROM sessions",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            Ok(total >= 360_000)
        }
        _ => Ok(false),
    }
}

pub fn evaluate_achievements(
    _app: &AppHandle,
    connection: &Connection,
) -> Result<Vec<AchievementStatus>, String> {
    let mut unlocked_now = Vec::new();

    for definition in ACHIEVEMENTS {
        if is_unlocked(connection, definition.id)? {
            continue;
        }
        if !condition_met(connection, definition.id)? {
            continue;
        }
        if unlock(connection, definition.id)? == 0 {
            continue;
        }

        unlocked_now.push(AchievementStatus {
            id: definition.id.to_string(),
            name: definition.name.to_string(),
            description: definition.description.to_string(),
            icon: definition.icon.to_string(),
            unlocked: true,
            unlocked_at: Some(chrono_now()),
        });
    }

    Ok(unlocked_now)
}

pub fn get_achievements(connection: &Connection) -> Result<Vec<AchievementStatus>, String> {
    let mut results = Vec::new();

    for definition in ACHIEVEMENTS {
        let unlocked_at: Option<i64> = connection
            .query_row(
                "SELECT unlocked_at FROM achievements WHERE id = ?1",
                [definition.id],
                |row| row.get(0),
            )
            .ok();

        results.push(AchievementStatus {
            id: definition.id.to_string(),
            name: definition.name.to_string(),
            description: definition.description.to_string(),
            icon: definition.icon.to_string(),
            unlocked: unlocked_at.is_some(),
            unlocked_at,
        });
    }

    Ok(results)
}

pub fn get_achievements_command(app: AppHandle) -> Result<Vec<AchievementStatus>, String> {
    with_db(&app, get_achievements)
}
