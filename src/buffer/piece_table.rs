#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Original,
    Add,
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub source: Source,
    pub offset: usize, // Character index in original or add buffer
    pub length: usize, // Number of characters
}

pub struct PieceTable {
    original: Vec<char>,
    add: Vec<char>,
    pieces: Vec<Piece>,
}

impl PieceTable {
    pub fn new(content: String) -> Self {
        let chars: Vec<char> = content.chars().collect();
        let length = chars.len();
        Self {
            original: chars,
            add: Vec::new(),
            pieces: vec![Piece {
                source: Source::Original,
                offset: 0,
                length,
            }],
        }
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        let add_offset = self.add.len();
        let char_len = chars.len();
        self.add.extend(chars);
        
        let new_piece = Piece {
            source: Source::Add,
            offset: add_offset,
            length: char_len,
        };

        if self.pieces.is_empty() {
            self.pieces.push(new_piece);
            return;
        }

        self.split_at(char_idx);
        
        let mut current_chars = 0;
        let mut target_idx = self.pieces.len();
        for (i, piece) in self.pieces.iter().enumerate() {
            if current_chars == char_idx {
                target_idx = i;
                break;
            }
            current_chars += piece.length;
        }
        self.pieces.insert(target_idx, new_piece);
    }

    pub fn delete(&mut self, char_idx: usize, length: usize) {
        if length == 0 || self.pieces.is_empty() {
            return;
        }

        self.split_at(char_idx);
        self.split_at(char_idx + length);

        let mut current_chars = 0;
        let mut start_idx = None;
        let mut end_idx = None;

        for (i, piece) in self.pieces.iter().enumerate() {
            if current_chars == char_idx {
                start_idx = Some(i);
            }
            if current_chars == char_idx + length {
                end_idx = Some(i);
                break;
            }
            current_chars += piece.length;
        }
        
        let end = end_idx.unwrap_or(self.pieces.len());
        if let Some(start) = start_idx {
            self.pieces.drain(start..end);
        }
    }

    fn split_at(&mut self, char_idx: usize) {
        let mut current_chars = 0;
        let mut piece_idx = None;

        for (i, piece) in self.pieces.iter().enumerate() {
            if current_chars < char_idx && char_idx < current_chars + piece.length {
                piece_idx = Some(i);
                break;
            }
            current_chars += piece.length;
        }

        if let Some(idx) = piece_idx {
            let piece = self.pieces[idx];
            let split_point = char_idx - current_chars;

            let left = Piece {
                source: piece.source,
                offset: piece.offset,
                length: split_point,
            };
            let right = Piece {
                source: piece.source,
                offset: piece.offset + split_point,
                length: piece.length - split_point,
            };

            self.pieces[idx] = left;
            self.pieces.insert(idx + 1, right);
        }
    }

    pub fn get_text(&self) -> String {
        let mut result = String::new();
        for piece in &self.pieces {
            let buf = match piece.source {
                Source::Original => &self.original,
                Source::Add      => &self.add,
            };
            result.extend(&buf[piece.offset..piece.offset + piece.length]);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_unicode() {
        let mut pt = PieceTable::new("hello".to_string());
        pt.insert(5, " 🌍");
        assert_eq!(pt.get_text(), "hello 🌍");
    }

    #[test]
    fn test_delete_unicode() {
        let mut pt = PieceTable::new("hello 🌍".to_string());
        pt.delete(6, 1); // delete 🌍
        assert_eq!(pt.get_text(), "hello ");
    }
}
