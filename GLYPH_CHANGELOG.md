# Glyph Changelog

## [0.3.0] – 2026-07-09

### Added
- **Sidebar file‑tree navigation** – clicking folders now changes the project root, allowing you to browse subdirectories directly from the sidebar.
- **Persistent sidebar state** – the sidebar collapse state (`sidebar_collapsed`) and its width (`sidebar_width`) are saved to `settings.json` and restored on launch.
- **Persistent line‑numbers toggle** – the `show_line_numbers` setting is now stored in `settings.json`.
- **Larger default sidebar width** – now 440px for better readability of file names.
- **Bigger, more responsive sidebar toggle button** – 36×36 with a full‑size `TouchArea` for easier clicking.

### Changed
- **Improved file‑tree layout** – entries are taller (44px) with larger fonts (17px) for better visibility.
- **Reduced UI lag** – optimised the `sync_ui!` macro to release borrows earlier, preventing `RefCell` contention.
- **Cleaned up duplicate UI buttons** – removed stray “Settings” gear icons from the File/Edit menus and the settings dialog.

### Fixed
- **Sidebar collapse now works reliably** – the Rust callback is correctly registered and the borrow is properly released before UI sync.
- **Syntax errors in `appwindow.slint`** – fixed invalid `padding` shorthand and removed unsupported `horizontal-alignment` on `Button`.
- **Settings callback signature mismatch** – aligned the Rust closure with the Slint callback (6 parameters, correct order).

---

## [0.2.0] – 2026-07-06
### Added
- Initial Pro feature: session restore (saves open tabs and active index).
- Theme support: Sovereign Slate (default), Sovereign Night, Temple (Pro).
- License activation via `GLYPH-PRO-XXXX-XXXX` keys.
- Project‑wide search with clickable results.

### Changed
- Upgraded to Slint 1.5 with `backend-winit` and `renderer-winit-software`.
- Switched to `tree-sitter-rust` for syntax highlighting.

### Fixed
- Unicode safety in PieceTable (multi‑byte char handling).
- Cursor positioning on multi‑byte characters.
- Undo/redo stack corruption after large edits.