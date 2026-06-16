mod buffer;

use buffer::piece_table::PieceTable;
use buffer::highlighter::Highlighter;
use buffer::renderer::{ColoredSpan, GlyphRenderer};
use slint::ComponentHandle;
use slint::SharedString;
use std::rc::Rc;
use std::cell::RefCell;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use ignore::WalkBuilder;
use serde::{Serialize, Deserialize};
use rfd::FileDialog;

slint::include_modules!();

const CHAR_HEIGHT_FACTOR: f32 = 1.3;
const CHAR_WIDTH_FACTOR:  f32 = 0.6;
const EDITOR_PAD:         f32 = 10.0;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Settings {
    font_size: i32,
    word_wrap: bool,
    font_family: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14,
            word_wrap: false,
            font_family: "DejaVu Sans Mono".to_string(),
        }
    }
}

impl Settings {
    fn load() -> Self {
        if let Ok(data) = fs::read_to_string("settings.json") {
            if let Ok(settings) = serde_json::from_str(&data) {
                return settings;
            }
        }
        Self::default()
    }

    fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write("settings.json", data);
        }
    }
}

struct Editor {
    buffer:           PieceTable,
    highlighter:      Highlighter,
    cursor_pos:       usize,
    selection_anchor: Option<usize>,
    clipboard:        ClipboardContext,
    cached_text:      String,
    path:             Option<String>,
    name:             String,
    scroll_offset:    usize,
    undo_stack:       Vec<String>,
    redo_stack:       Vec<String>,
}

