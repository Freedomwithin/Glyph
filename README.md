# GLYPH - High-Performance Text Editor

GLYPH is a high-performance, minimalist text editor developed in Rust. It utilizes a custom rendering engine to provide a zero-latency development environment, bypassing the overhead associated with Electron-based editors.

## Technical Architecture

### Sovereign Skia Rendering Engine
Powered by tiny-skia and cosmic-text, GLYPH renders text directly to a pixel buffer. This architecture ensures consistent 60fps performance and avoids the layout complexities of standard UI frameworks.

### Unicode Piece Table Buffer
The core data structure is a character-indexed Piece Table. This ensures O(1) edits and complete Unicode safety, preventing data corruption when handling multi-byte characters or complex international scripts.

### Project Navigation and Search
Integrated project-wide search is optimized for speed, allowing for rapid navigation through large codebases with minimal resource consumption.

## Features

- Multi-tab management for concurrent project handling.
- 100-step persistent Undo/Redo history.
- Tree-sitter powered syntax highlighting with the Sovereign Night palette.
- Dynamic font selection supporting professional monospace families (Fira Code, JetBrains Mono, Hack, etc.).
- Native OS window management for stable integration with desktop environments.
- Native file dialogs for standard system interoperability.

## Keybindings

| Action | Command |
|---|---|
| New Tab | Ctrl + N / Ctrl + T |
| Close Tab | Ctrl + W |
| Save File | Ctrl + S |
| Open File | Ctrl + O |
| Search Project | Ctrl + F |
| Select All | Ctrl + A |
| Undo / Redo | Ctrl + Z / Ctrl + Y |

## Installation

### Prerequisites (Linux)
Ensure the following system dependencies are installed:
```bash
sudo apt install build-essential libfontconfig1-dev libx11-dev libwayland-dev libxkbcommon-dev libgles2-mesa-dev
```

### Build from Source
```bash
git clone https://github.com/[your-repo]/glyph.git
cd glyph
cargo build --release
```

## Strategy and Vision
GLYPH is designed for developers who prioritize efficiency and architectural integrity. By minimizing latency and maximizing resource control, it serves as a robust tool for professional software engineering. Developed by Jonathon and Maya.
