# c_launcher 🚀

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Made with Tauri](https://img.shields.io/badge/Made%20with-Tauri-24C8DB?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)

A powerful, fast, and customizable application launcher for Windows, Linux and macOS built with Tauri and Rust.

## ✨ Features

### 🔍 Smart Search
- **Fuzzy file search** across Desktop, Documents, and Downloads
- **Application launcher** with common system apps
- **Executable search** in system directories
- **Real-time results** with intelligent scoring
- **Clipboard search** using `clip:` prefix
- **Command execution** using `>` prefix

### 📋 Clipboard History (Secure)
- Automatic clipboard monitoring in background
- Stores up to **30 recent entries**
- **Sensitive content detection** (tokens, passwords, API keys)
- Sensitive items are hidden from preview
- One-click restore to clipboard

### ⚡ Custom Commands
c_launcher supports powerful custom commands with prefixes:

#### 🌐 Web Search
- `g: term` or `google: term` – Search on Google
- `yt: video` – Search on YouTube

#### 🛠️ Utilities
- `calc: 2+2`
- `= 50*3`
- `10+5`
- Built-in calculator with expression parsing

#### 💻 Terminal
- `> ipconfig`
- `> ping google.com`
- Executes commands in **PowerShell / system shell**

### ⌨️ Keyboard Shortcuts
- `Ctrl + Shift + Space` – Toggle launcher window
- `↑ ↓` – Navigate through results
- `Enter` – Open selected item
- `Esc` – Close launcher

### 🧠 Smart Behavior
- Automatically hides when window loses focus
- Prioritizes results based on usage and relevance
- Lightweight background execution

### 🎨 Modern UI
- **Transparent background** with blur effect
- **Dark theme** optimized for focus
- **System tray integration** – runs in background
- **Always on top** – quick access when needed
- **Auto-start** with system (optional)

## 🚀 Getting Started

### Prerequisites
- Windows 10/11, Linux or macOS
- No additional runtime required (standalone executable)

### Installation

#### Option 1: Download Release
1. Go to [Releases](https://github.com/Clebson-Torres/c_launcher/releases)
2. Download the installer for your OS:
   - **Windows** (`.msi` / `.exe`)
   - **Linux** (`.AppImage` / `.deb`)
   - **macOS** (`.dmg`)
3. Run the installer
4. Launch c_launcher from the system tray / menu bar

#### Option 2: Build from Source

##### Requirements
- [Rust](https://rustup.rs/) (1.70 or higher)
- [Node.js](https://nodejs.org/) (18 or higher)
- [pnpm](https://pnpm.io/) or npm

##### Steps
```bash
git clone https://github.com/Clebson-Torres/c_launcher.git
cd clauncher
pnpm install
pnpm tauri dev
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/`

## 📖 Usage Examples

```
notepad
chrome
g: rust programming
calc: 10+5
clip: token
> ipconfig
yt: rust tutorial
```

## 🛠️ Tech Stack

- **Frontend:** TypeScript, HTML, CSS
- **Backend:** Rust
- **Framework:** Tauri v2

## 📝 License

MIT License
