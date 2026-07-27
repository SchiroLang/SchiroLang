use schiro_ast::*;
use schiro_lexer::token::TokenKind;

use crate::parser::Parser;

impl Parser {
    // ========================================================================
    // Statements
    // ========================================================================

    pub(crate) fn parse_statement(&mut self) -> Option<Statement> {
        match self.peek_kind()? {
            TokenKind::Let => Some(Statement::Let(self.parse_let_decl()?)),
            TokenKind::Return => self.parse_return_stmt().map(Statement::Return),
            TokenKind::Break => self.parse_break_stmt().map(Statement::Break),
            TokenKind::Continue => {
                self.advance()?;
                self.expect(&TokenKind::Semicolon);
                Some(Statement::Continue)
            }
            TokenKind::If => {
                self.advance()?;
                let expr = self.parse_if_expr()?;
                Some(Statement::Expression(Expression::If(expr)))
            }
            TokenKind::Match => {
                self.advance()?;
                let expr = self.parse_match_expr()?;
                Some(Statement::Expression(Expression::Match(expr)))
            }
            TokenKind::Loop => {
                self.advance()?;
                let expr = self.parse_loop_expr()?;
                Some(Statement::Expression(Expression::Loop(expr)))
            }
            TokenKind::While => {
                self.advance()?;
                let expr = self.parse_while_expr()?;
                Some(Statement::Expression(Expression::While(expr)))
            }
            TokenKind::For => {
                self.advance()?;
                let expr = self.parse_for_expr()?;
                Some(Statement::Expression(Expression::For(expr)))
            }
            TokenKind::LBrace => Some(Statement::Block(self.parse_block()?)),
            TokenKind::Super => {
                if self.peek_ahead_is_lparen() {
                    self.parse_super_call().map(Statement::SuperCall)
                } else {
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon);
                    Some(Statement::Expression(expr))
                }
            }
            _ => {
                if let Some(stmt) = self.try_parse_assignment() {
                    return Some(stmt);
                }
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::Semicolon);
                Some(Statement::Expression(expr))
            }
        }
    }

    // ========================================================================
    // Assignment
    // ========================================================================

    fn try_parse_assignment(&mut self) -> Option<Statement> {
        let saved = self.pos;
        let lvalue = self.parse_lvalue();
        if lvalue.is_some() && self.consume(&TokenKind::Equals).is_some() {
            let value = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon);
            Some(Statement::Assignment(Assignment {
                lvalue: lvalue.unwrap(),
                value,
            }))
        } else {
            self.pos = saved;
            None
        }
    }

    fn parse_lvalue(&mut self) -> Option<LValue> {
        let tok = self.advance()?;
        match &tok.kind {
            TokenKind::Identifier(name) => {
                let mut lv = LValue::Variable(name.clone());
                loop {
                    if self.consume(&TokenKind::Dot).is_some() {
                        let field = self.expect_ident()?;
                        lv = LValue::Field(Box::new(lv), field);
                    } else if self.consume(&TokenKind::LBracket).is_some() {
                        let index = self.parse_expression()?;
                        self.expect(&TokenKind::RBracket)?;
                        lv = LValue::Index(Box::new(lv), Box::new(index));
                    } else {
                        break;
                    }
                }
                Some(lv)
            }
            _ => {
                self.pos -= 1;
                None
            }
        }
    }

    // ========================================================================
    // Let declaration
    // ========================================================================

    fn parse_let_decl(&mut self) -> Option<LetDecl> {
        self.advance()?;
        let pattern = self.parse_pattern()?;
        let type_ = if self.consume(&TokenKind::Colon).is_some() {
            self.parse_type_ref()
        } else {
            None
        };
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon);
        Some(LetDecl {
            pattern,
            type_,
            value,
        })
    }

    // ========================================================================
    // Return / Break / SuperCall
    // ========================================================================

    fn parse_return_stmt(&mut self) -> Option<Option<Expression>> {
        self.advance()?;
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            Some(None)
        } else {
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon);
            Some(Some(expr))
        }
    }

    fn parse_break_stmt(&mut self) -> Option<Option<Expression>> {
        self.advance()?;
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            Some(None)
        } else {
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon);
            Some(Some(expr))
        }
    }

    fn parse_super_call(&mut self) -> Option<Vec<Expression>> {
        self.advance()?;
        self.expect(&TokenKind::LParen)?;
        let args = self.parse_arg_list();
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Semicolon);
        Some(args)
    }
}