impl Editor {
    fn new(content: String, path: Option<String>) -> Self {
        let name = path.as_ref()
            .and_then(|p| Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();

        let cached = content.clone();
        let mut highlighter = Highlighter::new();
        highlighter.update(&cached);
        Self {
            buffer:           PieceTable::new(content),
            highlighter,
            cursor_pos:       0,
            selection_anchor: None,
            clipboard:        ClipboardContext::new().unwrap(),
            cached_text:      cached,
            path,
            name,
            scroll_offset:    0,
            undo_stack:       Vec::new(),
            redo_stack:       Vec::new(),
        }
    }

    fn record_history(&mut self) {
        if self.undo_stack.last() != Some(&self.cached_text) {
            self.undo_stack.push(self.cached_text.clone());
            if self.undo_stack.len() > 100 { self.undo_stack.remove(0); }
            self.redo_stack.clear();
        }
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.cached_text.clone());
            self.buffer = PieceTable::new(prev.clone());
            self.cached_text = prev;
            self.cursor_pos = self.cursor_pos.min(self.char_count());
            self.selection_anchor = None;
            self.highlighter.update(&self.cached_text);
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.cached_text.clone());
            self.buffer = PieceTable::new(next.clone());
            self.cached_text = next;
            self.cursor_pos = self.cursor_pos.min(self.char_count());
            self.selection_anchor = None;
            self.highlighter.update(&self.cached_text);
        }
    }

    fn sync_cache(&mut self) {
        self.cached_text = self.buffer.get_text();
        self.highlighter.update(&self.cached_text);
    }

    fn char_count(&self) -> usize { self.cached_text.chars().count() }

    fn insert(&mut self, text: &str) {
        self.record_history();
        self.delete_selection();
        self.buffer.insert(self.cursor_pos, text);
        self.cursor_pos += text.chars().count();
        self.selection_anchor = None;
        self.sync_cache();
    }

    fn backspace(&mut self) {
        self.record_history();
        if self.selection_anchor.is_some() {
            self.delete_selection();
        } else if self.cursor_pos > 0 {
            self.buffer.delete(self.cursor_pos - 1, 1);
            self.cursor_pos -= 1;
            self.sync_cache();
        }
    }

    fn delete_key(&mut self) {
        self.record_history();
        if self.selection_anchor.is_some() {
            self.delete_selection();
        } else if self.cursor_pos < self.char_count() {
            self.buffer.delete(self.cursor_pos, 1);
            self.sync_cache();
        }
    }

    fn delete_selection(&mut self) {
        if let Some(anchor) = self.selection_anchor {
            if anchor == self.cursor_pos { self.selection_anchor = None; return; }
            let (start, end) = sel_range(anchor, self.cursor_pos);
            self.buffer.delete(start, end - start);
            self.cursor_pos = start;
            self.selection_anchor = None;
            self.sync_cache();
        }
    }

    fn select_all(&mut self) {
        self.selection_anchor = Some(0);
        self.cursor_pos = self.char_count();
    }

    fn copy(&mut self) {
        if let Some(anchor) = self.selection_anchor {
            let (start, end) = sel_range(anchor, self.cursor_pos);
            let chars: Vec<char> = self.cached_text.chars().collect();
            if start < end && end <= chars.len() {
                let selected: String = chars[start..end].iter().collect();
                let _ = self.clipboard.set_contents(selected);
            }
        }
    }

    fn cut(&mut self) { self.copy(); self.delete_selection(); }

    fn paste(&mut self) {
        if let Ok(content) = self.clipboard.get_contents() { self.insert(&content); }
    }

    fn move_left(&mut self, shift: bool) {
        self.anchor_if_shift(shift);
        if !shift { self.selection_anchor = None; }
        if self.cursor_pos > 0 { self.cursor_pos -= 1; }
    }

    fn move_right(&mut self, shift: bool) {
        self.anchor_if_shift(shift);
        if !shift { self.selection_anchor = None; }
        if self.cursor_pos < self.char_count() { self.cursor_pos += 1; }
    }

    fn move_up(&mut self, shift: bool) {
        self.anchor_if_shift(shift);
        if !shift { self.selection_anchor = None; }
        let (line_idx, col_idx) = self.cursor_coords(self.cursor_pos);
        if line_idx == 0 { return; }
        let lines = self.lines();
        let prev_len = lines[line_idx - 1].chars().count();
        let target_col = col_idx.min(prev_len);
        self.cursor_pos = self.line_start(line_idx - 1) + target_col;
    }

    fn move_down(&mut self, shift: bool) {
        self.anchor_if_shift(shift);
        if !shift { self.selection_anchor = None; }
        let (line_idx, col_idx) = self.cursor_coords(self.cursor_pos);
        let lines = self.lines();
        if line_idx + 1 >= lines.len() { return; }
        let next_len = lines[line_idx + 1].chars().count();
        let target_col = col_idx.min(next_len);
        self.cursor_pos = self.line_start(line_idx + 1) + target_col;
    }

    fn move_home(&mut self, shift: bool) {
        self.anchor_if_shift(shift);
        if !shift { self.selection_anchor = None; }
        let (line_idx, _) = self.cursor_coords(self.cursor_pos);
        self.cursor_pos = self.line_start(line_idx);
    }

    fn move_end(&mut self, shift: bool) {
        self.anchor_if_shift(shift);
        if !shift { self.selection_anchor = None; }
        let (line_idx, _) = self.cursor_coords(self.cursor_pos);
        let lines = self.lines();
        self.cursor_pos = self.line_start(line_idx) + lines[line_idx].chars().count();
    }

    fn scroll(&mut self, delta_y: f32) {
        let lines_count = self.lines().len();
        if delta_y > 0.0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        } else if delta_y < 0.0 {
            if self.scroll_offset + 1 < lines_count {
                self.scroll_offset += 1;
            }
        }
    }

    fn click(&mut self, x: f32, y: f32, font_size: i32) {
        let char_w = font_size as f32 * CHAR_WIDTH_FACTOR;
        let char_h = font_size as f32 * CHAR_HEIGHT_FACTOR;
        let target_line = ((y - EDITOR_PAD) / char_h).max(0.0) as usize + self.scroll_offset;
        let target_col  = ((x - EDITOR_PAD) / char_w).max(0.0).round() as usize;
        let lines = self.lines();
        if lines.is_empty() { return; }
        let line_idx = target_line.min(lines.len() - 1);
        let col_idx  = target_col.min(lines[line_idx].chars().count());
        self.cursor_pos = self.line_start(line_idx) + col_idx;
        self.selection_anchor = None;
    }

    fn drag(&mut self, x: f32, y: f32, font_size: i32) {
        if self.selection_anchor.is_none() { self.selection_anchor = Some(self.cursor_pos); }
        let char_w = font_size as f32 * CHAR_WIDTH_FACTOR;
        let char_h = font_size as f32 * CHAR_HEIGHT_FACTOR;
        let target_line = ((y - EDITOR_PAD) / char_h).max(0.0) as usize + self.scroll_offset;
        let target_col  = ((x - EDITOR_PAD) / char_w).max(0.0).round() as usize;
        let lines = self.lines();
        if lines.is_empty() { return; }
        let line_idx = target_line.min(lines.len() - 1);
        let col_idx  = target_col.min(lines[line_idx].chars().count());
        self.cursor_pos = self.line_start(line_idx) + col_idx;
    }

    fn anchor_if_shift(&mut self, shift: bool) {
        if shift && self.selection_anchor.is_none() { self.selection_anchor = Some(self.cursor_pos); }
    }

    fn lines(&self) -> Vec<&str> { self.cached_text.split('\n').collect() }

    fn line_start(&self, line_idx: usize) -> usize {
        let lines = self.lines();
        let mut pos = 0;
        for i in 0..line_idx.min(lines.len()) {
            pos += lines[i].chars().count() + 1;
        }
        pos
    }

    fn cursor_coords(&self, pos: usize) -> (usize, usize) {
        let mut line = 0usize;
        let mut col  = 0usize;
        for (i, c) in self.cached_text.chars().enumerate() {
            if i == pos { break; }
            if c == '\n' { line += 1; col = 0; } else { col += 1; }
        }
        (line, col)
    }

    fn get_colored_lines(&self) -> Vec<Vec<ColoredSpan>> {
        let styled = self.highlighter.get_styled_lines(&self.cached_text);
        styled.into_iter().map(|line_segs| {
            line_segs.into_iter().map(|seg| {
                let c = seg.color;
                ColoredSpan { text: seg.text.to_string(), r: c.red(), g: c.green(), b: c.blue() }
            }).collect()
        }).collect()
    }

    fn get_line_numbers(&self) -> Vec<SharedString> {
        let count = self.lines().len().max(1);
        (1..=count).map(|n| n.to_string().into()).collect()
    }

    fn get_status_text(&self) -> String {
        let (line, col) = self.cursor_coords(self.cursor_pos);
        let byte_offset: usize = self.cached_text.chars().take(self.cursor_pos).map(|c| c.len_utf8()).sum();
        let scope = self.highlighter.get_scope_at(byte_offset);
        format!("Ln {}, Col {} | Scope: {}", line + 1, col + 1, scope)
    }

    fn save(&mut self, path: Option<String>) -> anyhow::Result<()> {
        let p = path.or_else(|| self.path.clone());
        if let Some(final_path) = p {
            fs::write(&final_path, &self.cached_text)?;
            self.path = Some(final_path.clone());
            self.name = Path::new(&final_path).file_name().unwrap().to_string_lossy().to_string();
        }
        Ok(())
    }
}

