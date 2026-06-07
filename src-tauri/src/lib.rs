mod features;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = features::database::migrations();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(features::database::DB_URI, migrations)
                .build(),
        )
        .setup(|app| {
            let env_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.env");
            let _ = dotenvy::from_path(env_path);

            features::database::init(app.handle())?;
            features::window::restore_main_window_position(app.handle());

            let now = features::database::chrono_now();
            let interaction = std::sync::Arc::new(features::reminders::InteractionState::new(now));
            app.manage(interaction.clone());
            features::reminders::start_scheduler(app.handle().clone(), interaction);

            app.manage(features::ai::RecentEventsState::new());
            app.manage(features::system_monitor::SystemMonitorState::new());
            app.manage(features::tts::TtsState::new());
            features::system_monitor::start_monitor(app.handle().clone());
            features::gamification::init(app.handle())?;
            features::quotes::init(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            features::config::get_config,
            features::config::set_config,
            features::window::get_window_position,
            features::window::set_window_position,
            features::window::open_settings,
            features::window::close_settings,
            features::window::quit_app,
            features::reminders::get_reminders,
            features::reminders::create_reminder,
            features::reminders::update_reminder,
            features::reminders::delete_reminder,
            features::reminders::toggle_reminder,
            features::reminders::get_mood,
            features::reminders::update_mood,
            features::reminders::record_interaction,
            features::reminders::reconcile_mood,
            features::system_monitor::get_system_stats,
            features::system_monitor::get_coding_sessions,
            features::system_monitor::get_uptime,
            features::ai::ai_complete,
            features::ai::test_ai_connection,
            features::ai::get_ai_config,
            features::ai::set_ai_config,
            features::events::emit_pet_interaction,
            features::tts::speak,
            features::tts::stop_speaking,
            features::tts::get_tts_config,
            features::tts::set_tts_config,
            features::tts::list_voices,
            features::gamification::get_pet_stats,
            features::gamification::get_daily_objectives,
            features::gamification::get_achievements,
            features::gamification::get_weekly_summary,
            features::gamification::get_productivity_breakdown,
            features::gamification::award_xp,
            features::gamification::acknowledge_reminder,
            features::customization::list_skins,
            features::customization::get_skin_manifest,
            features::customization::set_active_skin,
            features::customization::list_sound_packs,
            features::customization::set_active_sound_pack,
            features::customization::set_active_theme,
            features::customization::list_personalities,
            features::customization::import_uwu_bundle,
            features::customization::export_uwu_bundle,
            features::quotes::get_quote_config,
            features::quotes::set_quote_config,
            features::quotes::preview_quote,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
