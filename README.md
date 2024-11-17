# ServeLite

🚀 The simplest way to start a development server! Perfect for web developers who just want to focus on coding.

## Why ServeLite?

- 🎯 **Zero Configuration**: Just click and serve - no complex setup needed
- 🔄 **Live Reload**: Changes appear instantly in your browser
- 🎨 **Perfect for Web Development**: HTML, CSS, JavaScript - just save and see
- 🔌 **Works Everywhere**: Windows, Mac, or Linux - same easy experience
- 🎉 **Beginner Friendly**: No command line needed!

## Quick Start

1. Download ServeLite from our [releases page](https://github.com/daniissac/servelite/releases)
2. Install it like any other app
3. Click the ServeLite icon in your system tray
4. Choose your project folder
5. Start coding! 

Your site is instantly available at `http://localhost:8000` (or another port if 8000 is busy)

## Example Projects

Check out our example projects in the `examples/` directory:

### Basic Website (`examples/basic-website`)
A simple website demonstrating:
- Live reload functionality
- Basic HTML/CSS structure
- Time display using JavaScript
- Feature testing components

Try it out:
1. Download ServeLite
2. Open the `examples/basic-website` folder
3. Watch changes appear instantly as you edit!

## Perfect For...

- 🎓 Learning HTML, CSS, and JavaScript
- 💻 Building static websites
- 🛠️ Testing web components
- 📱 Viewing your site on mobile devices (same network)
- 🎨 Previewing design changes in real-time

## Features That Make Development Easy

- **Auto Port Finding**: Don't worry about port conflicts
- **Recent Projects**: Quick access to your frequent folders
- **Copy URL**: Share your work with teammates on the same network
- **System Tray**: Always there when you need it, out of your way when you don't
- **Live Reload**: See changes instantly without manual refresh

## Installation

### Windows
- Download `ServeLite_x64.msi` or `ServeLite_x64-setup.exe`
- Double-click to install
- That's it! Look for the ServeLite icon in your system tray

### macOS
- Download `ServeLite_x64.dmg`
- Drag to Applications
- Works great on both Intel and M1/M2 Macs!

### Linux
Choose either:
- `servelite_amd64.deb` for Ubuntu/Debian
- `servelite_amd64.AppImage` for other distributions

## Tips for New Developers

1. **Starting a Project**:
   - Create a new folder for your project
   - Add an `index.html` file
   - Select the folder in ServeLite
   - Start coding!

2. **Using Live Reload**:
   - Make changes to your files
   - Save them
   - Watch your browser update automatically!

3. **Testing on Mobile**:
   - Start ServeLite
   - Copy the URL (it shows your computer's network address)
   - Open it on your phone (while on the same WiFi)
   - Perfect for responsive design testing!

## Project Structure

```
servelite/
├── examples/              # Example projects to get started
│   └── basic-website/    # Simple website with live reload demo
├── src-tauri/            # Desktop app backend (Rust)
├── ui/                   # Core app interface
├── package.json          # Node.js dependencies
└── README.md            # You are here!
```

## Common Questions

**Q: Do I need to install anything else?**
A: Nope! ServeLite is all you need to start serving files.

**Q: What file types can I serve?**
A: Any web files! HTML, CSS, JavaScript, images, and more.

**Q: Can I use it with React/Vue/Angular development?**
A: For those frameworks, we recommend using their dedicated dev servers. ServeLite is perfect for simple web projects!

**Q: How do I stop the server?**
A: Just click "Stop Server" in the tray menu. Your ports are freed up immediately.

## For Advanced Users

If you're comfortable with development tools, ServeLite also supports:
- Cross-platform development
- Network access for local testing
- Custom port selection (8000-8099 range)
- Integration with your existing workflow

## Contributing

Want to help make ServeLite even better for new developers?
1. Fork the repo
2. Make your changes
3. Submit a pull request

## License

MIT License - Use it freely in your projects!
