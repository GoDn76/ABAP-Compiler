use super::diagnostics::{DiagnosticBag, Position};
use super::tokens::{Token, TokenKind, KEYWORDS};
use regex::Regex;
use std::sync::LazyLock;

static COMPOUND_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)(SELECT-OPTIONS|NO-INTERVALS|NO-EXTENSION|NO-GAPS|SELECTION-SCREEN|START-OF-SELECTION|END-OF-SELECTION|VALUE-REQUEST|HELP-REQUEST|USER-COMMAND|NO-DISPLAY|READ-ONLY|CLASS-DATA|CLASS-METHODS)(?:\s|[:,.]|$)").unwrap()
});

pub struct Lexer {
    source: Vec<char>,
    diagnostics: DiagnosticBag,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            diagnostics: DiagnosticBag::new(),
        }
    }

    pub fn tokenize(mut self) -> (Vec<Token>, DiagnosticBag) {
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut line = 1;
        let mut column = 1;
        let mut line_start = true;

        let len = self.source.len();

        while i < len {
            let ch = self.source[i];

            if ch == '\r' {
                i += 1;
                continue;
            }

            if ch.is_whitespace() {
                if ch == '\n' {
                    line += 1;
                    column = 1;
                    line_start = true;
                } else {
                    column += 1;
                }
                i += 1;
                continue;
            }

            // ABAP full line comment starts with * at column 1 / lineStart
            if ch == '*' && line_start {
                while i < len && self.source[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            // ABAP inline comment starts with "
            if ch == '"' {
                while i < len && self.source[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            let start = Position { line, column, offset: i };

            // Check compound keywords with hyphens
            let remaining_slice: String = self.source[i..].iter().take(40).collect();
            if let Some(mat) = COMPOUND_REGEX.find(&remaining_slice) {
                // The regex captures compound keyword group 1
                let cap = COMPOUND_REGEX.captures(&remaining_slice).unwrap();
                let word_len = cap.get(1).unwrap().as_str().chars().count();
                let word: String = self.source[i..i + word_len].iter().collect();

                for _ in 0..word_len {
                    i += 1;
                    column += 1;
                }
                line_start = false;

                let end = Position { line, column, offset: i };
                tokens.push(Token::new(TokenKind::Keyword, word, start, end));
                continue;
            }

            // Identifiers / regular keywords
            if ch.is_ascii_alphabetic() || ch == '_' {
                let mut s = String::new();
                while i < len && (self.source[i].is_ascii_alphanumeric() || self.source[i] == '_') {
                    s.push(self.source[i]);
                    i += 1;
                    column += 1;
                }
                line_start = false;

                let end = Position { line, column, offset: i };
                let is_kw = KEYWORDS.contains(s.to_ascii_uppercase().as_str());
                tokens.push(Token::new(if is_kw { TokenKind::Keyword } else { TokenKind::Identifier }, s, start, end));
                continue;
            }

            // Numbers: integer or decimal
            if ch.is_ascii_digit() {
                let mut s = String::new();
                while i < len && self.source[i].is_ascii_digit() {
                    s.push(self.source[i]);
                    i += 1;
                    column += 1;
                }

                let mut decimal = false;
                if i < len && self.source[i] == '.' && (i + 1 < len && self.source[i + 1].is_ascii_digit()) {
                    decimal = true;
                    s.push(self.source[i]);
                    i += 1;
                    column += 1;
                    while i < len && self.source[i].is_ascii_digit() {
                        s.push(self.source[i]);
                        i += 1;
                        column += 1;
                    }
                }
                line_start = false;

                let end = Position { line, column, offset: i };
                tokens.push(Token::new(if decimal { TokenKind::Decimal } else { TokenKind::Integer }, s, start, end));
                continue;
            }

            // Strings with single quotes and '' escape
            if ch == '\'' {
                i += 1;
                column += 1;
                let mut val = String::new();
                let mut closed = false;

                while i < len {
                    if self.source[i] == '\'' {
                        i += 1;
                        column += 1;
                        if i < len && self.source[i] == '\'' {
                            val.push('\'');
                            i += 1;
                            column += 1;
                            continue;
                        }
                        closed = true;
                        break;
                    }
                    if self.source[i] == '\n' {
                        break;
                    }
                    val.push(self.source[i]);
                    i += 1;
                    column += 1;
                }

                line_start = false;
                let end = Position { line, column, offset: i };
                if !closed {
                    self.diagnostics.error("LEX001", "Unterminated string literal.", start, end);
                }
                tokens.push(Token::new(TokenKind::String, val, start, end));
                continue;
            }

            // Two-character and three-character tokens
            let two: String = self.source[i..].iter().take(2).collect();
            let three: String = self.source[i..].iter().take(3).collect();

            if three == "->>" {
                for _ in 0..3 { i += 1; column += 1; }
                line_start = false;
                tokens.push(Token::new(TokenKind::Arrow, three, start, Position { line, column, offset: i }));
                continue;
            }

            if two == "=>" || two == "->" || two == "<-" {
                for _ in 0..2 { i += 1; column += 1; }
                line_start = false;
                tokens.push(Token::new(TokenKind::Arrow, two, start, Position { line, column, offset: i }));
                continue;
            }

            if matches!(two.as_str(), "<>" | "<=" | ">=" | "==" | "+=" | "-=" | "*=" | "/=" | "&&" | "||") {
                for _ in 0..2 { i += 1; column += 1; }
                line_start = false;
                tokens.push(Token::new(TokenKind::Operator, two, start, Position { line, column, offset: i }));
                continue;
            }

            // Single character symbols
            i += 1;
            column += 1;
            line_start = false;
            let end = Position { line, column, offset: i };

            match ch {
                '=' | '+' | '-' | '/' | '*' | '<' | '>' | '!' | '&' | '|' => {
                    tokens.push(Token::new(TokenKind::Operator, ch.to_string(), start, end));
                }
                '.' => tokens.push(Token::new(TokenKind::Dot, ch.to_string(), start, end)),
                ',' => tokens.push(Token::new(TokenKind::Comma, ch.to_string(), start, end)),
                ':' => tokens.push(Token::new(TokenKind::Colon, ch.to_string(), start, end)),
                '(' => tokens.push(Token::new(TokenKind::LParen, ch.to_string(), start, end)),
                ')' => tokens.push(Token::new(TokenKind::RParen, ch.to_string(), start, end)),
                '[' => tokens.push(Token::new(TokenKind::LBracket, ch.to_string(), start, end)),
                ']' => tokens.push(Token::new(TokenKind::RBracket, ch.to_string(), start, end)),
                '@' => tokens.push(Token::new(TokenKind::At, ch.to_string(), start, end)),
                '#' => tokens.push(Token::new(TokenKind::Hash, ch.to_string(), start, end)),
                '~' => tokens.push(Token::new(TokenKind::Tilde, ch.to_string(), start, end)),
                _ => {
                    self.diagnostics.error("LEX002", format!("Unexpected character '{}'.", ch), start, end);
                }
            }
        }

        let eof_pos = Position { line, column, offset: i };
        tokens.push(Token::eof(eof_pos));

        (tokens, self.diagnostics)
    }
}
