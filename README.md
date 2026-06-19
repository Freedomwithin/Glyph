# Glyph - High-Performance Text Editor

Glyph is a high-performance, native text editor built in Rust. It uses a custom pixel-buffer rendering engine to deliver a zero-latency editing experience without the overhead of Electron or browser-based UI frameworks.

## Technical Architecture

### Custom Skia Rendering Engine
Powered by tiny-skia and cosmic-text, Glyph renders text directly to a pixel buffer. This ensures consistent performance and sidesteps the layout overhead of standard UI toolkits.

### Unicode Piece Table Buffer
The core data structure is a character-indexed Piece Table providing O(1) inserts and deletes with full Unicode safety. Multi-byte characters and international scripts are handled without data corruption.

### Tree-Sitter Syntax Highlighting
Syntax highlighting is driven by Tree-sitter, providing accurate, incremental, scope-aware parsing for Rust and other supported languages.

### Project Navigation and Search
Project-wide file search is built in and optimized for speed, enabling rapid navigation through large codebases with minimal resource consumption.

## Features

### Core (Free)
- Multi-tab file management
- 100-step undo/redo history
- Tree-sitter syntax highlighting
- Dynamic font selection (Fira Code, JetBrains Mono, Hack, and more)
- Project file tree with automatic root detection
- Project-wide search
- Native OS file dialogs
- Native window management

### Pro
- Pro themes (Sovereign Night, Temple)
- Session restore: reopen your last set of files automatically on launch
- Additional Pro features in development

## Licensing

Glyph Core is free. A Pro license key (format: `GLYPH-PRO-XXXX-XXXX`) unlocks Pro features via the Settings panel. Activation is persistent and stored locally.

## Keybindings

| Action | Shortcut |
|---|---|
| New Tab | Ctrl + N |
| Close Tab | Ctrl + W |
| Save File | Ctrl + S |
| Open File | Ctrl + O |
| Search Project | Ctrl + F |
| Select All | Ctrl + A |
| Undo | Ctrl + Z |
| Redo | Ctrl + Y |

## Installation

### Linux - AppImage (Recommended)

Download `Glyph-x86_64.AppImage` from the releases page, make it executable, and run:

```bash
chmod +x Glyph-x86_64.AppImage
./Glyph-x86_64.AppImage
```

### Build from Source (Linux)

Install system dependencies:
```bash
sudo apt install build-essential libfontconfig1-dev libx11-dev libwayland-dev libxkbcommon-dev libgles2-mesa-dev
```

Clone and build:
```bash
git clone https://github.com/Freedomwithin/Glyph.git
cd Glyph
cargo build --release
./target/release/glyph
```

### Windows

See `WINDOWS_BUILD_GUIDE.md` for full instructions. In brief: install Rust, Visual Studio Build Tools, and CMake, then run `cargo build --release`.

## Vision

Glyph is built for developers who value speed and architectural correctness. The goal is a professional-grade editor that is genuinely fast, genuinely native, and not dependent on a web runtime. Developed by Jonathon Koerner.
