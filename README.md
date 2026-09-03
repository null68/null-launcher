# null-launcher

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=fff)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=000)
![TypeScript](https://img.shields.io/badge/TypeScript-6-3178C6?logo=typescript&logoColor=fff)

A simple, fast Minecraft launcher.

## Features

- ⚡ **Lightweight & fast** — runs on Tauri (Rust + native WebView) instead of Electron, so it stays small and easy on RAM
- 📦 **Resumable downloads** — interrupted downloads of game files and libraries pick up where they left off
- 🎮 **Offline account support** — jump straight into singleplayer or your own servers
- 🎨 **Clean, no-nonsense UI**

## Tech Stack

| Layer    | Stack                          |
|----------|---------------------------------|
| Frontend | React 19 + TypeScript + Vite   |
| Backend  | Rust + Tauri v2                |
| Plugins  | `clipboard-manager`            |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS) + npm
- [Rust](https://www.rust-lang.org/tools/install) (stable, via `rustup`)
- Tauri's platform-specific system dependencies — full details in the [official prerequisites guide](https://tauri.app/start/prerequisites/), quick version below:

<details>
<summary><b>Linux</b></summary>

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Arch
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file \
  openssl appmenu-gtk-module libappindicator-gtk3 librsvg xdotool
```

</details>

<details>
<summary><b>Windows</b></summary>

- **Windows:** Microsoft C++ Build Tools (Visual Studio Build Tools → "Desktop development with C++" workload). WebView2 already ships with Windows 10/11.

</details>

### Build & Run

```bash
# Clone the repo
git clone https://github.com/null68/null-launcher.git
cd null-launcher

# Install JS dependencies
npm install

# Dev mode, with hot reload
npm run tauri dev

# Production build (installer/binary)
npm run tauri build
```

The finished app/installer lands in `src-tauri/target/release/bundle/`.

## Roadmap

- [ ] Premium (Microsoft) account support
- [x] Fabric mod loader support
- [x] Forge mod loader support
- [x] Neogorge mod loader support
- [x] Quilt mod loader support 
 
**Further down the line:**
- [ ] Modpack creation & support
- [ ] MacOS support

## Contributing

Issues and PRs are welcome if you want to help take it further.
