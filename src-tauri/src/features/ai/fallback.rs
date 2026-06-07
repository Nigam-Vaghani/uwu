use crate::features::ai::context::AiContext;
use crate::features::events::PetEventType;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::HashMap;

const RULES_JSON: &str = include_str!("../../../resources/responses/rules.json");

#[derive(Debug, Deserialize)]
struct RulesFile {
    templates: HashMap<String, Vec<String>>,
}

pub struct FallbackProvider {
    rules: RulesFile,
}

impl FallbackProvider {
    pub fn new() -> Result<Self, String> {
        let rules: RulesFile =
            serde_json::from_str(RULES_JSON).map_err(|error| error.to_string())?;
        Ok(Self { rules })
    }

    pub fn complete_for_event(
        &self,
        event_type: PetEventType,
        context: &AiContext,
        payload: &crate::features::events::PetEventPayload,
    ) -> Result<String, String> {
        let key = event_key(event_type);
        let templates = self
            .rules
            .templates
            .get(key)
            .ok_or_else(|| format!("No fallback templates for {key}"))?;

        let mut rng = rand::thread_rng();
        let template = templates
            .choose(&mut rng)
            .ok_or_else(|| format!("No fallback templates for {key}"))?;

        Ok(substitute(template, context, payload))
    }
}

fn event_key(event_type: PetEventType) -> &'static str {
    match event_type {
        PetEventType::IdleDetected => "IdleDetected",
        PetEventType::ReminderFired => "ReminderFired",
        PetEventType::RandomMessage => "RandomMessage",
        PetEventType::UserInteraction => "UserInteraction",
        PetEventType::HighCpu => "HighCpu",
        PetEventType::HighRam => "HighRam",
        PetEventType::LowBattery => "LowBattery",
        PetEventType::CriticalBattery => "CriticalBattery",
        PetEventType::CodingSessionStarted => "CodingSessionStarted",
        PetEventType::CodingSessionEnded => "CodingSessionEnded",
        PetEventType::LevelUp => "LevelUp",
        PetEventType::AchievementUnlocked => "AchievementUnlocked",
        PetEventType::ObjectiveCompleted => "ObjectiveCompleted",
        PetEventType::StreakMilestone => "StreakMilestone",
    }
}

fn substitute(
    template: &str,
    context: &AiContext,
    payload: &crate::features::events::PetEventPayload,
) -> String {
    let cpu = format!("{:.0}", context.cpu_usage);
    let ram = format!("{:.0}", context.ram_usage);
    let battery = context
        .battery_level
        .map(|level| format!("{:.0}", level))
        .unwrap_or_else(|| "?".to_string());
    let app = context
        .active_app
        .clone()
        .unwrap_or_else(|| "your app".to_string());
    let value = payload
        .value
        .map(|v| format!("{v:.0}"))
        .unwrap_or_else(|| cpu.clone());
    let title = payload
        .title
        .clone()
        .unwrap_or_else(|| "Reminder".to_string());
    let message = payload.message.clone().unwrap_or_default();
    let editor = payload
        .app_name
        .clone()
        .unwrap_or_else(|| "your editor".to_string());

    template
        .replace("{pet_name}", &context.pet_name)
        .replace("{personality}", &context.personality)
        .replace("{mood}", &context.mood)
        .replace("{cpu}", &cpu)
        .replace("{ram}", &ram)
        .replace("{battery}", &battery)
        .replace("{app}", &app)
        .replace("{value}", &value)
        .replace("{title}", &title)
        .replace("{message}", &message)
        .replace("{editor}", &editor)
}
