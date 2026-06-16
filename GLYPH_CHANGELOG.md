# Glyph: The Sovereign Changelog

## [06.12.2026] — The Skia & Hub Strike

### 🏛️ Architecture
- **Sovereign Skia Engine:** Successfully transitioned from high-level Slint layout to a custom pixel-painter architecture using `tiny-skia` and `cosmic-text`.
- **Project Hub:** Refactored the core into a multi-tab system with dynamic file tree navigation.
- **Sovereign Search:** Integrated Ripgrep DNA (`grep`/`ignore` crates) for project-wide traversal.

---

## [06.13.2026] — Strategic Calibration & High-Fidelity Polish

### 🎯 Strategy
- **Marketing:** Anchored the 'Sovereign Open-Core' strategy in `marketing.md`.
- **Roadmap:** Defined Phases 12–15 (Color, Scrolling, Somatic Blink, AI Bridge).

### ✨ Visuals
- **Somatic Pulse:** Implemented 500ms `slint::Timer` for blinking cursor synchronization.
- **Visual Scrollbar:** Added "Sovereign Night" viewport scrollbar with dynamic height scaling.

### 🛠️ Functionality
- **Viewport Mastery:** Fully implemented `scroll_offset` in the Skia renderer to support large-file navigation.
- **Tab Management:** Added "X" close buttons and refactored the Project Hub to handle tab removal with automatic focus shifting.
- **Syntax Palette:** Completed Phase 12; mapped full Rust grammar to Mauve, Green, and Blue.

---

## [06.14.2026] — Presidential Polish & Prototype Finalization

### ✨ Visuals
- **README Upgrade:** Refactored the documentation to a "Presidential" standard, highlighting the Sovereign Feature Suite (Zero-Latency Skia, Unicode Absolute).
- **Z-Order Calibration:** Refined the custom title bar hit-boxes to resolve the window-maximize control conflict.

### 🛡️ Hardening
- **Settings Persistence:** Implemented `settings.json` saving and loading for font size and word-wrap.
- **Code Cleanup:** Surgically removed all remaining compiler warnings and dead code.
- **Window Mastery:** Finalized window callbacks (Minimize/Maximize) using Slint native APIs.

### ✅ Milestone Completion
- **Prototype Status:** **STABLE** (High-Fidelity Sovereign Prototype).

---

## [06.14.2026] — Native Shell & Performance Strike (Claude/Jonathon)

### 🏛️ Architecture
- **Native Chrome Restoration:** Reverted to native OS decorations (`no-frame: false`) to ensure 100% stability with Linux window managers (Cinnamon/Muffin).
- **Constraint Resolution:** Swapped hard `width/height` for `preferred-width/height` in Slint, successfully unlocking the **Maximize** button and edge-resizing.
- **Optimized Compositor:** Fixed a channel swap bug (BGR to RGB) and refactored the renderer to a 3-pass pipeline (Selection -> Glyphs -> Cursor) for zero-latency visual layering.
- **Release Zenith:** Shifted to `--release` builds, eliminating all debug-mode lag for a buttery-smooth 60fps experience.

### ✅ Milestone Completion
- **Window Mastery:** Maximize, Resize, and Drag are now 100% stable — **COMPLETE**
- **Sovereign Skia Engine:** Fixed selection/cursor drift via pre-pass metric shaping — **COMPLETE**
