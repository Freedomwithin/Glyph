use tree_sitter::{Parser, Language, Tree, Node};
use slint::Color;

#[derive(Clone, Default, Debug)]
pub struct StyledSegment {
    pub text: slint::SharedString,
    pub color: Color,
}

pub struct Highlighter {
    parser: Parser,
    tree: Option<Tree>,
}

impl Highlighter {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_rust::language();
        parser.set_language(&language).expect("Error loading Rust grammar");
        Self {
            parser,
            tree: None,
        }
    }

    pub fn update(&mut self, text: &str) {
        self.tree = self.parser.parse(text, None);
    }

    pub fn get_styled_lines(&self, text: &str) -> Vec<Vec<StyledSegment>> {
        let mut all_segments = Vec::new();
        if text.is_empty() { return vec![vec![]]; }

        if let Some(tree) = &self.tree {
            let root = tree.root_node();
            self.walk_node(root, text, &mut all_segments);
        } else {
            all_segments.push(StyledSegment {
                text: text.into(),
                color: Color::from_rgb_u8(205, 214, 244),
            });
        }

        let mut lines = Vec::new();
        let mut current_line: Vec<StyledSegment> = Vec::new();

        for seg in all_segments {
            let seg_text = seg.text.as_str();
            let parts: Vec<&str> = seg_text.split('\n').collect();
            
            for (i, part) in parts.iter().enumerate() {
                // ✨ Fix: Empty slices are preserved to prevent layout character deletion
                if let Some(last) = current_line.last_mut() {
                    if last.color == seg.color {
                        let mut new_text = last.text.to_string();
                        new_text.push_str(part);
                        last.text = new_text.into();
                    } else {
                        current_line.push(StyledSegment { text: (*part).into(), color: seg.color });
                    }
                } else {
                    current_line.push(StyledSegment { text: (*part).into(), color: seg.color });
                }

                if i < parts.len() - 1 {
                    lines.push(current_line);
                    current_line = Vec::new();
                }
            }
        }
        lines.push(current_line);
        lines
    }

    fn walk_node(&self, node: Node, text: &str, segments: &mut Vec<StyledSegment>) {
        if node.child_count() == 0 {
            let start = node.start_byte();
            let end = node.end_byte();
            if start < end && end <= text.len() {
                let content = &text[start..end];
                segments.push(StyledSegment {
                    text: content.into(),
                    color: self.get_color_for_kind(node.kind()),
                });
            }
        } else {
            let mut last_end = node.start_byte();
            for i in 0..node.child_count() {
                let child = node.child(i).unwrap();
                if child.start_byte() > last_end {
                    let missing = &text[last_end..child.start_byte()];
                    segments.push(StyledSegment {
                        text: missing.into(),
                        color: Color::from_rgb_u8(205, 214, 244),
                    });
                }
                self.walk_node(child, text, segments);
                last_end = child.end_byte();
            }
            if last_end < node.end_byte() {
                let remaining = &text[last_end..node.end_byte()];
                segments.push(StyledSegment {
                    text: remaining.into(),
                    color: Color::from_rgb_u8(205, 214, 244),
                });
            }
        }
    }

    fn get_color_for_kind(&self, kind: &str) -> Color {
        match kind {
            // Keywords: Indigo/Mauve
            "keyword" | "use" | "mod" | "pub" | "fn" | "let" | "mut" | "match" | "if" | "else" | "return" | 
            "struct" | "enum" | "impl" | "type" | "trait" | "where" | "as" | "const" | "static" | 
            "async" | "await" | "for" | "in" | "while" | "loop" | "break" | "continue" | 
            "crate" | "super" | "self" | "Self" | "dyn" | "move" | "unsafe" | "extern" => {
                Color::from_rgb_u8(203, 166, 247)
            }
            // Strings: Green (Now properly matches token values)
            "string_literal" | "string" | "char_literal" | "raw_string_literal" => {
                Color::from_rgb_u8(166, 227, 161)
            }
            // Comments: Muted Gray
            "comment" | "line_comment" | "block_comment" => {
                Color::from_rgb_u8(108, 112, 134)
            }
            // Functions & Identifiers: Electric Blue
            "function_item" | "call_expression" | "function" | "identifier" | "field_identifier" => {
                Color::from_rgb_u8(137, 180, 250)
            }
            // Numbers: Peach
            "number_literal" | "integer_literal" | "float_literal" => {
                Color::from_rgb_u8(250, 179, 135)
            }
            // Types: Yellow
            "type_identifier" | "primitive_type" => {
                Color::from_rgb_u8(249, 226, 175)
            }
            // Symbols/Punctuation: Flamingo
            "{" | "}" | "[" | "]" | "(" | ")" | "=>" | "->" | "::" | ":" | ";" | "," | "." | "!" => {
                Color::from_rgb_u8(245, 194, 231)
            }
            _ => Color::from_rgb_u8(205, 214, 244), // Default Foreground (Slate)
        }
    }

    pub fn get_scope_at(&self, pos: usize) -> String {
        if let Some(tree) = &self.tree {
            let root = tree.root_node();
            // ✨ Fix: Safe multi-byte calculation boundary lookup checks
            if pos <= root.end_byte() {
                if let Some(node) = root.descendant_for_byte_range(pos, pos) {
                    return node.kind().to_string();
                }
            }
        }
        "none".to_string()
    }
}
