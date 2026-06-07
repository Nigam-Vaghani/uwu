import { useState, useEffect } from "react";

import { getCurrentWindow } from "@tauri-apps/api/window";

import { listen } from "@tauri-apps/api/event";

import { useSettingsStore } from "../../store/settingsStore";

import { invokeCommand, useTauriStore } from "../../hooks/useTauriStore";

import { GeneralSettings } from "./GeneralSettings";

import { ReminderList } from "../reminders/ReminderList";

import { SystemMonitorWidget } from "../system-monitor/SystemMonitorWidget";

import { AISettings } from "../ai/AISettings";

import { VoiceSettings } from "./VoiceSettings";

import { ProgressPanel } from "../gamification/ProgressPanel";

import { CustomizationPanel } from "../customization/CustomizationPanel";

import { useGamification } from "../gamification/useGamification";

import { applyThemeById } from "../customization/themeUtils";

import { defaultConfig } from "../../store/settingsStore";



type SettingsTab = "general" | "reminders" | "system" | "ai" | "voice" | "progress" | "customize";



const TABS: { id: SettingsTab; label: string; hint: string }[] = [

  { id: "general", label: "General", hint: "Name & personality" },

  { id: "reminders", label: "Reminders", hint: "Schedules & alerts" },

  { id: "system", label: "System", hint: "CPU, RAM, battery" },

  { id: "ai", label: "AI", hint: "Providers & models" },

  { id: "voice", label: "Voice", hint: "Speech settings" },

  { id: "progress", label: "Progress", hint: "XP & achievements" },

  { id: "customize", label: "Customize", hint: "Skins & themes" },

];



export function SettingsPanel() {

  const [tab, setTab] = useState<SettingsTab>("general");

  const config = useSettingsStore((state) => state.config);

  const setConfig = useSettingsStore((state) => state.setConfig);

  const patchConfig = useSettingsStore((state) => state.patchConfig);

  const { loadConfig, saveConfig, loading } = useTauriStore();

  const gamification = useGamification();

  const activeTab = TABS.find((item) => item.id === tab) ?? TABS[0];



  useEffect(() => {

    const currentWindow = getCurrentWindow();

    void currentWindow.setTitle("UWU Companion Settings");



    void loadConfig().then((loaded) => {

      setConfig(loaded);

      void applyThemeById(loaded.activeTheme);

    });

  }, [loadConfig, setConfig]);



  useEffect(() => {

    const currentWindow = getCurrentWindow();

    const unlisten = currentWindow.onCloseRequested(async (event) => {

      event.preventDefault();

      await invokeCommand("close_settings", {});

    });



    return () => {

      void unlisten.then((release) => release());

    };

  }, []);



  useEffect(() => {

    const unlistenTheme = listen<string>("customization:theme-changed", (event) => {

      void applyThemeById(event.payload);

    });

    const unlistenImport = listen<typeof config>("customization:bundle-imported", (event) => {

      setConfig(event.payload);

      void applyThemeById(event.payload.activeTheme);

    });



    return () => {

      void unlistenTheme.then((release) => release());

      void unlistenImport.then((release) => release());

    };

  }, [setConfig]);



  const handleSave = async () => {

    await saveConfig(config);

    await invokeCommand("close_settings", {});

  };



  const handleCancel = async () => {

    const loaded = await loadConfig();

    setConfig(loaded);

    await invokeCommand("close_settings", {});

  };



  const handleReset = () => {

    setConfig({

      ...defaultConfig,

      hasSeenWelcome: config.hasSeenWelcome,

      position: config.position,

    });

  };



  return (

    <main className="settings-shell">

      <aside className="settings-sidebar">

        <div className="settings-brand">

          <span className="settings-brand-mark" aria-hidden="true">

            ✦

          </span>

          <div>

            <strong>UWU Companion</strong>

            <span>Settings</span>

          </div>

        </div>



        <nav className="settings-sidebar-nav" aria-label="Settings sections">

          {TABS.map((item) => (

            <button

              key={item.id}

              type="button"

              className={`settings-sidebar-link${tab === item.id ? " settings-sidebar-link--active" : ""}`}

              onClick={() => setTab(item.id)}

            >

              <span className="settings-sidebar-link-label">{item.label}</span>

              <span className="settings-sidebar-link-hint">{item.hint}</span>

            </button>

          ))}

        </nav>

      </aside>



      <section className="settings-main">

        <header className="settings-main-header">

          <div>

            <h1 className="settings-main-title">{activeTab.label}</h1>

            <p className="settings-main-subtitle">{activeTab.hint}</p>

          </div>

        </header>



        <div className="settings-card">

          {tab === "general" ? (

            <GeneralSettings config={config} onPatch={patchConfig} />

          ) : tab === "reminders" ? (

            <ReminderList />

          ) : tab === "system" ? (

            <SystemMonitorWidget />

          ) : tab === "ai" ? (

            <AISettings />

          ) : tab === "voice" ? (

            <VoiceSettings />

          ) : tab === "customize" ? (

            <CustomizationPanel />

          ) : (

            <ProgressPanel

              stats={gamification.stats}

              objectives={gamification.objectives}

              achievements={gamification.achievements}

              weeklySummary={gamification.weeklySummary}

              breakdown={gamification.breakdown}

              loading={gamification.loading}

              error={gamification.error}

              onRetry={gamification.refresh}

            />

          )}



          {tab === "general" ? (

            <div className="settings-actions">

              <button

                className="settings-secondary"

                type="button"

                data-element-id="cancel-btn"

                onClick={handleCancel}

                disabled={loading}

              >

                Cancel

              </button>

              <button

                className="settings-secondary"

                type="button"

                onClick={handleReset}

                disabled={loading}

              >

                Reset

              </button>

              <button

                className="settings-primary"

                type="button"

                data-element-id="save-btn"

                onClick={handleSave}

                disabled={loading}

              >

                Save

              </button>

            </div>

          ) : null}

        </div>

      </section>

    </main>

  );

}


