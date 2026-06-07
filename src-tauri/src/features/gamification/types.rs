use serde::{Deserialize, Serialize};

pub const XP_CODING_PER_30_MIN: i64 = 50;
pub const XP_OBJECTIVE_COMPLETE: i64 = 100;
pub const XP_HYDRATION_ACK: i64 = 10;
pub const XP_BREAK_ON_TIME: i64 = 20;
pub const XP_STREAK_DAILY: i64 = 25;
pub const XP_ACHIEVEMENT: i64 = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetStats {
    pub xp: i64,
    pub level: i64,
    pub streak: i64,
    pub productivity_score: i64,
    pub xp_into_level: i64,
    pub xp_for_next_level: i64,
    pub level_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyObjective {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub category: String,
    pub target: i64,
    pub progress: i64,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub unlocked: bool,
    pub unlocked_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummary {
    pub week_start: String,
    pub coding_minutes: i64,
    pub score: i64,
    pub objectives_completed: i64,
    pub achievements_unlocked: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductivityBreakdown {
    pub coding: i64,
    pub breaks: i64,
    pub hydration: i64,
    pub goals: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwardXpResult {
    pub stats: PetStats,
    pub leveled_up: bool,
    pub new_level: Option<i64>,
    pub achievements_unlocked: Vec<AchievementStatus>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwardXpInput {
    pub amount: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgeReminderInput {
    pub reminder_id: i64,
    pub category: String,
}

pub fn xp_required_for_level_up(from_level: i64) -> i64 {
    100 * from_level * from_level
}

pub fn total_xp_for_level(level: i64) -> i64 {
    if level <= 1 {
        return 0;
    }
    (1..level).map(xp_required_for_level_up).sum()
}

pub fn level_from_xp(xp: i64) -> i64 {
    let mut level = 1i64;
    while total_xp_for_level(level + 1) <= xp {
        level += 1;
    }
    level
}

pub fn level_progress(xp: i64, level: i64) -> (i64, i64) {
    let base = total_xp_for_level(level);
    let next = total_xp_for_level(level + 1);
    (xp - base, next - base)
}

pub fn level_name(level: i64) -> String {
    match level {
        1..=2 => "Companion Newcomer".to_string(),
        3..=5 => "Companion Apprentice".to_string(),
        6..=9 => "Companion Partner".to_string(),
        10..=19 => "Companion Veteran".to_string(),
        _ => "Companion Legend".to_string(),
    }
}

pub fn today_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

pub fn week_start_date() -> String {
    use chrono::{Datelike, Duration, Local};
    let today = Local::now().date_naive();
    let weekday = today.weekday().num_days_from_monday();
    (today - Duration::days(weekday as i64))
        .format("%Y-%m-%d")
        .to_string()
}
