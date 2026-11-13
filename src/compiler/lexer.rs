use super::diagnostics::{DiagnosticBag, Position};
use super::tokens::{Token, TokenKind, KEYWORDS};
use regex::Regex;
use std::sync::LazyLock;

static COMPOUND_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)(SELECT-OPTIONS|NO-INTERVALS|NO-EXTENSION|NO-GAPS|SELECTION-SCREEN|START-OF-SELECTION|END-OF-SELECTION|VALUE-REQUEST|HELP-REQUEST|USER-COMMAND|NO-DISPLAY|READ-ONLY|CLASS-DATA|CLASS-METHODS|FIELD-SYMBOLS|FIELD-SYMBOL)(?:\s|[:,.]|$)").unwrap()
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
            if COMPOUND_REGEX.is_match(&remaining_slice) {
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
                while i < len {
                    let cur = self.source[i];
                    if cur.is_ascii_alphanumeric() || cur == '_' {
                        s.push(cur);
                        i += 1;
                        column += 1;
                    } else if cur == '-' && i + 1 < len && (self.source[i + 1].is_ascii_alphanumeric() || self.source[i + 1] == '_') {
                        s.push(cur);
                        i += 1;
                        column += 1;
                    } else {
                        break;
                    }
                }
                line_start = false;

                let end = Position { line, column, offset: i };
                let upper = s.to_ascii_uppercase();

                // Explicit 2-character ABAP comparison operators
                let comp_op = match upper.as_str() {
                    "CP" => Some(TokenKind::CP),
                    "NP" => Some(TokenKind::NP),
                    "CS" => Some(TokenKind::CS),
                    "NS" => Some(TokenKind::NS),
                    "CA" => Some(TokenKind::CA),
                    "NA" => Some(TokenKind::NA),
                    "CO" => Some(TokenKind::CO),
                    "CN" => Some(TokenKind::CN),
                    "EQ" => Some(TokenKind::EQ),
                    "NE" => Some(TokenKind::NE),
                    "LT" => Some(TokenKind::LT),
                    "GT" => Some(TokenKind::GT),
                    "LE" => Some(TokenKind::LE),
                    "GE" => Some(TokenKind::GE),
                    _ => None,
                };

                if let Some(kind) = comp_op {
                    tokens.push(Token::new(kind, s, start, end));
                    continue;
                }

                let is_kw = KEYWORDS.contains(upper.as_str());
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
                        if i + 1 < len && self.source[i + 1] == '\'' {
                            val.push('\'');
                            i += 2;
                            column += 2;
                            continue;
                        }
                        i += 1;
                        column += 1;
                        closed = true;
                        break;
                    }
                    if self.source[i] == '\\' && i + 1 < len {
                        if self.source[i + 1] == '\'' {
                            val.push('\'');
                            i += 2;
                            column += 2;
                            continue;
                        } else if self.source[i + 1] == '\\' {
                            val.push('\\');
                            i += 2;
                            column += 2;
                            continue;
                        }
                    }
                    if self.source[i] == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    val.push(self.source[i]);
                    i += 1;
                }

                line_start = false;
                let end = Position { line, column, offset: i };
                if !closed {
                    self.diagnostics.error("LEX001", "Unterminated string literal.", start, end);
                }
                tokens.push(Token::new(TokenKind::StringLiteral, val, start, end));
                continue;
            }

            // String objects with backticks `...` and `` escape
            if ch == '`' {
                i += 1;
                column += 1;
                let mut val = String::new();
                let mut closed = false;

                while i < len {
                    if self.source[i] == '`' {
                        if i + 1 < len && self.source[i + 1] == '`' {
                            val.push('`');
                            i += 2;
                            column += 2;
                            continue;
                        }
                        i += 1;
                        column += 1;
                        closed = true;
                        break;
                    }
                    if self.source[i] == '\\' && i + 1 < len {
                        if self.source[i + 1] == '`' {
                            val.push('`');
                            i += 2;
                            column += 2;
                            continue;
                        } else if self.source[i + 1] == '\\' {
                            val.push('\\');
                            i += 2;
                            column += 2;
                            continue;
                        }
                    }
                    if self.source[i] == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    val.push(self.source[i]);
                    i += 1;
                }

                line_start = false;
                let end = Position { line, column, offset: i };
                if !closed {
                    self.diagnostics.error("LEX001", "Unterminated string object.", start, end);
                }
                tokens.push(Token::new(TokenKind::StringObject, val, start, end));
                continue;
            }

            // String templates delimited by |...|
            if ch == '|' {
                i += 1;
                column += 1;
                let mut val = String::new();
                let mut closed = false;
                let mut brace_depth = 0;

                while i < len {
                    if self.source[i] == '|' && brace_depth == 0 {
                        i += 1;
                        column += 1;
                        closed = true;
                        break;
                    }
                    if self.source[i] == '{' {
                        brace_depth += 1;
                    } else if self.source[i] == '}' && brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    if self.source[i] == '\\' && i + 1 < len {
                        val.push(self.source[i + 1]);
                        i += 2;
                        column += 2;
                        continue;
                    }
                    if self.source[i] == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    val.push(self.source[i]);
                    i += 1;
                }

                line_start = false;
                let end = Position { line, column, offset: i };
                if !closed {
                    self.diagnostics.error("LEX003", "Unterminated string template.", start, end);
                }
                tokens.push(Token::new(TokenKind::StringTemplate, val, start, end));
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

            // Field symbols: <fs_name>
            if ch == '<' {
                let mut j = i + 1;
                while j < len && (self.source[j].is_ascii_alphanumeric() || self.source[j] == '_') {
                    j += 1;
                }
                if j > i + 1 && j < len && self.source[j] == '>' {
                    let word_len = j - i + 1;
                    let word: String = self.source[i..=j].iter().collect();
                    for _ in 0..word_len {
                        i += 1;
                        column += 1;
                    }
                    line_start = false;
                    let end = Position { line, column, offset: i };
                    tokens.push(Token::new(TokenKind::Identifier, word, start, end));
                    continue;
                }
            }

            // Single character symbols
            i += 1;
            column += 1;
            line_start = false;
            let end = Position { line, column, offset: i };

            match ch {
                '=' | '+' | '-' | '/' | '*' | '<' | '>' | '!' | '&' => {
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

        let unchained = unchain_tokens(tokens);
        (unchained, self.diagnostics)
    }
}

pub fn unchain_tokens(raw: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::with_capacity(raw.len() * 2);
    let mut prefix = Vec::new();
    let mut chain_prefix: Option<Vec<Token>> = None;
    let mut paren_depth = 0;
    let mut in_begin_of = false;
    let len = raw.len();

    for i in 0..len {
        let tok = &raw[i];

        if tok.upper == "BEGIN" && i + 1 < len && raw[i + 1].upper == "OF" {
            in_begin_of = true;
        }

        match tok.kind {
            TokenKind::LParen | TokenKind::LBracket => {
                paren_depth += 1;
                if chain_prefix.is_none() {
                    prefix.push(tok.clone());
                }
                out.push(tok.clone());
            }
            TokenKind::RParen | TokenKind::RBracket => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
                if chain_prefix.is_none() {
                    prefix.push(tok.clone());
                }
                out.push(tok.clone());
            }
            TokenKind::Colon if paren_depth == 0 && chain_prefix.is_none() => {
                chain_prefix = Some(prefix.clone());
            }
            TokenKind::Comma if paren_depth == 0 && chain_prefix.is_some() => {
                if in_begin_of {
                    out.push(tok.clone());
                    continue;
                }

                let dot_pos = tok.end;
                out.push(Token::new(TokenKind::Dot, ".".to_string(), tok.start, dot_pos));
                if let Some(ref cp) = chain_prefix {
                    for pt in cp {
                        out.push(pt.clone());
                    }
                }
            }
            TokenKind::Dot if paren_depth == 0 => {
                out.push(tok.clone());
                prefix.clear();
                chain_prefix = None;
                in_begin_of = false;
            }
            TokenKind::EOF => {
                out.push(tok.clone());
            }
            _ => {
                if tok.upper == "END" && i + 1 < len && raw[i + 1].upper == "OF" {
                    in_begin_of = false;
                }
                if chain_prefix.is_none() {
                    prefix.push(tok.clone());
                }
                out.push(tok.clone());
            }
        }
    }
    out
}
