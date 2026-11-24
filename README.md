# TLyrics — Universal Synchronized Lyrics & Touch Bar Companion for macOS

<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="TLyrics Logo" width="100" />
</p>

<p align="center">
  <strong>A lightweight, menu-bar-only macOS companion that displays real-time synchronized lyrics on your screen and physical MacBook Touch Bar.</strong>
</p>

<p align="center">
  <a href="https://github.com/Teddir/tlyrics/releases"><img src="https://img.shields.io/badge/platform-macOS%2013%2B-blue?logo=apple" alt="Platform: macOS 13+"></a>
  <a href="https://github.com/Teddir/tlyrics"><img src="https://img.shields.io/badge/arch-Apple%20Silicon%20%7C%20Intel-success" alt="Apple Silicon & Intel"></a>
  <a href="https://tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8DB?logo=tauri" alt="Tauri 2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust%202024-DEA584?logo=rust" alt="Rust"></a>
  <a href="https://lrclib.net/"><img src="https://img.shields.io/badge/lyrics-LRCLIB-blueviolet" alt="LRCLIB"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="MIT License"></a>
</p>

---

## 📖 Overview

**TLyrics** is a native macOS background application and synchronized lyrics engine designed with a **Spotube-first** architecture and universal macOS playback detection. Built with **Tauri 2**, **Rust**, **AppKit / Swift**, and vanilla web technologies, TLyrics delivers synchronized lyrics straight to a minimal companion window and your physical **MacBook Touch Bar** without draining system resources.

TLyrics runs as a **macOS Agent / Menu Bar application** (`LSUIElement = true`): it lives entirely in your menu bar and background, completely free of Dock clutter.

---

## ✨ Features

| Feature | Description |
| :--- | :--- |
| **🎵 Spotube-First & Universal Detection** | Dedicated zero-latency position tracking for Spotube with automatic fallback to macOS System Media (Apple Music, Spotify, browsers). |
| **📜 Synchronized LRCLIB Retrieval** | Automatic fetching, subtitle cleaning, fuzzy title matching, chronological LRC parsing, and local disk/memory caching. |
| **💻 Global Physical Touch Bar Integration** | Native Swift & AppKit `NSTouchBar` (`DFRFoundation`) integration displaying current and upcoming lyrics across any active application (Safari, Terminal, VS Code, Finder). |
| **🔌 Third-Party Touch Bar IPC** | Continuous background streaming to `~/.cache/tlyrics/current_lyric.txt` and `touchbar.json` for one-click **BetterTouchTool (BTT)**, **MTMR**, or **Pock** integration. |
| **🎛️ Menu Bar Only / Dock-Free Agent** | Operates as a true background accessory (`LSUIElement = true`) without a Dock icon. Window hide-on-close ensures persistent monitoring. |
| **🎨 Minimal Lyrics Companion UI** | Distraction-free 3-line focus layout (previous, current, next) with Apple system typography, smooth transitions, and customizable font size and timing offset (+/- ms). |
| **⚡ High Performance & Low Overhead** | Minimal CPU footprint, debounced metadata tracking, lazy-loaded artwork, and zero-waste DOM rendering. |

---

