//! Text transformation functions
//!
//! This module contains all the text transformation functions that can be
//! applied to clipboard content.

use regex::Regex;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

/// Remove all extra whitespace and normalize to single spaces
pub fn normalize_whitespace(text: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(text.trim(), " ").to_string()
}

/// Remove all leading/trailing whitespace from each line
pub fn trim_lines(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove empty lines
pub fn remove_empty_lines(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert to lowercase
pub fn to_lowercase(text: &str) -> String {
    text.to_lowercase()
}

/// Convert to UPPERCASE
pub fn to_uppercase(text: &str) -> String {
    text.to_uppercase()
}

/// Convert to Title Case
pub fn to_title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert to Sentence case (first letter of each sentence capitalized)
pub fn to_sentence_case(text: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in text.chars() {
        if capitalize_next && c.is_alphabetic() {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
            if c == '.' || c == '!' || c == '?' {
                capitalize_next = true;
            }
        }
    }

    result
}

/// Remove duplicate lines (preserves order, keeps first occurrence)
pub fn remove_duplicate_lines(text: &str) -> String {
    let mut seen = HashSet::new();
    text.lines()
        .filter(|line| seen.insert(line.to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sort lines alphabetically
pub fn sort_lines(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines.sort();
    lines.join("\n")
}

/// Sort lines alphabetically (reverse)
pub fn sort_lines_reverse(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines.sort();
    lines.reverse();
    lines.join("\n")
}

/// Reverse the order of lines
pub fn reverse_lines(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines.reverse();
    lines.join("\n")
}

/// Convert smart quotes to straight quotes
pub fn fix_smart_quotes(text: &str) -> String {
    // Smart single quotes: U+2018, U+2019
    // Smart double quotes: U+201C, U+201D
    text.replace('\u{2018}', "'")
        .replace('\u{2019}', "'")
        .replace('\u{201C}', "\"")
        .replace('\u{201D}', "\"")
        .replace('\u{2026}', "...")  // Ellipsis
        .replace('\u{2013}', "-")    // En dash
        .replace('\u{2014}', "--")   // Em dash
}

/// Remove all non-ASCII characters
pub fn remove_non_ascii(text: &str) -> String {
    text.chars().filter(|c| c.is_ascii()).collect()
}

/// Normalize Unicode (NFC form)
pub fn normalize_unicode(text: &str) -> String {
    text.nfc().collect()
}

/// Remove emojis and other symbols
pub fn remove_emojis(text: &str) -> String {
    text.chars()
        .filter(|c| {
            let cp = *c as u32;
            // Filter out emoji ranges
            !(0x1F600..=0x1F64F).contains(&cp)  // Emoticons
                && !(0x1F300..=0x1F5FF).contains(&cp)  // Misc Symbols and Pictographs
                && !(0x1F680..=0x1F6FF).contains(&cp)  // Transport and Map
                && !(0x1F1E0..=0x1F1FF).contains(&cp)  // Flags
                && !(0x2600..=0x26FF).contains(&cp)    // Misc symbols
                && !(0x2700..=0x27BF).contains(&cp)    // Dingbats
                && !(0xFE00..=0xFE0F).contains(&cp)    // Variation Selectors
                && !(0x1F900..=0x1F9FF).contains(&cp)  // Supplemental Symbols and Pictographs
                && !(0x1FA00..=0x1FA6F).contains(&cp)  // Chess Symbols
                && !(0x1FA70..=0x1FAFF).contains(&cp)  // Symbols and Pictographs Extended-A
        })
        .collect()
}

/// Strip all formatting (leave only plain text)
/// This is primarily useful for rich text, but for plain text it normalizes whitespace
pub fn strip_formatting(text: &str) -> String {
    // Remove HTML tags if present
    let re = Regex::new(r"<[^>]+>").unwrap();
    let text = re.replace_all(text, "");
    
    // Normalize whitespace
    normalize_whitespace(&text)
}

/// Convert tabs to spaces
pub fn tabs_to_spaces(text: &str, spaces: usize) -> String {
    text.replace('\t', &" ".repeat(spaces))
}

/// Convert spaces to tabs (leading spaces only)
pub fn spaces_to_tabs(text: &str, spaces_per_tab: usize) -> String {
    text.lines()
        .map(|line| {
            let leading_spaces = line.len() - line.trim_start().len();
            let tabs = leading_spaces / spaces_per_tab;
            let remaining = leading_spaces % spaces_per_tab;
            format!(
                "{}{}{}",
                "\t".repeat(tabs),
                " ".repeat(remaining),
                line.trim_start()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove URLs from text
pub fn remove_urls(text: &str) -> String {
    let re = Regex::new(r"https?://\S+").unwrap();
    re.replace_all(text, "").to_string()
}

/// Remove email addresses from text
pub fn remove_emails(text: &str) -> String {
    let re = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    re.replace_all(text, "").to_string()
}

/// Remove phone numbers from text (basic patterns)
pub fn remove_phone_numbers(text: &str) -> String {
    let re = Regex::new(r"(\+\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}").unwrap();
    re.replace_all(text, "").to_string()
}

/// Add line numbers
pub fn add_line_numbers(text: &str) -> String {
    text.lines()
        .enumerate()
        .map(|(i, line)| format!("{:4}: {}", i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove line numbers (assumes format "N: " or "N. ")
pub fn remove_line_numbers(text: &str) -> String {
    let re = Regex::new(r"^\s*\d+[.:]\s*").unwrap();
    text.lines()
        .map(|line| re.replace(line, "").to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove line numbers that are directly stuck to the line content (no separator)
/// Handles formats like "1import React" -> "import React" or "93\t\tconst" -> "\t\tconst"
pub fn remove_line_numbers_stuck(text: &str) -> String {
    let re = Regex::new(r"^(\s*)\d+").unwrap();
    text.lines()
        .map(|line| re.replace(line, "$1").to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert to Unix line endings (LF)
pub fn to_unix_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Convert to Windows line endings (CRLF)
pub fn to_windows_line_endings(text: &str) -> String {
    to_unix_line_endings(text).replace('\n', "\r\n")
}

/// Wrap lines at specified column
pub fn wrap_lines(text: &str, width: usize) -> String {
    text.lines()
        .map(|line| {
            if line.len() <= width {
                line.to_string()
            } else {
                let mut wrapped = String::new();
                let mut current_line = String::new();
                
                for word in line.split_whitespace() {
                    if current_line.is_empty() {
                        current_line = word.to_string();
                    } else if current_line.len() + 1 + word.len() <= width {
                        current_line.push(' ');
                        current_line.push_str(word);
                    } else {
                        if !wrapped.is_empty() {
                            wrapped.push('\n');
                        }
                        wrapped.push_str(&current_line);
                        current_line = word.to_string();
                    }
                }
                
                if !current_line.is_empty() {
                    if !wrapped.is_empty() {
                        wrapped.push('\n');
                    }
                    wrapped.push_str(&current_line);
                }
                
                wrapped
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract only numbers from text
pub fn extract_numbers(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == ' ' || *c == '\n')
        .collect::<String>()
        .split_whitespace()
        .filter(|s| s.parse::<f64>().is_ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Encode HTML entities
pub fn encode_html_entities(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Decode HTML entities
pub fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Slugify text (for URLs)
pub fn slugify(text: &str) -> String {
    let text = text.to_lowercase();
    let re = Regex::new(r"[^a-z0-9\s-]").unwrap();
    let text = re.replace_all(&text, "");
    let re = Regex::new(r"[\s_]+").unwrap();
    re.replace_all(&text, "-").trim_matches('-').to_string()
}

/// camelCase
pub fn to_camel_case(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    
    let mut result = words[0].to_lowercase();
    for word in words.iter().skip(1) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push_str(&first.to_uppercase().to_string());
            result.push_str(&chars.as_str().to_lowercase());
        }
    }
    result
}

/// PascalCase
pub fn to_pascal_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().to_string() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// snake_case
pub fn to_snake_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// SCREAMING_SNAKE_CASE
pub fn to_screaming_snake_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// kebab-case
pub fn to_kebab_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

/// Remove markdown formatting
pub fn remove_markdown(text: &str) -> String {
    // Remove headers
    let re = Regex::new(r"^#{1,6}\s+").unwrap();
    let text = text.lines()
        .map(|line| re.replace(line, "").to_string())
        .collect::<Vec<_>>()
        .join("\n");
    
    // Remove bold/italic
    let re = Regex::new(r"\*\*([^*]+)\*\*").unwrap();
    let text = re.replace_all(&text, "$1");
    let re = Regex::new(r"\*([^*]+)\*").unwrap();
    let text = re.replace_all(&text, "$1");
    let re = Regex::new(r"__([^_]+)__").unwrap();
    let text = re.replace_all(&text, "$1");
    let re = Regex::new(r"_([^_]+)_").unwrap();
    let text = re.replace_all(&text, "$1");
    
    // Remove links but keep text
    let re = Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap();
    let text = re.replace_all(&text, "$1");
    
    // Remove code blocks
    let re = Regex::new(r"`([^`]+)`").unwrap();
    let text = re.replace_all(&text, "$1");
    
    // Remove bullet points
    let re = Regex::new(r"^\s*[-*+]\s+").unwrap();
    text.lines()
        .map(|line| re.replace(line, "").to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Join lines into a single line
pub fn join_lines(text: &str, separator: &str) -> String {
    text.lines().collect::<Vec<_>>().join(separator)
}

/// Split on delimiter and create lines
pub fn split_to_lines(text: &str, delimiter: &str) -> String {
    text.split(delimiter).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
    }

    #[test]
    fn test_remove_duplicate_lines() {
        assert_eq!(remove_duplicate_lines("a\nb\na\nc\nb"), "a\nb\nc");
    }

    #[test]
    fn test_fix_smart_quotes() {
        // Input has smart quotes: U+2018, U+2019, U+201C, U+201D
        let input = "\u{2018}hello\u{2019} \u{201C}world\u{201D}";
        assert_eq!(fix_smart_quotes(input), "'hello' \"world\"");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("hello world"), "Hello World");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World! 2024"), "hello-world-2024");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Hello World"), "hello_world");
    }

    #[test]
    fn test_remove_line_numbers_stuck() {
        let input = "1import React from \"react\";\n2import { useState } from \"react\";\n3\n93\t\tconst foo = 42;\n100\t\treturn foo;";
        let expected = "import React from \"react\";\nimport { useState } from \"react\";\n\n\t\tconst foo = 42;\n\t\treturn foo;";
        assert_eq!(remove_line_numbers_stuck(input), expected);
    }
}
