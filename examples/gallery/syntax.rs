//! A minimal Rust tokenizer for gallery code snippets.
//!
//! GPUI ships no syntax engine, so snippets are highlighted with a small
//! dependency-free lexer and `StyledText` highlights. It recognizes the Rust
//! constructs the gallery snippets use; anything unknown stays plain.

use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Token {
    Plain,
    Comment,
    String,
    Number,
    Keyword,
    Type,
    Function,
    Macro,
    Lifetime,
    Attribute,
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "box", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "union", "unsafe", "use", "where", "while",
];

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn ident(text: &str, start: usize) -> &str {
    let bytes = text.as_bytes();
    let mut end = start;
    while end < bytes.len() && is_ident(bytes[end]) {
        end += 1;
    }
    &text[start..end]
}

/// Lexes `source` into non-overlapping plain/token spans covering the whole
/// text, in order.
pub(crate) fn tokenize(source: &str) -> Vec<(Range<usize>, Token)> {
    let bytes = source.as_bytes();
    let mut spans: Vec<(Range<usize>, Token)> = Vec::new();
    let mut plain_start = 0usize;
    let mut i = 0usize;

    fn flush(spans: &mut Vec<(Range<usize>, Token)>, plain_start: &mut usize, end: usize) {
        if *plain_start < end {
            spans.push((*plain_start..end, Token::Plain));
        }
        *plain_start = end;
    }

    while i < bytes.len() {
        let byte = bytes[i];
        // Comments.
        if byte == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                let end = source[i..].find('\n').map_or(bytes.len(), |n| i + n);
                flush(&mut spans, &mut plain_start, i);
                spans.push((i..end, Token::Comment));
                i = end;
                plain_start = end;
                continue;
            }
            if bytes[i + 1] == b'*' {
                let mut depth = 1;
                let mut j = i + 2;
                while j + 1 < bytes.len() && depth > 0 {
                    match (bytes[j], bytes[j + 1]) {
                        (b'/', b'*') => {
                            depth += 1;
                            j += 2;
                        }
                        (b'*', b'/') => {
                            depth -= 1;
                            j += 2;
                        }
                        _ => j += 1,
                    }
                }
                let end = if depth == 0 { j } else { bytes.len() };
                flush(&mut spans, &mut plain_start, i);
                spans.push((i..end, Token::Comment));
                i = end;
                plain_start = end;
                continue;
            }
        }
        // Attributes: `#[...]` up to the matching bracket on one line.
        if byte == b'#' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            let end = source[i..].find(']').map_or(bytes.len(), |n| i + n + 1);
            flush(&mut spans, &mut plain_start, i);
            spans.push((i..end, Token::Attribute));
            i = end;
            plain_start = end;
            continue;
        }
        // Strings, raw strings and char/lifetime literals.
        if byte == b'"' || byte == b'r' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            // Start past the opening quote (and the `r` prefix).
            let mut j = if byte == b'r' { i + 2 } else { i + 1 };
            let mut closed = false;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => j += 2,
                    b'"' => {
                        j += 1;
                        closed = true;
                        break;
                    }
                    _ => j += 1,
                }
            }
            let end = if closed { j } else { bytes.len() };
            flush(&mut spans, &mut plain_start, i);
            spans.push((i..end, Token::String));
            i = end;
            plain_start = end;
            continue;
        }
        if byte == b'\'' {
            // Lifetime: `'` followed by an identifier and not a char close.
            let rest = &source[i + 1..];
            let inner = ident(rest, 0);
            let is_lifetime = !inner.is_empty()
                && i + 1 + inner.len() < bytes.len()
                && bytes[i + 1 + inner.len()] != b'\'';
            if is_lifetime {
                flush(&mut spans, &mut plain_start, i);
                let end = i + 1 + inner.len();
                spans.push((i..end, Token::Lifetime));
                i = end;
                plain_start = end;
                continue;
            }
            // Char literal with escapes.
            let mut j = i + 1;
            let mut closed = false;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => j += 2,
                    b'\'' => {
                        j += 1;
                        closed = true;
                        break;
                    }
                    _ => j += 1,
                }
            }
            let end = if closed { j } else { bytes.len() };
            flush(&mut spans, &mut plain_start, i);
            spans.push((i..end, Token::String));
            i = end;
            plain_start = end;
            continue;
        }
        // Numbers, including bases, suffixes and separators.
        if byte.is_ascii_digit() {
            let mut j = i;
            while j < bytes.len() {
                let b = bytes[j];
                if is_ident(b) || b == b'.' || b == b'-' && j > 0 && bytes[j - 1] == b'e' {
                    j += 1;
                } else {
                    break;
                }
            }
            flush(&mut spans, &mut plain_start, i);
            spans.push((i..j, Token::Number));
            i = j;
            plain_start = j;
            continue;
        }
        // Identifiers, keywords, types, macros and function calls.
        if is_ident(byte) {
            let word = ident(source, i);
            let word_end = i + word.len();
            let after = bytes.get(word_end).copied().unwrap_or(0);
            let (end, kind) = if after == b'!' {
                (word_end + 1, Token::Macro)
            } else if KEYWORDS.contains(&word) {
                (word_end, Token::Keyword)
            } else if word.bytes().next().is_some_and(|b| b.is_ascii_uppercase()) {
                (word_end, Token::Type)
            } else if after == b'(' {
                (word_end, Token::Function)
            } else {
                (word_end, Token::Plain)
            };
            if kind != Token::Plain {
                flush(&mut spans, &mut plain_start, i);
                spans.push((i..end, kind));
                plain_start = end;
            }
            i = end;
            continue;
        }
        i += 1;
    }
    flush(&mut spans, &mut plain_start, bytes.len());
    spans
}

