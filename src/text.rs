//! Conservative text-width estimate and word wrapping.
use unicode_width::UnicodeWidthChar;

/// Wide/fullwidth characters count 1 em; everything else .55 em. Final metrics are checked in a browser.
pub fn text_width(value: &str, size: f64) -> f64 {
    value
        .chars()
        .fold(0.0, |sum, c| sum + if c.width() == Some(2) { 1.0 } else { 0.55 })
        * size
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')
}

/// Tokens of `[A-Za-z0-9_./-]+ | \s+ | any other single char`.
fn tokens(paragraph: &str) -> Vec<String> {
    let chars: Vec<char> = paragraph.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut j = i + 1;
        if is_word(c) {
            while j < chars.len() && is_word(chars[j]) {
                j += 1;
            }
        } else if c.is_whitespace() {
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
        }
        out.push(chars[i..j].iter().collect());
        i = j;
    }
    out
}

/// Keep Latin words intact; explicit newlines remain authoritative.
pub fn wrap_text(value: &str, width: f64, size: f64) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in value.split('\n') {
        let mut current = String::new();
        for word in tokens(paragraph) {
            let is_space = word.chars().all(char::is_whitespace);
            if is_space && current.is_empty() {
                continue;
            }
            if text_width(&word, size) > width {
                if !current.trim().is_empty() {
                    lines.push(current.trim_end().to_string());
                    current.clear();
                }
                let chunks = ((text_width(&word, size) / width).ceil() as usize).max(2);
                let chars: Vec<char> = word.chars().collect();
                let length = chars.len().div_ceil(chunks).max(1);
                let mut parts: Vec<String> = chars.chunks(length).map(|p| p.iter().collect()).collect();
                let last = parts.pop().unwrap_or_default();
                lines.extend(parts);
                current = last;
            } else if !current.is_empty() && text_width(&format!("{current}{word}"), size) > width {
                lines.push(current.trim_end().to_string());
                current = word.trim_start().to_string();
            } else {
                current.push_str(&word);
            }
        }
        lines.push(current.trim_end().to_string());
    }
    lines
}
