use schiro_ast::*;
use schiro_lexer::token::{Span, Token, TokenKind};

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub(crate) fn new(msg: impl Into<String>, t: &Token) -> Self {
        Self {
            message: msg.into(),
            span: t.span,
            line: t.line,
            column: t.column,
        }
    }
}

// ============================================================================
// Fixity
// ============================================================================

#[derive(Clone, Copy)]
pub enum Fixity {
    Left,
    Suffix,
}

// ============================================================================
// Parser
// ============================================================================

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    // -- core helpers -------------------------------------------------------

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    pub(crate) fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(t)
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    pub(crate) fn consume(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            self.advance()
        } else {
            None
        }
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            self.advance()
        } else {
            let found = self.peek().cloned().unwrap_or_else(|| {
                Token::new(TokenKind::Eof, Span::empty(), 0, 0)
            });
            let msg = format!("expected {kind}, found {}", found.kind);
            let err = ParseError::new(msg, &found);
            self.errors.push(err);
            None
        }
    }

    pub(crate) fn check_ident(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Identifier(_)))
    }

    pub(crate) fn expect_ident(&mut self) -> Option<String> {
        match self.peek_kind()? {
            TokenKind::Identifier(s) => {
                let s = s.clone();
                self.advance();
                Some(s)
            }
            _ => {
                let found = self.peek().cloned().unwrap_or_else(|| {
                    Token::new(TokenKind::Eof, Span::empty(), 0, 0)
                });
                let msg = format!("expected identifier, found {}", found.kind);
                let err = ParseError::new(msg, &found);
                self.errors.push(err);
                None
            }
        }
    }

    pub(crate) fn peek_ahead_is_lparen(&self) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map_or(false, |t| t.kind == TokenKind::LParen)
    }

    pub(crate) fn peek_ahead_is_new(&self) -> bool {
        let mut i = self.pos;
        while let Some(t) = self.tokens.get(i) {
            if t.kind == TokenKind::Fn {
                i += 1;
                continue;
            }
            return t.kind == TokenKind::New;
        }
        false
    }

    // ========================================================================
    // Entry point
    // ========================================================================

    pub fn parse(&mut self) -> CompilationUnit {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        while let Some(k) = self.peek_kind().cloned() {
            match k {
                TokenKind::Import => {
                    if let Some(imp) = self.parse_import() {
                        imports.push(imp);
                    }
                }
                _ => break,
            }
        }

        while self.peek().is_some() {
            let before = self.pos;
            if let Some(decl) = self.parse_top_level_decl() {
                declarations.push(decl);
            }
            if self.pos == before {
                self.advance();
            }
        }

        CompilationUnit {
            imports,
            declarations,
        }
    }

    // ========================================================================
    // Imports
    // ========================================================================

    fn parse_import(&mut self) -> Option<ImportDirective> {
        self.advance()?;
        let path = self.parse_module_path()?;
        let alias = if self.consume(&TokenKind::As).is_some() {
            self.expect_ident()
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon);
        Some(ImportDirective { path, alias })
    }

    fn parse_module_path(&mut self) -> Option<Vec<String>> {
        let mut parts = vec![self.expect_ident()?];
        while self.consume(&TokenKind::Dot).is_some() {
            parts.push(self.expect_ident()?);
        }
        Some(parts)
    }

    // ========================================================================
    // Block
    // ========================================================================

    pub(crate) fn parse_block(&mut self) -> Option<Block> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
            let before = self.pos;
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            }
            if self.pos == before {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace);
        Some(stmts)
    }
}