struct ProjectHub {
    editors: Vec<Editor>,
    active_idx: usize,
    search_results: Vec<SearchResult>,
    cursor_visible: bool,
    settings: Settings,
    project_root: String,
}

impl ProjectHub {
    fn new() -> Self {
        let root = fs::canonicalize(".").map(|p| p.to_string_lossy().to_string()).unwrap_or(".".to_string());
        Self {
            editors: vec![Editor::new("Welcome to Glyph Hub.\n\nStart typing here...".to_string(), None)],
            active_idx: 0,
            search_results: Vec::new(),
            cursor_visible: true,
            settings: Settings::load(),
            project_root: root,
        }
    }
    fn active(&mut self) -> &mut Editor { &mut self.editors[self.active_idx] }

    fn detect_root(&mut self, path: &str) {
        let mut current = Path::new(path);
        if current.is_file() {
            current = current.parent().unwrap_or(current);
        }

        let mut found = false;
        let mut check = current;
        while let Some(parent) = check.parent() {
            let markers = [".git", "Cargo.toml", "package.json", ".gemini", "requirements.txt"];
            for marker in &markers {
                if parent.join(marker).exists() {
                    self.project_root = parent.to_string_lossy().to_string();
                    found = true;
                    break;
                }
            }
            if found { break; }
            check = parent;
        }
        if !found {
             self.project_root = current.to_string_lossy().to_string();
        }
    }

