// search.rs - Find and Find & Replace for the writing area
//
// This module is deliberately egui-light: `SearchState` only deals with
// character-index ranges into the document `String`, so it can be unit
// tested and reasoned about without a UI. The one exception is
// `build_layout_job`, which needs egui's text types to paint the
// highlight decorations directly into the TextEdit's layouter.

use eframe::egui;

/// Find / find-and-replace state for the currently active document.
pub struct SearchState {
    pub open: bool,
    pub replace_mode: bool,
    pub query: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,

    /// Character-index ranges (start, end) of every match, in document order.
    pub matches: Vec<(usize, usize)>,
    /// Index into `matches` of the currently selected/active match.
    pub current: Option<usize>,

    /// Set by the UI layer once a char index should be scrolled into view.
    pub pending_scroll: Option<usize>,
    /// Ask the UI to grab keyboard focus for the query field this frame.
    pub focus_query: bool,
    /// Small transient status text (e.g. "Replaced 4 occurrences").
    pub status: Option<String>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            open: false,
            replace_mode: false,
            query: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            matches: Vec::new(),
            current: None,
            pending_scroll: None,
            focus_query: false,
            status: None,
        }
    }

    pub fn open_find(&mut self) {
        self.open = true;
        self.replace_mode = false;
        self.focus_query = true;
    }

    pub fn open_replace(&mut self) {
        self.open = true;
        self.replace_mode = true;
        self.focus_query = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.matches.clear();
        self.current = None;
        self.status = None;
    }

    /// Recompute all match positions for the current query against `text`.
    pub fn refresh_matches(&mut self, text: &str) {
        self.matches = find_matches(text, &self.query, self.case_sensitive, self.whole_word);
        self.status = None;
        self.current = match self.current {
            Some(cur) if cur < self.matches.len() => Some(cur),
            _ if !self.matches.is_empty() => Some(0),
            _ => None,
        };
    }

    pub fn current_match(&self) -> Option<(usize, usize)> {
        self.current.and_then(|i| self.matches.get(i)).copied()
    }

    /// Move to the first match starting at or after `from_char`, wrapping around.
    pub fn find_next(&mut self, text: &str, from_char: usize) -> Option<(usize, usize)> {
        self.refresh_matches(text);
        if self.matches.is_empty() {
            self.status = Some("No matches".to_string());
            return None;
        }
        let idx = self
            .matches
            .iter()
            .position(|&(s, _)| s >= from_char)
            .unwrap_or(0);
        self.current = Some(idx);
        Some(self.matches[idx])
    }

    /// Move to the last match starting strictly before `from_char`, wrapping around.
    pub fn find_prev(&mut self, text: &str, from_char: usize) -> Option<(usize, usize)> {
        self.refresh_matches(text);
        if self.matches.is_empty() {
            self.status = Some("No matches".to_string());
            return None;
        }
        let idx = self
            .matches
            .iter()
            .rposition(|&(s, _)| s < from_char)
            .unwrap_or(self.matches.len() - 1);
        self.current = Some(idx);
        Some(self.matches[idx])
    }

    /// Replace the current match with `replace_text`. Returns the char index
    /// right after the inserted replacement, if there was a current match.
    pub fn replace_current(&mut self, text: &mut String) -> Option<usize> {
        let (start, end) = self.current_match()?;
        replace_char_range(text, start, end, &self.replace_text);
        let new_pos = start + self.replace_text.chars().count();
        self.refresh_matches(text);
        self.current = self
            .matches
            .iter()
            .position(|&(s, _)| s >= new_pos)
            .or(if self.matches.is_empty() { None } else { Some(0) });
        Some(new_pos)
    }

    /// Replace every match. Returns the number of replacements made.
    pub fn replace_all(&mut self, text: &mut String) -> usize {
        self.refresh_matches(text);
        let count = self.matches.len();
        if count == 0 {
            self.status = Some("No matches".to_string());
            return 0;
        }
        // Replace back-to-front so earlier char indices stay valid.
        for &(start, end) in self.matches.clone().iter().rev() {
            replace_char_range(text, start, end, &self.replace_text);
        }
        self.refresh_matches(text);
        self.status = Some(format!(
            "Replaced {} occurrence{}",
            count,
            if count == 1 { "" } else { "s" }
        ));
        count
    }
}

fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

fn replace_char_range(text: &mut String, start_char: usize, end_char: usize, replacement: &str) {
    let start_b = char_to_byte(text, start_char);
    let end_b = char_to_byte(text, end_char);
    text.replace_range(start_b..end_b, replacement);
}

/// Find all non-overlapping occurrences of `query` in `text`, returning
/// character-index ranges `[start, end)`.
pub fn find_matches(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    if query.is_empty() {
        return result;
    }

    let fold = |c: char| -> char {
        if case_sensitive {
            c
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };

    let hay: Vec<char> = text.chars().collect();
    let hay_f: Vec<char> = hay.iter().copied().map(fold).collect();
    let needle_f: Vec<char> = query.chars().map(fold).collect();

    if needle_f.is_empty() || needle_f.len() > hay_f.len() {
        return result;
    }

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    let mut start = 0usize;
    let last_start = hay_f.len() - needle_f.len();
    while start <= last_start {
        if hay_f[start..start + needle_f.len()] == needle_f[..] {
            let end = start + needle_f.len();
            let ok = !whole_word || {
                let before_ok = start == 0 || !is_word_char(hay[start - 1]);
                let after_ok = end == hay.len() || !is_word_char(hay[end]);
                before_ok && after_ok
            };
            if ok {
                result.push((start, end));
                start = end;
                continue;
            }
        }
        start += 1;
    }

    result
}

/// Build a `LayoutJob` for the writing area's TextEdit that paints a
/// background highlight behind every match, and a stronger highlight
/// behind the current one. Falls back to a plain job when there are no
/// matches, matching egui's own default layouter exactly.
pub fn build_layout_job(
    text: &str,
    font_id: egui::FontId,
    text_color: egui::Color32,
    wrap_width: f32,
    matches: &[(usize, usize)],
    current: Option<usize>,
) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};

    if matches.is_empty() {
        return LayoutJob::simple(text.to_owned(), font_id, text_color, wrap_width);
    }

    // Map the char-index match boundaries to byte offsets in one linear pass.
    let mut byte_at_char = Vec::with_capacity(hint_char_count(text) + 1);
    for (b, _) in text.char_indices() {
        byte_at_char.push(b);
    }
    byte_at_char.push(text.len());

    const MATCH_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(120, 100, 20, 110);
    const CURRENT_BG: egui::Color32 = egui::Color32::from_rgb(255, 140, 0);
    const CURRENT_FG: egui::Color32 = egui::Color32::BLACK;

    let mut job = LayoutJob {
        break_on_newline: true,
        ..Default::default()
    };
    job.wrap.max_width = wrap_width;

    let mut cursor_byte = 0usize;
    for (i, &(s, e)) in matches.iter().enumerate() {
        let sb = byte_at_char[s];
        let eb = byte_at_char[e];
        if sb > cursor_byte {
            job.append(
                &text[cursor_byte..sb],
                0.0,
                TextFormat::simple(font_id.clone(), text_color),
            );
        }
        let is_current = Some(i) == current;
        job.append(
            &text[sb..eb],
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: if is_current { CURRENT_FG } else { text_color },
                background: if is_current { CURRENT_BG } else { MATCH_BG },
                ..Default::default()
            },
        );
        cursor_byte = eb;
    }
    if cursor_byte < text.len() {
        job.append(
            &text[cursor_byte..],
            0.0,
            TextFormat::simple(font_id, text_color),
        );
    }

    job
}

fn hint_char_count(text: &str) -> usize {
    // Cheap upper-bound guess to size the byte_at_char Vec without a second pass.
    text.len()
}
