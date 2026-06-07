import type { AppConfig } from "../store/settingsStore";
import type { MoodState, PetMood } from "../features/mood/mood.types";
import type {
  CreateReminderInput,
  Reminder,
  UpdateReminderInput,
} from "../features/reminders/reminders.types";
import type {
  CodingSession,
  SystemStats,
} from "../features/system-monitor/system-monitor.types";
import type {
  AiCompleteResponse,
  AiConfig,
  AiConnectionTestResult,
  SetAiConfigInput,
} from "../features/ai/ai.types";
import type {
  SetTtsConfigInput,
  TtsConfig,
  TtsProvider,
  VoiceInfo,
} from "../features/speech/speech.types";
import type { PetEventPayload as AiPetEventPayload } from "../features/events/eventEngine";
import type {
  AchievementStatus,
  DailyObjective,
  PetStats,
  ProductivityBreakdown,
  WeeklySummary,
} from "../features/gamification/gamification.types";
import type {
  PersonalityInfo,
  ResolvedSkinManifest,
  SkinManifest,
  SoundPackManifest,
} from "../features/customization/customization.types";
import type { QuoteConfig, QuotePayload } from "../features/quotes/quote.types";

export type WindowPosition = {
  x: number;
  y: number;
};

export type TauriCommands = {
  get_config: {
    args: Record<string, never>;
    result: AppConfig;
  };
  set_config: {
    args: { config: AppConfig };
    result: AppConfig;
  };
  get_window_position: {
    args: Record<string, never>;
    result: WindowPosition;
  };
  set_window_position: {
    args: { position: WindowPosition };
    result: void;
  };
  open_settings: {
    args: Record<string, never>;
    result: void;
  };
  close_settings: {
    args: Record<string, never>;
    result: void;
  };
  quit_app: {
    args: Record<string, never>;
    result: void;
  };
  get_reminders: {
    args: Record<string, never>;
    result: Reminder[];
  };
  create_reminder: {
    args: { input: CreateReminderInput };
    result: Reminder;
  };
  update_reminder: {
    args: { input: UpdateReminderInput };
    result: Reminder;
  };
  delete_reminder: {
    args: { id: number };
    result: void;
  };
  toggle_reminder: {
    args: { id: number };
    result: Reminder;
  };
  get_mood: {
    args: Record<string, never>;
    result: MoodState;
  };
  update_mood: {
    args: { mood: PetMood };
    result: MoodState;
  };
  record_interaction: {
    args: Record<string, never>;
    result: void;
  };
  reconcile_mood: {
    args: Record<string, never>;
    result: MoodState;
  };
  get_system_stats: {
    args: Record<string, never>;
    result: SystemStats;
  };
  get_coding_sessions: {
    args: Record<string, never>;
    result: CodingSession[];
  };
  get_uptime: {
    args: Record<string, never>;
    result: number;
  };
  ai_complete: {
    args: { payload: AiPetEventPayload };
    result: AiCompleteResponse;
  };
  test_ai_connection: {
    args: Record<string, never>;
    result: AiConnectionTestResult;
  };
  get_ai_config: {
    args: Record<string, never>;
    result: AiConfig;
  };
  set_ai_config: {
    args: { input: SetAiConfigInput };
    result: AiConfig;
  };
  emit_pet_interaction: {
    args: Record<string, never>;
    result: void;
  };
  speak: {
    args: { text: string; provider?: TtsProvider | null };
    result: void;
  };
  stop_speaking: {
    args: Record<string, never>;
    result: void;
  };
  get_tts_config: {
    args: Record<string, never>;
    result: TtsConfig;
  };
  set_tts_config: {
    args: { input: SetTtsConfigInput };
    result: TtsConfig;
  };
  list_voices: {
    args: { provider: TtsProvider };
    result: VoiceInfo[];
  };
  get_pet_stats: {
    args: Record<string, never>;
    result: PetStats;
  };
  get_daily_objectives: {
    args: Record<string, never>;
    result: DailyObjective[];
  };
  get_achievements: {
    args: Record<string, never>;
    result: AchievementStatus[];
  };
  get_weekly_summary: {
    args: Record<string, never>;
    result: WeeklySummary;
  };
  get_productivity_breakdown: {
    args: Record<string, never>;
    result: ProductivityBreakdown;
  };
  award_xp: {
    args: { input: { amount: number; reason: string } };
    result: {
      stats: PetStats;
      leveledUp: boolean;
      newLevel: number | null;
      achievementsUnlocked: AchievementStatus[];
    };
  };
  acknowledge_reminder: {
    args: { input: { reminderId: number; category: string } };
    result: {
      stats: PetStats;
      leveledUp: boolean;
      newLevel: number | null;
      achievementsUnlocked: AchievementStatus[];
    };
  };
  list_skins: {
    args: Record<string, never>;
    result: SkinManifest[];
  };
  get_skin_manifest: {
    args: { skinId: string };
    result: ResolvedSkinManifest;
  };
  set_active_skin: {
    args: { skinId: string };
    result: AppConfig;
  };
  list_sound_packs: {
    args: Record<string, never>;
    result: SoundPackManifest[];
  };
  set_active_sound_pack: {
    args: { packId: string };
    result: AppConfig;
  };
  set_active_theme: {
    args: { themeId: string };
    result: AppConfig;
  };
  list_personalities: {
    args: Record<string, never>;
    result: PersonalityInfo[];
  };
  import_uwu_bundle: {
    args: { inputPath: string };
    result: AppConfig;
  };
  export_uwu_bundle: {
    args: { outputPath: string };
    result: string;
  };
  get_quote_config: {
    args: Record<string, never>;
    result: QuoteConfig;
  };
  set_quote_config: {
    args: { config: QuoteConfig };
    result: QuoteConfig;
  };
  preview_quote: {
    args: { quoteType: string };
    result: QuotePayload;
  };
};

export type TauriCommand = keyof TauriCommands;

export type TauriCommandArgs<T extends TauriCommand> = TauriCommands[T]["args"];
export type TauriCommandResult<T extends TauriCommand> = TauriCommands[T]["result"];
