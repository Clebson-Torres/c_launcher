# c_launcher 🚀

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Made with Tauri](https://img.shields.io/badge/Made%20with-Tauri-24C8DB?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)

A powerful, fast, and customizable application launcher for Windows built with Tauri and Rust.


## ✨ Features

### 🔍 Smart Search
- **Fuzzy file search** across Desktop, Documents, and Downloads
- **Application launcher** with common Windows apps
- **Executable search** in Program Files directories
- **Real-time results** with intelligent scoring

### ⚡ Custom Commands
c_launcher supports powerful custom commands with prefixes:

#### 🌐 Web Search
- `g: term` or `google: term` - Search on Google
- `yt: video` - Search on YouTube
- `wiki: topic` - Search on Wikipedia
- `reddit: term` or `r: /subreddit` - Search on Reddit
- `so: question` - Search on Stack Overflow
- `amazon: product` - Search on Amazon

#### 🛠️ Utilities
- `calc: 2+2` or `= 50*3` or just `10+5` - Built-in calculator
- `tr: hello world` - Translate with Google Translate
- `maps: location` or `m: address` - Open in Google Maps
- `mail: email@example.com` - Compose email

#### 💻 Developer Tools
- `gh: user/repo` - Open GitHub repository
- `gh: search term` - Search on GitHub

#### 💱 Conversions
- `100 usd to brl` - Currency conversion
- `10 km to miles` - Unit conversion

### ⌨️ Keyboard Shortcuts
- `Ctrl + Shift + Space` - Toggle launcher window
- `↑↓` - Navigate through results
- `Enter` - Open selected item
- `Esc` - Close launcher

### 🎨 Modern UI
- **Transparent background** with blur effect
- **Dark theme** optimized for focus
- **System tray integration** - runs in background
- **Always on top** - quick access when needed
- **Auto-start** with Windows (optional)

## 🚀 Getting Started

### Prerequisites
- Windows 10/11
- No additional runtime required (standalone executable)

### Installation

#### Option 1: Download Release
1. Go to [Releases]( https://github.com/Clebson-Torres/c_launcher/releases)
2. Download the latest `.msi` or `.exe` installer
3. Run the installer
4. Launch CLauncher from the system tray

#### Option 2: Build from Source

##### Requirements
- [Rust](https://rustup.rs/) (1.70 or higher)
- [Node.js](https://nodejs.org/) (18 or higher)
- [pnpm](https://pnpm.io/) or npm

##### Steps
```bash
# Clone the repository
git clone  https://github.com/Clebson-Torres/c_launcher.git
cd clauncher

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/`

## 📖 Usage Examples

```
# Search for files
notepad
chrome

# Google search
g: rust programming
google: tauri framework

# Calculator
calc: 15 * 8 + 20
= 2^8
100+50

# YouTube
yt: rust tutorial

# GitHub
gh: tauri-apps/tauri
gh: web assembly

# Wikipedia
wiki: rust programming language

# Translate
tr: hello world

# Maps
maps: New York
m: times square

# Currency conversion
100 usd to brl
50 eur to usd
```

## 🛠️ Tech Stack

- **Frontend:** TypeScript, HTML, CSS
- **Backend:** Rust
- **Framework:** [Tauri](https://tauri.app/) v2
- **Search:** Fuzzy matching with [fuzzy-matcher](https://github.com/lotabout/fuzzy-matcher)
- **File System:** [walkdir](https://github.com/BurntSushi/walkdir)

## 📁 Project Structure

```
clauncher/
├── src/                  # Frontend source
│   ├── main.ts          # Main TypeScript logic
│   ├── styles.css       # Styling
│   └── index.html       # Main HTML
├── src-tauri/           # Rust backend
│   ├── src/
│   │   └── main.rs      # Main Rust application
│   ├── icons/           # Application icons
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri configuration
└── README.md
```

## ⚙️ Configuration

### Custom Hotkey
Currently set to `Ctrl + Shift + Space`. To change, edit `src-tauri/src/main.rs`:

```rust
let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
```

### Common Apps
Edit the `get_common_apps()` function in `src-tauri/src/main.rs` to add your frequently used applications.

### Auto-start
The application can start automatically with Windows. This is configured during installation or can be toggled in settings.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.




**Made with ❤️ and Rust**
