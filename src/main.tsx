import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { SettingsPanel } from "./features/settings/SettingsPanel";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyEleganceTheme } from "./features/customization/themeUtils";
import "./App.css";

const currentWindow = getCurrentWindow();
const isSettingsWindow = currentWindow.label === "settings";

applyEleganceTheme();

document.documentElement.classList.add(isSettingsWindow ? "settings-window-root" : "pet-window-root");
document.body.classList.add(isSettingsWindow ? "settings-window-body" : "pet-window-body");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isSettingsWindow ? <SettingsPanel /> : <App />}
  </React.StrictMode>,
);
