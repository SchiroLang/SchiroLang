use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.pos..].chars();
        chars.next()?;
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        self.column += 1;
        Some(c)
    }

    fn consume_while<F>(&mut self, mut predicate: F) -> &'a str
    where
        F: FnMut(char) -> bool,
    {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if predicate(c) {
                self.advance();
            } else {
                break;
            }
        }
        &self.source[start..self.pos]
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        let mut depth = 1u32;
        while depth > 0 {
            match (self.peek(), self.peek_next()) {
                (Some('/'), Some('*')) => {
                    self.advance();
                    self.advance();
                    depth += 1;
                }
                (Some('*'), Some('/')) => {
                    self.advance();
                    self.advance();
                    depth -= 1;
                }
                (Some('\n'), _) => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                (Some(_), _) => {
                    self.advance();
                }
                (None, _) => {
                    break;
                }
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                Some('/') if self.peek_next() == Some('/') => {
                    self.skip_line_comment();
                }
                Some('/') if self.peek_next() == Some('*') => {
                    self.advance();
                    self.advance();
                    self.skip_block_comment();
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self, quote: char) -> TokenKind {
        let start = self.pos;
        loop {
            match self.advance() {
                Some('\\') => {
                    self.advance();
                }
                Some(c) if c == quote => {
                    let raw = &self.source[start..self.pos - 1];
                    let unescaped = self.unescape_string(raw);
                    return match unescaped {
                        Ok(s) => TokenKind::StringLiteral(s),
                        Err(msg) => TokenKind::Error(msg),
                    };
                }
                Some('\n') | None => {
                    return TokenKind::Error("unterminated string literal".into());
                }
                _ => {}
            }
        }
    }

    fn read_char(&mut self) -> TokenKind {
        let c = self.advance();
        match c {
            Some('\\') => {
                let escaped = self.read_escape();
                match (escaped, self.advance()) {
                    (Ok(ch), Some('\'')) => TokenKind::CharLiteral(ch),
                    (Ok(_), Some(c)) => {
                        TokenKind::Error(format!("expected closing ' after char escape, found '{c}'"))
                    }
                    (Ok(_), None) => TokenKind::Error("unterminated char literal".into()),
                    (Err(msg), _) => TokenKind::Error(msg),
                }
            }
            Some(ch) if ch == '\'' => TokenKind::Error("empty char literal".into()),
            Some(ch) => match self.advance() {
                Some('\'') => TokenKind::CharLiteral(ch),
                Some(c) => TokenKind::Error(format!("expected closing ' after char '{ch}', found '{c}'")),
                None => TokenKind::Error("unterminated char literal".into()),
            },
            None => TokenKind::Error("unterminated char literal".into()),
        }
    }

    fn read_escape(&mut self) -> Result<char, String> {
        match self.advance() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('0') => Ok('\0'),
            Some('\\') => Ok('\\'),
            Some('"') => Ok('"'),
            Some('\'') => Ok('\''),
            Some('x') => self.read_hex_escape(2),
            Some('u') => {
                if self.advance() != Some('{') {
                    return Err("expected '{{' after \\u".into());
                }
                let result = self.read_hex_escape_until('}');
                if self.advance() != Some('}') {
                    return Err("expected '}}' after unicode escape".into());
                }
                result
            }
            Some(c) => Err(format!("invalid escape sequence '\\{c}'")),
            None => Err("unterminated escape sequence".into()),
        }
    }

    fn read_hex_escape(&mut self, digits: usize) -> Result<char, String> {
        let mut hex = String::with_capacity(digits);
        for _ in 0..digits {
            match self.advance() {
                Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                Some(c) => return Err(format!("invalid hex digit '{c}' in escape")),
                None => return Err("unterminated hex escape".into()),
            }
        }
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| format!("invalid unicode value '\\x{hex}'"))
    }

    fn read_hex_escape_until(&mut self, end: char) -> Result<char, String> {
        let mut hex = String::new();
        loop {
            match self.peek() {
                Some(c) if c == end => break,
                Some(c) if c.is_ascii_hexdigit() => {
                    self.advance();
                    hex.push(c);
                }
                Some(c) => return Err(format!("invalid hex digit '{c}' in unicode escape")),
                None => return Err("unterminated unicode escape".into()),
            }
        }
        if hex.is_empty() {
            return Err("empty unicode escape".into());
        }
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| format!("invalid unicode value '\\u{{{hex}}}'"))
    }

    fn unescape_string(&self, raw: &str) -> Result<String, String> {
        let mut result = String::with_capacity(raw.len());
        let mut chars = raw.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('0') => result.push('\0'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('x') => {
                        let h1 = chars.next().and_then(|c| c.to_digit(16)).ok_or("bad hex escape")?;
                        let h2 = chars.next().and_then(|c| c.to_digit(16)).ok_or("bad hex escape")?;
                        let code = (h1 << 4) | h2;
                        result.push(char::from_u32(code).ok_or("bad unicode value")?);
                    }
                    Some('u') => {
                        if chars.next() != Some('{') {
                            return Err("expected '{{' in unicode escape".into());
                        }
                        let mut hex = String::new();
                        loop {
                            match chars.next() {
                                Some('}') => break,
                                Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                                Some(c) => return Err(format!("invalid char in unicode escape: '{c}'")),
                                None => return Err("unterminated unicode escape".into()),
                            }
                        }
                        let code = u32::from_str_radix(&hex, 16).map_err(|_| "bad unicode escape")?;
                        result.push(char::from_u32(code).ok_or("bad unicode value")?);
                    }
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => return Err("unterminated escape".into()),
                }
            } else {
                result.push(c);
            }
        }
        Ok(result)
    }

    fn read_number(&mut self, first: char) -> TokenKind {
        let start = self.pos - first.len_utf8();
        self.consume_while(|c| c.is_ascii_digit());
        let mut is_float = false;

        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            self.advance();
            self.consume_while(|c| c.is_ascii_digit());
        }

        if let Some('e') | Some('E') = self.peek() {
            is_float = true;
            self.advance();
            if let Some('+') | Some('-') = self.peek() {
                self.advance();
            }
            if self.peek().map_or(true, |c| !c.is_ascii_digit()) {
                return TokenKind::Error("invalid float literal: expected digit after exponent".into());
            }
            self.consume_while(|c| c.is_ascii_digit());
        }

        let raw = &self.source[start..self.pos];
        if is_float {
            TokenKind::FloatLiteral(raw.to_string())
        } else {
            TokenKind::IntLiteral(raw.to_string())
        }
    }

    fn read_identifier_or_keyword(&mut self, first: char) -> TokenKind {
        let start = self.pos - first.len_utf8();
        self.consume_while(|c| c == '_' || c.is_alphanumeric());
        let word = &self.source[start..self.pos];

        let keyword = match word {
            "import" => TokenKind::Import,
            "as" => TokenKind::As,
            "type" => TokenKind::Type,
            "abstract" => TokenKind::Abstract,
            "class" => TokenKind::Class,
            "extends" => TokenKind::Extends,
            "impl" => TokenKind::Impl,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "fn" => TokenKind::Fn,
            "new" => TokenKind::New,
            "static" => TokenKind::Static,
            "virtual" => TokenKind::Virtual,
            "override" => TokenKind::Override,
            "trait" => TokenKind::Trait,
            "prop" => TokenKind::Prop,
            "get" => TokenKind::Get,
            "set" => TokenKind::Set,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "loop" => TokenKind::Loop,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "self" => TokenKind::Self_,
            "public" => TokenKind::Public,
            "protected" => TokenKind::Protected,
            "private" => TokenKind::Private,
            "mut" => TokenKind::Mut,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "Self" => TokenKind::SelfType,
            "_" => TokenKind::Underscore,
            _ => return TokenKind::Identifier(word.to_string()),
        };
        keyword
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.pos;
        let line = self.line;
        let col = self.column;

        let kind = match self.advance() {
            None => TokenKind::Eof,

            Some(c) if c == '_' || c.is_alphabetic() => self.read_identifier_or_keyword(c),

            Some(c) if c.is_ascii_digit() => self.read_number(c),

            Some('"') => self.read_string('"'),

            Some('\'') => self.read_char(),

            Some(';') => TokenKind::Semicolon,
            Some(':') => TokenKind::Colon,
            Some(',') => TokenKind::Comma,
            Some('(') => TokenKind::LParen,
            Some(')') => TokenKind::RParen,
            Some('{') => TokenKind::LBrace,
            Some('}') => TokenKind::RBrace,
            Some('[') => TokenKind::LBracket,
            Some(']') => TokenKind::RBracket,
            Some('?') => TokenKind::Question,
            Some('%') => TokenKind::Percent,

            Some('=') => match self.peek() {
                Some('=') => { self.advance(); TokenKind::EqEq }
                Some('>') => { self.advance(); TokenKind::FatArrow }
                _ => TokenKind::Equals,
            },

            Some('!') => match self.peek() {
                Some('=') => { self.advance(); TokenKind::BangEq }
                _ => TokenKind::Bang,
            },

            Some('<') => match self.peek() {
                Some('=') => { self.advance(); TokenKind::LessEq }
                _ => TokenKind::Less,
            },

            Some('>') => match self.peek() {
                Some('=') => { self.advance(); TokenKind::GreaterEq }
                _ => TokenKind::Greater,
            },

            Some('|') => match self.peek() {
                Some('|') => { self.advance(); TokenKind::PipePipe }
                Some('>') => { self.advance(); TokenKind::PipeGreater }
                _ => TokenKind::Pipe,
            },

            Some('&') => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AmpAmp
                } else {
                    let start_for_mut = self.pos;
                    let mut_saved = (self.line, self.column);
                    if let Some(c) = self.peek() {
                        if c == '_' || c.is_alphabetic() {
                            self.advance();
                            self.consume_while(|c| c == '_' || c.is_alphanumeric());
                            let word = &self.source[start_for_mut..self.pos];
                            if word == "mut" {
                                TokenKind::AmpMut
                            } else {
                                self.pos = start_for_mut;
                                self.line = mut_saved.0;
                                self.column = mut_saved.1;
                                TokenKind::Amp
                            }
                        } else {
                            TokenKind::Amp
                        }
                    } else {
                        TokenKind::Amp
                    }
                }
            }

            Some('-') => match self.peek() {
                Some('>') => { self.advance(); TokenKind::Arrow }
                _ => TokenKind::Minus,
            },

            Some('+') => TokenKind::Plus,
            Some('*') => TokenKind::Star,

            Some('/') => {
                match self.peek() {
                    Some('/') => {
                        self.skip_line_comment();
                        return self.next_token();
                    }
                    Some('*') => {
                        self.advance();
                        self.skip_block_comment();
                        return self.next_token();
                    }
                    _ => TokenKind::Slash,
                }
            }

            Some('.') => match self.peek() {
                Some('.') => { self.advance(); TokenKind::DotDot }
                _ => TokenKind::Dot,
            },

            Some(c) => TokenKind::Error(format!("unexpected character '{c}'")),
        };

        let end = self.pos;
        Token::new(kind, Span::new(start, end), line, col)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}
