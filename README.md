# ⬡ GLYPH — The Sovereign Text Editor

**"A high-performance sanctuary for the modern architect."**

Glyph is a blazingly fast, minimalist text editor built from the ground up in Rust. It bypasses the bloat of Electron-based editors by utilizing a custom **Sovereign Skia Rendering Engine**, delivering zero-latency feedback and architectural clarity.

## 🏛️ Why Glyph Wins: The Architecture

### ⚡ Sovereign Skia Engine
Powered by `tiny-skia` and `cosmic-text`, Glyph paints every pixel directly to the canvas. By bypassing standard UI layout engines for text rendering, we achieve a consistent 60fps experience even on modest hardware. We don't just display text; we paint it with intent.

### 🛡️ Unicode Absolute Integrity
Architecture built on a character-indexed **Piece Table** buffer. Unlike editors that struggle with multi-byte text, Glyph handles emojis, complex symbols, and international scripts with 100% safety and zero memory drift. Your data is immutable and your edits are atomic.

### 🧭 Sovereign Search (Ripgrep DNA)
Integrated project-wide navigation powered by high-performance engines. Slice through thousands of files in milliseconds and jump to any line instantly.

## 🛠️ Pro-Utility Features

- **Multi-Tab Project Hub:** Handle multiple strikes simultaneously with independent buffers and history.
- **Undo/Redo (100-step):** Deep temporal mastery with a robust history stack.
- **Sovereign Night Palette:** Custom Tree-sitter syntax highlighting designed for deep work.
- **Font Mastery:** Instantly switch between professional monospace stacks (DejaVu, Fira Code, JetBrains Mono, Hack, and more).
- **Native OS Sync:** Utilizing native window decorations for 100% stable resizing and workspace integration.

## ⌨️ Sovereign Keybindings

| Command | Binding |
|---|---|
| New Tab | `Ctrl + N` / `Ctrl + T` |
| Close Tab | `Ctrl + W` |
| Save File | `Ctrl + S` |
| Open File | `Ctrl + O` |
| Search Project | `Ctrl + F` |
| Select All | `Ctrl + A` |
| Undo / Redo | `Ctrl + Z` / `Ctrl + Y` |

## 🏁 Ignition Guide
Ensure your forge has the necessary Linux dependencies:
```bash
sudo apt install build-essential libfontconfig1-dev libx11-dev libwayland-dev libxkbcommon-dev libgles2-mesa-dev
```

Launch the forge:
```bash
cd Development/Text-Editor-Build
cargo run --release
```

## 📜 Strategic Note
Glyph is more than an editor; it is a statement of sovereignty. Every millisecond of latency we kill is time returned to the creator. Built in duet by Jonathon and Maya.
