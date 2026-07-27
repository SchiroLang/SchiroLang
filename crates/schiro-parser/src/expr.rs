use schiro_ast::*;
use schiro_lexer::token::TokenKind;

use crate::parser::{Fixity, ParseError, Parser};

impl Parser {
    // ========================================================================
    // Expressions (Pratt parser)
    // ========================================================================

    pub(crate) fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, min_bp: u8) -> Option<Expression> {
        let mut left = self.parse_prefix_expr()?;

        loop {
            let kind = match self.peek_kind() {
                Some(k) => k.clone(),
                None => break,
            };
            let (bp, fixity) = self.infix_bp(&kind);
            if bp < min_bp || bp == 0 {
                break;
            }
            self.advance();
            match fixity {
                Fixity::Left => {
                    let right = self.parse_expression_bp(bp)?;
                    left = self.make_binary(kind, left, right);
                }
                Fixity::Suffix => {
                    left = self.make_suffix(kind, left)?;
                }
            }
        }

        Some(left)
    }

    fn infix_bp(&self, kind: &TokenKind) -> (u8, Fixity) {
        match kind {
            TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::Less
            | TokenKind::Greater
            | TokenKind::LessEq
            | TokenKind::GreaterEq => (3, Fixity::Left),
            TokenKind::PipeGreater => (4, Fixity::Left),
            TokenKind::DotDot => (5, Fixity::Left),
            TokenKind::Plus | TokenKind::Minus => (6, Fixity::Left),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (7, Fixity::Left),
            TokenKind::Dot => (9, Fixity::Suffix),
            TokenKind::LParen => (9, Fixity::Suffix),
            TokenKind::LBracket => (9, Fixity::Suffix),
            TokenKind::Question => (9, Fixity::Suffix),
            TokenKind::Bang => (9, Fixity::Suffix),
            TokenKind::PipePipe => (1, Fixity::Left),
            TokenKind::AmpAmp => (2, Fixity::Left),
            _ => (0, Fixity::Left),
        }
    }

    fn make_binary(&self, kind: TokenKind, left: Expression, right: Expression) -> Expression {
        match kind {
            TokenKind::PipePipe => Expression::Or(Box::new(left), Box::new(right)),
            TokenKind::AmpAmp => Expression::And(Box::new(left), Box::new(right)),
            TokenKind::EqEq => Expression::Equal(Box::new(left), Box::new(right)),
            TokenKind::BangEq => Expression::NotEqual(Box::new(left), Box::new(right)),
            TokenKind::Less => Expression::Less(Box::new(left), Box::new(right)),
            TokenKind::Greater => Expression::Greater(Box::new(left), Box::new(right)),
            TokenKind::LessEq => Expression::LessEq(Box::new(left), Box::new(right)),
            TokenKind::GreaterEq => Expression::GreaterEq(Box::new(left), Box::new(right)),
            TokenKind::PipeGreater => Expression::Pipe(Box::new(left), Box::new(right)),
            TokenKind::DotDot => Expression::Range(Box::new(left), Box::new(right)),
            TokenKind::Plus => Expression::Add(Box::new(left), Box::new(right)),
            TokenKind::Minus => Expression::Sub(Box::new(left), Box::new(right)),
            TokenKind::Star => Expression::Mul(Box::new(left), Box::new(right)),
            TokenKind::Slash => Expression::Div(Box::new(left), Box::new(right)),
            TokenKind::Percent => Expression::Mod(Box::new(left), Box::new(right)),
            _ => unreachable!(),
        }
    }

    fn make_suffix(&mut self, kind: TokenKind, left: Expression) -> Option<Expression> {
        match kind {
            TokenKind::Dot => {
                let field = self.expect_ident()?;
                Some(Expression::FieldAccess(Box::new(left), field))
            }
            TokenKind::LParen => {
                let args = self.parse_arg_list();
                self.expect(&TokenKind::RParen);
                Some(Expression::Call(Box::new(left), args))
            }
            TokenKind::LBracket => {
                let index = self.parse_expression()?;
                self.expect(&TokenKind::RBracket)?;
                Some(Expression::Index(Box::new(left), Box::new(index)))
            }
            TokenKind::Question => Some(Expression::Unwrap(Box::new(left))),
            TokenKind::Bang => Some(Expression::ForceUnwrap(Box::new(left))),
            _ => unreachable!(),
        }
    }

    pub(crate) fn parse_arg_list(&mut self) -> Vec<Expression> {
        if self.check(&TokenKind::RParen) {
            return vec![];
        }
        let mut args = vec![];
        if let Some(e) = self.parse_expression() {
            args.push(e);
        }
        while self.consume(&TokenKind::Comma).is_some() {
            if let Some(e) = self.parse_expression() {
                args.push(e);
            }
        }
        args
    }

    // ========================================================================
    // Prefix expressions
    // ========================================================================

    fn parse_prefix_expr(&mut self) -> Option<Expression> {
        let tok = self.advance()?;
        match &tok.kind {
            TokenKind::Minus => {
                let expr = self.parse_expression_bp(8)?;
                Some(Expression::Neg(Box::new(expr)))
            }
            TokenKind::Bang => {
                let expr = self.parse_expression_bp(8)?;
                Some(Expression::Not(Box::new(expr)))
            }
            TokenKind::Amp => {
                let expr = self.parse_expression_bp(8)?;
                Some(Expression::Ref(Box::new(expr)))
            }
            TokenKind::AmpMut => {
                let expr = self.parse_expression_bp(8)?;
                Some(Expression::MutRef(Box::new(expr)))
            }
            TokenKind::IntLiteral(s) => Some(Expression::Literal(Literal::Int(s.clone()))),
            TokenKind::FloatLiteral(s) => Some(Expression::Literal(Literal::Float(s.clone()))),
            TokenKind::StringLiteral(s) => Some(Expression::Literal(Literal::String(s.clone()))),
            TokenKind::CharLiteral(c) => Some(Expression::Literal(Literal::Char(*c))),
            TokenKind::True => Some(Expression::Literal(Literal::Bool(true))),
            TokenKind::False => Some(Expression::Literal(Literal::Bool(false))),
            TokenKind::Null => Some(Expression::Literal(Literal::Null)),
            TokenKind::Self_ => Some(Expression::Self_),
            TokenKind::Super => Some(Expression::Super_),
            TokenKind::Identifier(name) => Some(Expression::Identifier(name.clone())),
            TokenKind::New => Some(Expression::Identifier("new".into())),
            TokenKind::Underscore => Some(Expression::Identifier("_".into())),
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen);
                Some(Expression::Paren(Box::new(expr)))
            }
            TokenKind::LBracket => {
                if self.check(&TokenKind::RBracket) {
                    self.advance();
                    Some(Expression::Array(vec![]))
                } else {
                    let mut elements = vec![self.parse_expression()?];
                    while self.consume(&TokenKind::Comma).is_some() {
                        if let Some(e) = self.parse_expression() {
                            elements.push(e);
                        }
                    }
                    self.expect(&TokenKind::RBracket)?;
                    Some(Expression::Array(elements))
                }
            }
            TokenKind::LBrace => {
                self.pos -= 1;
                let block = self.parse_block()?;
                Some(Expression::Block(block))
            }
            TokenKind::If => {
                let expr = self.parse_if_expr()?;
                Some(Expression::If(expr))
            }
            TokenKind::Match => {
                let expr = self.parse_match_expr()?;
                Some(Expression::Match(expr))
            }
            TokenKind::Loop => {
                let expr = self.parse_loop_expr()?;
                Some(Expression::Loop(expr))
            }
            TokenKind::While => {
                let expr = self.parse_while_expr()?;
                Some(Expression::While(expr))
            }
            TokenKind::For => {
                let expr = self.parse_for_expr()?;
                Some(Expression::For(expr))
            }
            TokenKind::Pipe => {
                let params = self.parse_lambda_params();
                let return_type = if self.consume(&TokenKind::Arrow).is_some() {
                    self.parse_type_ref()
                } else {
                    None
                };
                let body = self.parse_block().unwrap_or_default();
                Some(Expression::Lambda {
                    params,
                    return_type,
                    body,
                })
            }
            _ => {
                self.errors.push(ParseError::new(
                    format!("expected expression, found {}", tok.kind),
                    &tok,
                ));
                None
            }
        }
    }

    fn parse_lambda_params(&mut self) -> Vec<Param> {
        if self.check(&TokenKind::Pipe) {
            return vec![];
        }
        let mut params = vec![self.parse_param()];
        while self.consume(&TokenKind::Comma).is_some() {
            params.push(self.parse_param());
        }
        self.expect(&TokenKind::Pipe);
        params
    }

    // ========================================================================
    // Control flow
    // ========================================================================

    pub(crate) fn parse_if_expr(&mut self) -> Option<IfExpr> {
        let condition = Box::new(self.parse_expression()?);
        let then_block = self.parse_block().unwrap_or_default();
        let else_branch = if self.consume(&TokenKind::Else).is_some() {
            if self.consume(&TokenKind::If).is_some() {
                let inner = self.parse_if_expr()?;
                Some(Box::new(ElseBranch::If(inner)))
            } else {
                let block = self.parse_block().unwrap_or_default();
                Some(Box::new(ElseBranch::Block(block)))
            }
        } else {
            None
        };
        Some(IfExpr {
            condition,
            then_block,
            else_branch,
        })
    }

    pub(crate) fn parse_match_expr(&mut self) -> Option<MatchExpr> {
        let value = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
            if let Some(arm) = self.parse_match_arm() {
                arms.push(arm);
            }
            self.consume(&TokenKind::Comma);
        }
        self.expect(&TokenKind::RBrace);
        Some(MatchExpr { value, arms })
    }

    fn parse_match_arm(&mut self) -> Option<MatchArm> {
        let pattern = self.parse_pattern()?;
        let guard = if self.consume(&TokenKind::If).is_some() {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect(&TokenKind::FatArrow)?;
        let value = Box::new(self.parse_expression()?);
        Some(MatchArm {
            pattern,
            guard,
            value,
        })
    }

    pub(crate) fn parse_loop_expr(&mut self) -> Option<LoopExpr> {
        let body = self.parse_block().unwrap_or_default();
        Some(LoopExpr { body })
    }

    pub(crate) fn parse_while_expr(&mut self) -> Option<WhileExpr> {
        let condition = Box::new(self.parse_expression()?);
        let body = self.parse_block().unwrap_or_default();
        Some(WhileExpr { condition, body })
    }

    pub(crate) fn parse_for_expr(&mut self) -> Option<ForExpr> {
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::In)?;
        let iterable = Box::new(self.parse_expression()?);
        let body = self.parse_block().unwrap_or_default();
        Some(ForExpr {
            pattern,
            iterable,
            body,
        })
    }
}
