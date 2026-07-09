use cosmic_text::{
    Attrs, Buffer as CosmicBuffer, Color as CosmicColor, Family, FontSystem, Metrics,
    Shaping, SwashCache,
};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

#[derive(Clone)]
pub struct ColoredSpan {
    pub text: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone)]
pub struct HighlightRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct RenderResult {
    pub image: slint::Image,
    pub _cursor_x: f32,
    pub _cursor_y: f32,
}

pub struct GlyphRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    pub last_lines_x: Vec<Vec<f32>>,
    pub last_scroll_offset: usize,
}

// Shaped metrics for a single visible line.
struct LineMetrics {
    col_x: Vec<f32>,
    cosmic_buf: CosmicBuffer,
}

fn clamped_rect(x: f32, y: f32, w: f32, h: f32, max_w: f32, max_h: f32) -> Option<Rect> {
    let x0 = x.max(0.0);
    let y0 = y.max(0.0);
    let x1 = (x + w).min(max_w);
    let y1 = (y + h).min(max_h);
    let cw = x1 - x0;
    let ch = y1 - y0;
    if cw <= 0.0 || ch <= 0.0 { return None; }
    Rect::from_xywh(x0, y0, cw, ch)
}

impl GlyphRenderer {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            last_lines_x: Vec::new(),
            last_scroll_offset: 0,
        }
    }

    fn shape_line(
        &mut self,
        line_text: &str,
        line_idx: usize,
        w: u32,
        pad: f32,
        metrics: Metrics,
        base_attrs: Attrs,
        line_colors: &[Vec<ColoredSpan>],
    ) -> LineMetrics {
        let char_count = line_text.chars().count();
        let mut cbuf = CosmicBuffer::new(&mut self.font_system, metrics);
        cbuf.set_size(&mut self.font_system, Some(w as f32 - pad * 2.0), None);

        match line_colors.get(line_idx) {
            Some(spans) if !spans.is_empty() => {
                let span_pairs: Vec<(String, Attrs)> = spans
                    .iter()
                    .map(|s| (s.text.clone(), base_attrs.color(CosmicColor::rgb(s.r, s.g, s.b))))
                    .collect();
                cbuf.set_rich_text(&mut self.font_system, span_pairs.iter().map(|(t, a)| (t.as_str(), *a)), base_attrs, Shaping::Advanced);
            }
            _ => {
                let display = if line_text.is_empty() { " " } else { line_text };
                cbuf.set_text(&mut self.font_system, display, base_attrs.color(CosmicColor::rgb(205, 214, 244)), Shaping::Advanced);
            }
        }

        cbuf.shape_until_scroll(&mut self.font_system, false);

        let mut col_x = vec![pad; char_count + 1];
        let byte_to_col: Vec<usize> = {
            let mut map = vec![0usize; line_text.len() + 1];
            let mut col = 0usize;
            for (byte_idx, _ch) in line_text.char_indices() {
                map[byte_idx] = col;
                col += 1;
            }
            map[line_text.len()] = col;
            map
        };

        for run in cbuf.layout_runs() {
            for glyph in run.glyphs.iter() {
                let start_col = if glyph.start < byte_to_col.len() { byte_to_col[glyph.start] } else { char_count };
                let end_col = if glyph.end <= byte_to_col.len() { byte_to_col[glyph.end.min(byte_to_col.len() - 1)] } else { char_count };
                if start_col <= char_count { col_x[start_col] = pad + glyph.x; }
                if end_col >= char_count && char_count < col_x.len() { col_x[char_count] = pad + glyph.x + glyph.w; }
            }
        }

        LineMetrics { col_x, cosmic_buf: cbuf }
    }

    pub fn render(
        &mut self,
        text: &str,
        font_size: f32,
        font_family: &str,
        canvas_w: u32,
        canvas_h: u32,
        cursor_row: usize,
        cursor_col: usize,
        cursor_visible: bool,
        scroll_offset: usize,
        selection: Option<(usize, usize)>,
        line_colors: &[Vec<ColoredSpan>],
    ) -> RenderResult {
        let pad    = 10.0f32;
        let char_h = font_size * 1.3;
        let w = canvas_w.max(8);
        let h = canvas_h.max(8);

        self.last_scroll_offset = scroll_offset;
        self.last_lines_x.clear();

        let mut pixmap = match Pixmap::new(w, h) {
            Some(p) => p,
            None => return RenderResult { image: slint::Image::default(), _cursor_x: 0.0, _cursor_y: 0.0 },
        };

        pixmap.fill(Color::from_rgba8(15, 23, 42, 255));
        let (sel_start, sel_end) = selection.map(|(s, e)| if s < e { (s, e) } else { (e, s) }).unwrap_or((0, 0));
        let has_sel = selection.is_some() && sel_start != sel_end;
        let text_lines: Vec<&str> = text.split('\n').collect();

        let metrics    = Metrics::new(font_size, char_h);
        let base_attrs = Attrs::new().family(Family::Name(font_family));

        let visible_line_count = ((h as f32 - pad) / char_h).ceil() as usize + 1;
        let mut shaped_lines: Vec<LineMetrics> = Vec::with_capacity(visible_line_count);

        for i in 0..visible_line_count {
            let line_idx = i + scroll_offset;
            if line_idx >= text_lines.len() { break; }
            let y_top = pad + i as f32 * char_h;
            if y_top > h as f32 { break; }
            let lm = self.shape_line(text_lines[line_idx], line_idx, w, pad, metrics, base_attrs, line_colors);
            self.last_lines_x.push(lm.col_x.clone());
            shaped_lines.push(lm);
        }

        // ── Pass 1: selection backgrounds ────────────────────────────────
        if has_sel {
            let mut sel_paint = Paint::default();
            sel_paint.set_color(Color::from_rgba8(30, 74, 110, 200));
            sel_paint.anti_alias = false;

            let mut global_char_idx = 0usize;
            for i in 0..scroll_offset {
                if let Some(line) = text_lines.get(i) { global_char_idx += line.chars().count() + 1; }
            }

            for (i, lm) in shaped_lines.iter().enumerate() {
                let line_len  = lm.col_x.len().saturating_sub(1);
                let line_end  = global_char_idx + line_len;
                let y_top     = pad + i as f32 * char_h;

                if sel_end > global_char_idx && sel_start <= line_end {
                    let start_col = if sel_start > global_char_idx { sel_start - global_char_idx } else { 0 };
                    let end_col = (if sel_end <= line_end { sel_end - global_char_idx } else { line_len }).min(line_len);
                    let rx = lm.col_x.get(start_col).copied().unwrap_or(pad);
                    let rx_end = lm.col_x.get(end_col).copied().unwrap_or(rx + 1.0);
                    let rw = (rx_end - rx).max(1.0);
                    if let Some(rect) = clamped_rect(rx, y_top, rw, char_h, w as f32, h as f32) {
                        pixmap.fill_rect(rect, &sel_paint, Transform::identity(), None);
                    }
                }
                global_char_idx += line_len + 1;
            }
        }

        // ── Pass 2: glyph rendering ──────────────────────────────────────
        let mut actual_cursor_x = pad;
        let mut actual_cursor_y = -1000.0;

        for (i, lm) in shaped_lines.iter().enumerate() {
            let line_idx  = i + scroll_offset;
            let y_top     = pad + i as f32 * char_h;

            if line_idx == cursor_row {
                actual_cursor_y = y_top;
                let col = cursor_col.min(lm.col_x.len().saturating_sub(1));
                actual_cursor_x = lm.col_x[col];
            }

            for run in lm.cosmic_buf.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let physical = glyph.physical((pad, y_top + run.line_y), 1.0);
                    let glyph_color = glyph.color_opt.unwrap_or(CosmicColor::rgb(205, 214, 244));
                    self.swash_cache.with_pixels(&mut self.font_system, physical.cache_key, glyph_color, |x, y, color| {
                        let px = physical.x + x;
                        let py = physical.y + y;
                        if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
                            let idx = (py as u32 * w + px as u32) as usize;
                            if let Some(dst) = pixmap.pixels_mut().get_mut(idx) {
                                let src_a = color.a() as f32 / 255.0;
                                let da    = 1.0 - src_a;
                                let r = ((color.r() as f32 * src_a) + (dst.red()   as f32 * da)) as u8;
                                let g = ((color.g() as f32 * src_a) + (dst.green() as f32 * da)) as u8;
                                let b = ((color.b() as f32 * src_a) + (dst.blue()  as f32 * da)) as u8;
                                if let Some(p) = tiny_skia::PremultipliedColorU8::from_rgba(r, g, b, 255) { *dst = p; }
                            }
                        }
                    });
                }
            }
        }

        // ── Pass 3: cursor ───────────────────────────────────────────────
        let cursor_on_screen = actual_cursor_y >= 0.0 && actual_cursor_y < h as f32;
        if cursor_visible && cursor_on_screen {
            let mut cur_paint = Paint::default();
            cur_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
            cur_paint.anti_alias = false;
            if let Some(rect) = clamped_rect(actual_cursor_x, actual_cursor_y, 2.0, char_h, w as f32, h as f32) {
                pixmap.fill_rect(rect, &cur_paint, Transform::identity(), None);
            }
        }

        let slint_buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(pixmap.data(), w, h);
        RenderResult { image: slint::Image::from_rgba8(slint_buf), _cursor_x: actual_cursor_x, _cursor_y: actual_cursor_y }
    }

    pub fn get_position_at(&self, click_x: f32, click_y: f32, font_size: f32, text: &str) -> usize {
        let pad = 10.0f32;
        let char_h = font_size * 1.3;
        
        let relative_y = click_y - pad;
        let line_offset = (relative_y / char_h).max(0.0) as usize;
        let line_idx = line_offset + self.last_scroll_offset;
        
        let text_lines: Vec<&str> = text.split('\n').collect();
        if text_lines.is_empty() { return 0; }
        
        let target_line = line_idx.min(text_lines.len() - 1);
        
        let mut char_pos = 0;
        for i in 0..target_line {
            char_pos += text_lines[i].chars().count() + 1;
        }
        
        if let Some(col_x) = self.last_lines_x.get(target_line.saturating_sub(self.last_scroll_offset)) {
            if col_x.is_empty() { return char_pos; }
            
            let mut best_col = 0;
            let mut min_diff = f32::MAX;
            for (col, &x_pos) in col_x.iter().enumerate() {
                let diff = (x_pos - click_x).abs();
                if diff < min_diff {
                    min_diff = diff;
                    best_col = col;
                }
            }
            char_pos += best_col;
        } else {
            let char_w = font_size * 0.6;
            let relative_x = click_x - pad;
            let col = (relative_x / char_w).max(0.0).round() as usize;
            let line_len = text_lines[target_line].chars().count();
            char_pos += col.min(line_len);
        }
        char_pos
    }
}
