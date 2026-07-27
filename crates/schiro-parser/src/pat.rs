use schiro_ast::*;
use schiro_lexer::token::TokenKind;

use crate::parser::{ParseError, Parser};

impl Parser {
    // ========================================================================
    // Patterns
    // ========================================================================

    pub(crate) fn parse_pattern(&mut self) -> Option<Pattern> {
        self.parse_pattern_or()
    }

    fn parse_pattern_or(&mut self) -> Option<Pattern> {
        let mut left = self.parse_pattern_atom()?;
        while self.consume(&TokenKind::Pipe).is_some() {
            let right = self.parse_pattern_atom()?;
            left = Pattern::Or(Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn parse_pattern_atom(&mut self) -> Option<Pattern> {
        let tok = self.advance()?;
        match &tok.kind {
            TokenKind::Underscore => Some(Pattern::Wildcard),
            TokenKind::IntLiteral(s) => Some(Pattern::Literal(Literal::Int(s.clone()))),
            TokenKind::FloatLiteral(s) => Some(Pattern::Literal(Literal::Float(s.clone()))),
            TokenKind::StringLiteral(s) => Some(Pattern::Literal(Literal::String(s.clone()))),
            TokenKind::CharLiteral(c) => Some(Pattern::Literal(Literal::Char(*c))),
            TokenKind::True => Some(Pattern::Literal(Literal::Bool(true))),
            TokenKind::False => Some(Pattern::Literal(Literal::Bool(false))),
            TokenKind::Null => Some(Pattern::Literal(Literal::Null)),
            TokenKind::Identifier(name) => {
                if self.consume(&TokenKind::LParen).is_some() {
                    let patterns = if self.check(&TokenKind::RParen) {
                        vec![]
                    } else {
                        let mut list = vec![self.parse_pattern()?];
                        while self.consume(&TokenKind::Comma).is_some() {
                            list.push(self.parse_pattern()?);
                        }
                        list
                    };
                    self.expect(&TokenKind::RParen)?;
                    Some(Pattern::DestructureVariant {
                        name: name.clone(),
                        patterns,
                    })
                } else {
                    Some(Pattern::Identifier(name.clone()))
                }
            }
            TokenKind::LBracket => {
                if self.check(&TokenKind::RBracket) {
                    self.advance();
                    Some(Pattern::DestructureTuple(vec![]))
                } else {
                    let mut patterns = vec![self.parse_pattern()?];
                    while self.consume(&TokenKind::Comma).is_some() {
                        if let Some(p) = self.parse_pattern() {
                            patterns.push(p);
                        }
                    }
                    self.expect(&TokenKind::RBracket)?;
                    Some(Pattern::DestructureTuple(patterns))
                }
            }
            _ => {
                self.errors.push(ParseError::new(
                    format!("expected pattern, found {}", tok.kind),
                    &tok,
                ));
                None
            }
        }
    }
}
