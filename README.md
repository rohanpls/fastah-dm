# FastahDM

A fast, modern download manager built with Tauri 2, Vue 3, and Rust.

![FastahDM](https://img.shields.io/badge/Made%20with-Tauri-purple) ![License](https://img.shields.io/badge/License-AGPL--3.0-blue)

## Features

### Download Management
- **Fast Downloads** - Multi-threaded download engine written in Rust
- **Resume Support** - Pause and resume downloads with ETag support
- **Real-time Progress** - Live speed tracking and time estimates
- **Duplicate Detection** - Smart handling of duplicate files (replace or keep both)
- **Download History** - Persistent storage of all downloads

### User Interface
- **Modern Design** - Clean interface with light and dark themes
- **File Type Icons** - Color-coded icons for different file types
- **Search & Filter** - Easily find downloads with search and category filters
- **Storage Monitor** - Real-time disk space tracking

### Customization
- **Theme Toggle** - Switch between light and dark modes
- **Launch on Startup** - Auto-start with your system
- **Custom Keybind** - Configurable global shortcut to show/hide window
- **Native Performance** - Built with Tauri for minimal resource usage

## Screenshots

<p align="center">
  <img width="49%" alt="Downloads screen" src="https://github.com/user-attachments/assets/bafce47f-944d-4e3e-84be-11c1a05695ce" />
  <img width="49%" alt="New Download screen" src="https://github.com/user-attachments/assets/97cc430f-4989-4e78-801d-2530d1de9175" />
</p>

## Installation

### Download Pre-built Binary

Download the latest release from the [Releases](https://github.com/rohanpls/fastah-dm/releases) page.

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [pnpm](https://pnpm.io/) (v8 or later)
- [Rust](https://rustup.rs/) (stable)
- Platform-specific dependencies:
  - **Windows**: Microsoft Visual Studio C++ Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `build-essential`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/rohanpls/fastah-dm.git
cd fastah-dm

# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Build for production (creates optimized executable)
pnpm tauri build
```

The built executable will be in `src-tauri/target/release/` (or `src-tauri/target/release/bundle/` for installers).

## Usage

1. Click "New Download" to add a download URL
2. Select a destination folder (saved as default for future downloads)
3. Monitor progress in real-time with speed and time estimates
4. Use category filters to view All, Active, or Completed downloads
5. Search downloads by filename or URL
6. Access Settings to customize theme, startup behavior, and keybinds

## Configuration

Settings can be accessed from the sidebar:
- **Theme**: Choose between light and dark modes
- **Launch on Startup**: Auto-start FastahDM with your system
- **Keybind**: Configure global shortcut (default: `Ctrl+Shift+D`)
- **Download History**: Clear all history when needed

## Known Issues

- Keybind change doesn't work correctly

## Contributing

Feature requests and bug reports are welcome! Please open an issue on GitHub.

## Tech Stack

- **Frontend**: Vue 3 with TypeScript
- **State Management**: Pinia
- **Backend**: Rust with Tokio for async operations
- **Framework**: Tauri 2
- **UI**: Custom CSS with theme system

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Author

Created by [@rohanpls](https://github.com/rohanpls)