    fn new_tab(&mut self) {
        self.editors.push(Editor::new(String::new(), None));
        self.active_idx = self.editors.len() - 1;
    }

    fn open_file(&mut self, path: String) {
        if let Some(idx) = self.editors.iter().position(|e| e.path.as_deref() == Some(&path)) {
            self.active_idx = idx;
            return;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            self.detect_root(&path);
            self.editors.push(Editor::new(content, Some(path)));
            self.active_idx = self.editors.len() - 1;
        }
    }

    fn close_tab(&mut self, idx: usize) {
        if self.editors.len() <= 1 {
            self.editors[0] = Editor::new(String::new(), None);
            self.active_idx = 0;
            return;
        }
        self.editors.remove(idx);
        if self.active_idx >= self.editors.len() {
            self.active_idx = self.editors.len() - 1;
        } else if self.active_idx > idx {
            self.active_idx -= 1;
        }
    }

    fn search(&mut self, query: &str) {
        self.search_results.clear();
        if query.is_empty() { return; }
        let walker = WalkBuilder::new(&self.project_root).build();
        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        for (i, line) in content.split('\n').enumerate() {
                            if line.contains(query) {
                                self.search_results.push(SearchResult {
                                    file_path: entry.path().to_string_lossy().to_string().into(),
                                    line_number: (i + 1) as i32,
                                    text: line.trim().to_string().into(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn sel_range(a: usize, b: usize) -> (usize, usize) { if a < b { (a, b) } else { (b, a) } }

fn main() -> anyhow::Result<()> {
    let ui = AppWindow::new()?;
    let hub = Rc::new(RefCell::new(ProjectHub::new()));
    let canvas_size = Rc::new(RefCell::new((1136u32, 768u32)));
    let renderer = Rc::new(RefCell::new(GlyphRenderer::new()));

    macro_rules! sync_ui {
        ($ui:expr, $hub:expr, $rend:expr, $cs:expr) => {{
            let mut h = $hub.borrow_mut();
            let active_idx = h.active_idx;
            let fs_val = h.settings.font_size;
            let ff_val = h.settings.font_family.clone();
            let ww_val = h.settings.word_wrap;
            let cursor_visible = h.cursor_visible;
            
            let (line_numbers, status_text, cursor_row, cursor_col,
                 selection, colored_lines, cached_text, scroll_offset) = {
                let ed = h.active();
                (
                    ed.get_line_numbers(),
                    ed.get_status_text(),
                    ed.cursor_coords(ed.cursor_pos).0,
                    ed.cursor_coords(ed.cursor_pos).1,
                    ed.selection_anchor.map(|a| (a, ed.cursor_pos)),
                    ed.get_colored_lines(),
                    ed.cached_text.clone(),
                    ed.scroll_offset,
                )
            };

            let tab_entries: Vec<TabEntry> = h.editors.iter().enumerate().map(|(i, e)| {
                TabEntry { name: e.name.clone().into(), active: i == active_idx }
            }).collect();

            let mut files = Vec::new();
            let root = h.project_root.clone();
            let walker = WalkDir::new(&root).max_depth(3).into_iter().filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules" && name != "venv" && name != "bin"
            });
            for entry in walker.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "." { continue; }
                files.push(FileEntry {
                    name: name.into(),
                    is_dir: entry.file_type().is_dir(),
                    path: entry.path().to_string_lossy().to_string().into(),
                });
            }

            let search_results = h.search_results.clone();
            let project_root_shared = h.project_root.clone();
            drop(h);

            let (cw, ch) = *$cs.borrow();
            let render_res = $rend.borrow_mut().render(
                &cached_text,
                fs_val as f32,
                &ff_val,
                cw,
                ch,
                cursor_row,
                cursor_col,
                cursor_visible,
                scroll_offset,
                selection,
                &colored_lines,
            );

            $ui.set_font_size(fs_val);
            $ui.set_font_family(ff_val.into());
            $ui.set_use_word_wrap(ww_val);
            $ui.set_editor_image(render_res.image);
            $ui.set_line_numbers(Rc::new(slint::VecModel::from(line_numbers)).into());
            $ui.set_status_text(status_text.into());
            $ui.set_tabs(Rc::new(slint::VecModel::from(tab_entries)).into());
            $ui.set_active_tab_idx(active_idx as i32);
            $ui.set_file_tree(Rc::new(slint::VecModel::from(files)).into());
            $ui.set_project_root(project_root_shared.into());
            $ui.set_search_results(Rc::new(slint::VecModel::from(search_results)).into());
            $ui.set_scroll_offset(scroll_offset as i32);
        }};
    }

    {
        let cs = canvas_size.clone();
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        ui.on_canvas_resized(move |w, h| {
            *cs.borrow_mut() = (w as u32, h as u32);
            if let Some(ui) = ui_w.upgrade() { sync_ui!(ui, hub_r, rend, cs); }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_key_pressed(move |text, key_name, ctrl, shift| {
            let Some(ui) = ui_w.upgrade() else { return };
            let mut h = hub_r.borrow_mut();
            let ed = h.active();
            if !key_name.is_empty() {
                match key_name.as_str() {
                    "Left" => ed.move_left(shift), "Right" => ed.move_right(shift),
                    "Up" => ed.move_up(shift), "Down" => ed.move_down(shift),
                    "Home" => ed.move_home(shift), "End" => ed.move_end(shift),
                    _ => {}
                }
            } else if ctrl {
                match text.to_lowercase().as_str() {
                    "a" | "\u{01}" => ed.select_all(), "c" | "\u{03}" => ed.copy(),
                    "v" | "\u{16}" => ed.paste(), "x" | "\u{18}" => ed.cut(),
                    "z" | "\u{1a}" => ed.undo(), "y" | "\u{19}" => ed.redo(),
                    _ => {}
                }
            } else {
                match text.as_str() {
                    "\u{08}" => ed.backspace(), "\u{7f}" => ed.delete_key(),
                    "\r" | "\n" => ed.insert("\n"),
                    t if t.chars().all(|c| !c.is_control()) => ed.insert(t),
                    _ => {}
                }
            }
            drop(h);
            sync_ui!(ui, hub_r, rend, cs);
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_mouse_pressed(move |x, y| {
            let Some(ui) = ui_w.upgrade() else { return };
            let fs_val = hub_r.borrow().settings.font_size;
            hub_r.borrow_mut().active().click(x, y, fs_val);
            sync_ui!(ui, hub_r, rend, cs);
        });
        let ui_w2 = ui.as_weak();
        let hub_r2 = hub.clone();
        let rend2 = renderer.clone();
        let cs2 = canvas_size.clone();
        ui.on_mouse_dragged(move |x, y| {
            let Some(ui) = ui_w2.upgrade() else { return };
            let fs_val = hub_r2.borrow().settings.font_size;
            hub_r2.borrow_mut().active().drag(x, y, fs_val);
            sync_ui!(ui, hub_r2, rend2, cs2);
        });
        let ui_w3 = ui.as_weak();
        let hub_r3 = hub.clone();
        let rend3 = renderer.clone();
        let cs3 = canvas_size.clone();
        ui.on_mouse_wheel(move |_dx, dy| {
            let Some(ui) = ui_w3.upgrade() else { return };
            hub_r3.borrow_mut().active().scroll(dy);
            sync_ui!(ui, hub_r3, rend3, cs3);
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_save = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_save_triggered(move || {
            let mut h = hub_save.borrow_mut();
            let current_path = h.active().path.clone();
            if let Some(path) = current_path {
                let _ = h.active().save(Some(path));
            } else if let Some(file_path) = FileDialog::new().save_file() {
                let _ = h.active().save(Some(file_path.to_string_lossy().to_string()));
            }
            drop(h);
            if let Some(ui) = ui_w.upgrade() { sync_ui!(ui, hub_save, rend, cs); }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_open = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_open_triggered(move || {
            if let Some(file_path) = FileDialog::new().pick_file() {
                let path: PathBuf = file_path;
                hub_open.borrow_mut().open_file(path.to_string_lossy().to_string());
                if let Some(ui) = ui_w.upgrade() { sync_ui!(ui, hub_open, rend, cs); }
            }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_tab = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_new_tab_triggered(move || {
            if let Some(ui) = ui_w.upgrade() {
                hub_tab.borrow_mut().new_tab();
                sync_ui!(ui, hub_tab, rend, cs);
            }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_tab = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_tab_selected(move |idx| {
            if let Some(ui) = ui_w.upgrade() {
                hub_tab.borrow_mut().active_idx = idx as usize;
                sync_ui!(ui, hub_tab, rend, cs);
            }
        });

        let ui_close = ui.as_weak();
        let hub_close = hub.clone();
        let rend_close = renderer.clone();
        let cs_close = canvas_size.clone();
        ui.on_tab_closed(move |idx| {
            if let Some(ui) = ui_close.upgrade() {
                hub_close.borrow_mut().close_tab(idx as usize);
                sync_ui!(ui, hub_close, rend_close, cs_close);
            }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_file = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_file_selected(move |path| {
            if let Some(ui) = ui_w.upgrade() {
                hub_file.borrow_mut().open_file(path.to_string());
                sync_ui!(ui, hub_file, rend, cs);
            }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_search_triggered(move |query| {
            if let Some(ui) = ui_w.upgrade() {
                hub_r.borrow_mut().search(&query);
                sync_ui!(ui, hub_r, rend, cs);
            }
        });

        let ui_res = ui.as_weak();
        let hub_res = hub.clone();
        let rend_res = renderer.clone();
        let cs_res = canvas_size.clone();
        ui.on_search_result_selected(move |path, line| {
            if let Some(ui) = ui_res.upgrade() {
                let mut h = hub_res.borrow_mut();
                h.open_file(path.to_string());
                let text = h.active().cached_text.clone();
                let mut pos = 0;
                for (i, line_text) in text.split('\n').enumerate() {
                    if i >= (line - 1) as usize { break; }
                    pos += line_text.chars().count() + 1;
                }
                h.active().cursor_pos = pos;
                drop(h);
                sync_ui!(ui, hub_res, rend_res, cs_res);
            }
        });
    }

    {
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        let timer = slint::Timer::default();
        timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(500), move || {
            if let Some(ui) = ui_w.upgrade() {
                let mut h = hub_r.borrow_mut();
                h.cursor_visible = !h.cursor_visible;
                drop(h);
                sync_ui!(ui, hub_r, rend, cs);
            }
        });
        Box::leak(Box::new(timer));
    }

    {
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        let resize_timer = slint::Timer::default();
        resize_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(150), move || {
            if let Some(ui) = ui_w.upgrade() {
                let w = ui.get_canvas_width() as u32;
                let h = ui.get_canvas_height() as u32;
                let changed = {
                    let current = *cs.borrow();
                    current.0 != w || current.1 != h
                };
                if changed && w > 0 && h > 0 {
                    *cs.borrow_mut() = (w, h);
                    sync_ui!(ui, hub_r, rend, cs);
                }
            }
        });
        Box::leak(Box::new(resize_timer));
    }

    {
        let ui_w = ui.as_weak();
        let hub_r = hub.clone();
        let rend = renderer.clone();
        let cs = canvas_size.clone();
        ui.on_settings_changed(move |font_size, use_word_wrap, font_family| {
            if let Some(ui) = ui_w.upgrade() {
                let mut h = hub_r.borrow_mut();
                h.settings.font_size = font_size;
                h.settings.word_wrap = use_word_wrap;
                h.settings.font_family = font_family.to_string();
                h.settings.save();
                drop(h);
                sync_ui!(ui, hub_r, rend, cs);
            }
        });
    }

    ui.on_menu_action(move |action| {
        match action.as_str() {
            "exit" => std::process::exit(0),
            _ => {}
        }
    });

    sync_ui!(ui, hub, renderer, canvas_size);
    ui.run()?;
    Ok(())
}