/// The theme-aware foreground color for a token. `Plain` returns `None` so the
/// surrounding text color applies.
pub(crate) fn color(token: Token, dark: bool) -> Option<u32> {
    match (token, dark) {
        (Token::Plain, _) => None,
        (Token::Comment, false) => Some(0xa0a1a7),
        (Token::Comment, true) => Some(0x8b949e),
        (Token::Keyword, false) => Some(0xa626a4),
        (Token::Keyword, true) => Some(0xc678dd),
        (Token::String, false) => Some(0x40a14f),
        (Token::String, true) => Some(0x98c379),
        (Token::Number, false) => Some(0x986801),
        (Token::Number, true) => Some(0xd19a66),
        (Token::Function, false) => Some(0x4078f2),
        (Token::Function, true) => Some(0x61afef),
        (Token::Type, false) => Some(0x0184bc),
        (Token::Type, true) => Some(0x56b6c2),
        (Token::Macro, false) => Some(0xe45649),
        (Token::Macro, true) => Some(0xe06c75),
        (Token::Lifetime, false) => Some(0x986801),
        (Token::Lifetime, true) => Some(0xd19a66),
        (Token::Attribute, false) => Some(0xa0a1a7),
        (Token::Attribute, true) => Some(0x8b949e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<Token> {
        tokenize(source).into_iter().map(|(_, kind)| kind).collect()
    }

    fn code(source: &str, token: Token) -> Vec<String> {
        tokenize(source)
            .into_iter()
            .filter(|(_, kind)| *kind == token)
            .map(|(range, _)| source[range].to_string())
            .collect()
    }

    #[test]
    fn lexes_a_typical_snippet() {
        let source = r#"// Give each item a stable ID.
el_space().children(self.tag.dynamic.iter().map(|tag| {
    let id = tag.id;
    el_tag().id(("tag", id)).label("Primary").closable(true)
}))"#;
        assert_eq!(code(source, Token::Keyword), ["self", "let", "true"]);
        // "ID" sits inside the comment, so it is not tokenized on its own.
        assert_eq!(code(source, Token::Type), Vec::<String>::new());
        assert_eq!(code(source, Token::Macro), Vec::<String>::new());
        assert_eq!(code(source, Token::String), ["\"tag\"", "\"Primary\""]);
        assert_eq!(code(source, Token::Number), Vec::<String>::new());
        assert_eq!(code(source, Token::Comment).len(), 1);
    }

    #[test]
    fn lexes_macros_types_numbers_and_lifetimes() {
        let source = r#"el_init!(cx, locale: Locale::SimplifiedChinese)?;
let count: u32 = 1_024 + 0xff;
fn first<'a>(items: &'a [&'static str]) -> Option<&'a str> {
    format!("{:?}", items)
}"#;
        assert_eq!(code(source, Token::Macro), ["el_init!", "format!"]);
        assert_eq!(
            code(source, Token::Type),
            ["Locale", "SimplifiedChinese", "Option"]
        );
        assert_eq!(code(source, Token::Number), ["1_024", "0xff"]);
        assert_eq!(code(source, Token::Lifetime), ["'a", "'a", "'static", "'a"]);
        assert_eq!(code(source, Token::Keyword), ["let", "fn"]);
    }

    #[test]
    fn lexes_attributes_and_block_comments() {
        let source = "#[derive(Default)]\n/* kept\n for later */ struct State;";
        assert_eq!(code(source, Token::Attribute), ["#[derive(Default)]"]);
        assert_eq!(code(source, Token::Comment), ["/* kept\n for later */"]);
        assert_eq!(kinds("struct"), [Token::Keyword]);
    }

    #[test]
    fn spans_cover_the_whole_text_without_gaps() {
        let sources = [
            "el_button().label(\"Hi\") // done\n",
            "let x = 'a' + r\"raw\";",
            "unclosed \"string at end",
            "/* unclosed comment",
            "#[cfg(test)]",
            "",
        ];
        for source in sources {
            let mut cursor = 0;
            for (range, _) in tokenize(source) {
                assert_eq!(range.start, cursor, "gap or overlap in {source:?}");
                cursor = range.end;
            }
            assert_eq!(cursor, source.len(), "coverage ends early in {source:?}");
        }
    }

    #[test]
    fn char_literals_are_strings_not_lifetimes() {
        let source = "let separator = ',';";
        assert_eq!(code(source, Token::String), ["','"]);
        assert!(code(source, Token::Lifetime).is_empty());
    }

    #[test]
    fn plain_tokens_are_merged_and_colors_are_theme_aware() {
        let source = "el_tag()";
        assert_eq!(kinds(source), [Token::Function, Token::Plain]);
        assert_eq!(color(Token::Plain, true), None);
        assert_eq!(color(Token::Keyword, false), Some(0xa626a4));
        assert_eq!(color(Token::Keyword, true), Some(0xc678dd));
    }
}
