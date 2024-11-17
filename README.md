# ServeLite

A minimalist system tray application for easily sharing local directories across different operating systems.

## Features

- One-click directory sharing via system tray
- Cross-platform support (Windows, macOS, Linux)
- Zero configuration required
- Native system integration
- Local network file serving

## Installation

Download the appropriate installer for your platform from the [releases page](https://github.com/daniissac/servelite/releases).

### Windows
- Download and run `ServeLite_x64.msi` or `ServeLite_x64-setup.exe`

### macOS
- Download and open `ServeLite_x64.dmg`
- Drag ServeLite to your Applications folder

### Linux
- Download and install either:
  - `servelite_amd64.deb` for Debian/Ubuntu
  - `servelite_amd64.AppImage` for other distributions

## Usage

1. Launch ServeLite - it will appear in your system tray
2. Click the tray icon to show the menu
3. Select "Start Server" and choose a directory to serve
4. Access your files at http://localhost:8000
5. Use "Stop Server" to stop serving files
6. Use "Quit" to exit the application

## Development

### Prerequisites
- Rust
- Node.js
- Tauri CLI

### Setup
```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## License

MIT License