## 🏗️ Architecture & How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│                     Audio & Music Sources                       │
│    Spotube (Dedicated)  │  Apple Music  │  Spotify  │  Browsers │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
               macOS Native MediaRemote & Now Playing
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                   TLyrics Rust Backend (Tauri 2)                │
│  • PlaybackManager (Spotube-First Priority Routing)             │
│  • LyricsEngine (LRCLIB Client + Chronological Parser + Cache)  │
│  • SettingsManager (Persistent Preferences & Launch at Login)   │
└───────────────┬─────────────────┬───────────────────┬───────────┘
                │                 │                   │
                ▼                 ▼                   ▼
      ┌──────────────────┐ ┌───────────────┐ ┌───────────────────┐
      │ Physical TouchBar│ │ Background IPC│ │ Minimal Companion │
      │   (Swift/AppKit) │ │ (BTT/MTMR/Pock│ │   (Vanilla UI)    │
      └──────────────────┘ └───────────────┘ └───────────────────┘
```

---

## 💻 System Requirements

* **Operating System:** macOS 13.0 (Ventura), macOS 14.0 (Sonoma), or macOS 15.0 (Sequoia) or later.
* **Architecture:** Apple Silicon (M1/M2/M3/M4) or Intel (x86_64).
* **Hardware:** Any Mac (MacBook Pro with physical Touch Bar recommended for Touch Bar features).
* **Music Player:** [Spotube](https://spotube.krtirtho.dev/) (recommended), Apple Music, Spotify, or any media player supporting macOS Now Playing.

---

## 🚀 Cloning and Building from Source

### 1. Prerequisites

Ensure you have the required developer tools installed on your Mac:

```bash
# 1. Install Xcode Command Line Tools
xcode-select --install

# 2. Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 3. Install Tauri CLI v2
cargo install tauri-cli --version "^2.0"
```

### 2. Clone the Repository

```bash
git clone https://github.com/Teddir/tlyrics.git
cd tlyrics
```

### 3. Run in Development Mode

To start the application in hot-reloading development mode:

```bash
cargo tauri dev
```

*Note: In development mode, macOS activation policy automatically hides the Dock icon and places TLyrics in the menu bar.*

### 4. Run Unit Tests

TLyrics includes unit tests covering playback priority, track fingerprinting, LRC parsing, timing offset, and LRCLIB integration:

```bash
cd src-tauri
cargo test -- --nocapture
```

### 5. Build the Packaged macOS Application (`.app`)

To produce the standalone macOS application bundle with full `LSUIElement` agent metadata:

```bash
# Debug bundle (fast build)
cargo tauri build -b app --debug

# Production release bundle (optimized with LTO)
cargo tauri build -b app
```

The compiled application bundle will be located at:
```bash
src-tauri/target/release/bundle/macos/tlyrics.app
# or for debug builds:
src-tauri/target/debug/bundle/macos/tlyrics.app
```

### 6. Install to Applications

You can copy the built application directly into your `/Applications` directory:

```bash
cp -r src-tauri/target/release/bundle/macos/tlyrics.app /Applications/
open /Applications/tlyrics.app
```

---

## 🛠️ Usage & Integration

### Menu Bar Controls
Click the **TLyrics** icon in the macOS menu bar to access:
* **Open Lyrics / Show/Hide Window:** Toggle the companion overlay.
* **Current Track Information:** Real-time track title and artist name.
* **Settings:** Configure timing offset, font size, start hidden, and autostart.
* **Enable/Disable Touch Bar:** Toggle the native global Touch Bar mirror.
* **Restart Lyrics Engine:** Flush cached track data and re-fetch lyrics from LRCLIB.
* **Quit TLyrics:** Completely stops background workers, releases IPC handles, and terminates cleanly.

### BetterTouchTool (BTT) & MTMR Integration
If you use third-party Touch Bar managers like [BetterTouchTool](https://folivora.ai/) or [MTMR](https://github.com/Toxblh/MTMR), TLyrics automatically streams the active lyric and next lyric to:
* **Text stream:** `~/.cache/tlyrics/current_lyric.txt`
* **JSON state:** `~/.cache/tlyrics/touchbar.json`

**Sample BTT Shell Script Widget:**
```bash
cat ~/.cache/tlyrics/current_lyric.txt 2>/dev/null || echo "♪ TLyrics"
```

---

## 🗺️ Project Roadmap

- [x] **Phase 1:** Spotube-specific playback detection via macOS Now Playing.
- [x] **Phase 2:** Universal playback provider architecture with priority routing.
- [x] **Phase 3:** LRCLIB integration, chronological LRC parsing, and local caching.
- [x] **Phase 4:** Native macOS Touch Bar integration (`AppKit` / `DFRFoundation`) & IPC streaming.
- [x] **Phase 5:** Minimalist macOS dark companion interface & timing offset control.
- [x] **Phase 6:** Background agent operation & Dock-free Menu Bar application (`LSUIElement`).
- [ ] **Phase 7:** Romanized / translation lyrics support & multi-source lyrics fallbacks.

---

## 🤝 Contributing

Contributions, bug reports, and suggestions are warmly welcome!
1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'feat: add amazing feature'`).
4. Push to the branch (`git push origin feature/amazing-feature`).
5. Open a Pull Request.

---

## 🌱 Purpose & Intention

As a Muslim developer, I believe technology is an *amanah* (trust) and an opportunity to create something genuinely beneficial for others.

TLyrics is developed with the hope of making audio content and beneficial words more accessible:

* **Ilmu yang Bermanfaat (علم نافع):** Making knowledge and lyrics easier to follow.
* **Khidmah (خدمة):** Creating purposeful technology that serves daily lives.
* **Ikhlas (إخلاص):** Developing and sharing with sincere intentions.
* **Amal Jariyah (صدقة جارية):** Hoping the positive benefits of this work continue to help others.

### 🤲 A Prayer

<p align="center">
  <strong>اللَّهُمَّ اجْعَلْ هَذَا الْعَمَلَ عِلْمًا نَافِعًا وَعَمَلًا صَالِحًا</strong><br>
  <em>"O Allah, make this work beneficial knowledge and a righteous deed."</em>
</p>

---

## 📜 License & Copyright

Distributed under the **MIT License**. See [`LICENSE`](file:///Users/mac/Documents/code/tlyrics/LICENSE) for more information.

Copyright © 2026 **Teddi_r**. All rights reserved.

<p align="center">
  <strong>Built with purpose. Designed for elegance. Shared for the benefit of others.</strong>
</p>
