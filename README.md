# BitFlow - Network Speed Monitor

**Version:** 1.0.0

A lightweight, real-time network speed monitor built with Tauri, React, and TypeScript. Monitors all physical network interfaces (Ethernet, Wi-Fi) and displays live upload/download speeds with beautiful visualizations.

![BitFlow Screenshot](https://github.com/qamarabbas408/bitflow/raw/main/screenshot.png)

## Features

### Core Functionality
- **Real-time monitoring** - Live network speed tracking for all physical interfaces
- **Visual dashboards** - Sparkline graphs and progress bars for each interface
- **System tray integration** - Animated tray icon showing current speeds
- **Offline detection** - Visual indicators for network connectivity status

### Interface Management
- **Welcome screen** - Initial setup guidance for new users
- **Settings window** - Select specific interfaces to monitor
- **Clear selection** - Reset interface monitoring preferences
- **Default behavior** - Monitor all physical interfaces if none selected

### Customization
- **Theme selection** - Dark and light themes with global application
- **Compact UI** - Widget-like window positioning at screen corner
- **Auto-hide** - Window hides to system tray when closed

### Technical Features
- **Cross-platform** - Works on Windows, macOS, and Linux
- **Lightweight** - Minimal resource usage
- **Persistent settings** - Preferences saved between sessions
- **Responsive design** - Adapts to different screen sizes

## Installation

### Prerequisites
- Node.js (v18 or later)
- Rust (stable)
- Tauri prerequisites: https://tauri.app/v1/guides/getting-started/prerequisites

### Development Setup
```bash
# Clone the repository
git clone https://github.com/qamarabbas408/bitflow.git
cd bitflow

# Install dependencies
npm install

# Run development server
npm run tauri dev
```

### Building for Production
```bash
# Build the application
npm run tauri build
```

## Usage

1. **Initial Launch** - Welcome screen guides you to select network interfaces
2. **Settings** - Click "Select Network Interfaces" or use system tray → Settings
3. **Theme** - Toggle between dark/light themes using the sun/moon button in header
4. **System Tray** - Right-click for options (Show/Hide, Settings, Quit)

## Technology Stack

- **Frontend**: React 19, TypeScript, Vite
- **Backend**: Rust, Tauri 2.0
- **State Management**: React hooks, Tauri store plugin
- **Styling**: CSS with CSS variables for theming
- **Build**: Vite for frontend, Cargo for Rust

## Project Structure

```
bitflow/
├── src/                # React frontend
│   ├── App.tsx         # Main application window
│   ├── Settings.tsx    # Settings window component
│   ├── App.css         # Global styles and themes
│   └── Settings.css    # Settings window styles
├── src-tauri/          # Rust backend
│   ├── src/lib.rs      # Tauri commands and network monitoring
│   ├── Cargo.toml      # Rust dependencies
│   └── tauri.conf.json # Tauri configuration
├── public/             # Static assets
└── package.json        # Frontend dependencies
```

## Configuration

- **Interface selection** - Saved in `.settings.dat` via Tauri store plugin
- **Theme preference** - Stored in same file, defaults to dark theme
- **Window position** - Remembered between sessions

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Network monitoring powered by [sysinfo](https://crates.io/crates/sysinfo) crate
- Icons and UI inspiration from modern dashboard designs

## Support

- **Issues**: https://github.com/qamarabbas408/bitflow/issues
- **Repository**: https://github.com/qamarabbas408/bitflow

---

**Note:** This application monitors network traffic. It only reads interface statistics and does not capture or inspect packet contents.
