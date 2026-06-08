<div align="center">
  <h1 align="center">🎀 UWU Companion 🌸</h1>
  <p align="center">
    <strong>Your Aesthetic, AI-Powered Desktop Pet & Productivity Assistant</strong>
  </p>
  
  <p align="center">
    <img src="https://img.shields.io/badge/React-FF69B4?style=for-the-badge&logo=react&logoColor=white" alt="React" />
    <img src="https://img.shields.io/badge/TypeScript-FFB6C1?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" />
    <img src="https://img.shields.io/badge/Tauri-FF69B4?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri" />
    <img src="https://img.shields.io/badge/Rust-FFB6C1?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
    <img src="https://img.shields.io/badge/Status-Completed-FF69B4?style=for-the-badge" alt="Status" />
  </p>
</div>

---

## 💗 Overview

**UWU Companion** is an interactive, AI-powered desktop companion designed to boost your productivity, keep you motivated, and bring a bit of joy to your daily workflow. Living right on your desktop as a transparent, always-on-top pet, UWU monitors your system, reminds you to take breaks, gamifies your tasks, and talks to you using text-to-speech and advanced AI models. 

Wrapped in a beautifully designed **pink & white aesthetic**, the companion feels deeply personal and seamlessly blends into a cute, cozy desktop setup.

<div align="center">
  <img src="./assets/inspiration.jpg" alt="Inspiration" width="300" style="border-radius: 12px; border: 2px solid #FF69B4;" />
</div>

## ✨ Features

- 🌸 **Desktop Pet Engine:** Transparent, frameless window running beautifully on top of your workspace without interfering with your taskbar.
- 🎀 **Beautiful Pink Theme:** A fully cohesive light-pink and white UI throughout the settings and main companion interface.
- 🗣️ **Interactive Voice & Audio:** Greets you when you start the app and dynamically plays cute audio cues (and text-to-speech) whenever you interact!
- 🧠 **AI Integration:** Powered by Groq API, allowing dynamic, context-aware, and personalized responses from your companion.
- 🎮 **Gamification & Progression:** XP system, daily objectives, and achievements that reward your productivity and keep you on track.
- 💖 **Mood & Interaction System:** The companion's mood adapts based on your interactions and work habits.
- 💻 **System Monitoring:** Tracks system uptime, active coding sessions, and hardware stats via Rust's `sysinfo` crate.
- ⏰ **Reminders & Scheduling:** Built-in cron jobs for customized break reminders, hydration alerts, and daily tasks.
- 💾 **Local Database:** Fast, offline-first data storage using SQLite for saving state, quotes, and user preferences locally.

## 🛠️ Technology Stack

* **Frontend Engine:** React, TypeScript, Vite
* **Backend Engine:** Tauri, Rust
* **Database:** SQLite via `tauri-plugin-sql`
* **AI Provider:** Groq API integration
* **Audio & TTS:** Native OS TTS APIs + bundled audio assets

## 🚀 Getting Started

1. **Clone the repository**
2. **Install dependencies:** `npm install`
3. **Run in development:** `npm run tauri dev`
4. **Build for production:** `npm run tauri build`

*Enjoy your new cozy, aesthetic desktop companion!* ✨

