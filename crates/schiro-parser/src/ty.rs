use schiro_ast::*;
use schiro_lexer::token::TokenKind;

use crate::parser::ParseError;
use crate::parser::Parser;

impl Parser {
    // ========================================================================
    // Type references (Pratt-style)
    // ========================================================================

    pub(crate) fn parse_type_ref(&mut self) -> Option<TypeRef> {
        let left = self.parse_base_type_ref()?;
        if self.consume(&TokenKind::Arrow).is_some() {
            let right = self.parse_type_ref()?;
            let param_types = match left {
                TypeRef::Tuple(ts) => ts,
                other => vec![other],
            };
            Some(TypeRef::Function {
                param_types,
                return_type: Box::new(right),
            })
        } else {
            Some(left)
        }
    }

    fn parse_base_type_ref(&mut self) -> Option<TypeRef> {
        let tok = self.advance()?;
        match &tok.kind {
            TokenKind::Amp => {
                let inner = self.parse_type_ref()?;
                Some(TypeRef::Ref(Box::new(inner)))
            }
            TokenKind::Mut => {
                let inner = self.parse_type_ref()?;
                Some(TypeRef::Mut(Box::new(inner)))
            }
            TokenKind::LBracket => {
                let inner = self.parse_type_ref()?;
                self.expect(&TokenKind::RBracket);
                Some(TypeRef::Array(Box::new(inner)))
            }
            TokenKind::LParen => {
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    Some(TypeRef::Tuple(vec![]))
                } else {
                    let first = self.parse_type_ref()?;
                    if self.consume(&TokenKind::Comma).is_some() {
                        let mut types = vec![first];
                        types.push(self.parse_type_ref()?);
                        while self.consume(&TokenKind::Comma).is_some() {
                            types.push(self.parse_type_ref()?);
                        }
                        self.expect(&TokenKind::RParen);
                        Some(TypeRef::Tuple(types))
                    } else {
                        self.expect(&TokenKind::RParen);
                        Some(first)
                    }
                }
            }
            TokenKind::SelfType => Some(TypeRef::Self_),
            TokenKind::Identifier(name) => {
                let args = if self.consume(&TokenKind::Less).is_some() {
                    let mut list = vec![self.parse_type_ref()?];
                    while self.consume(&TokenKind::Comma).is_some() {
                        list.push(self.parse_type_ref()?);
                    }
                    self.expect(&TokenKind::Greater);
                    list
                } else {
                    vec![]
                };
                Some(TypeRef::Named {
                    name: name.clone(),
                    args,
                })
            }
            _ => {
                self.errors.push(ParseError::new(
                    format!("expected type, found {}", tok.kind),
                    &tok,
                ));
                None
            }
        }
    }

    pub(crate) fn parse_trait_ref(&mut self) -> TraitRef {
        let name = self.expect_ident().unwrap_or_default();
        let args = if self.consume(&TokenKind::Less).is_some() {
            let mut list = vec![];
            if let Some(first) = self.parse_type_ref() {
                list.push(first);
                while self.consume(&TokenKind::Comma).is_some() {
                    if let Some(t) = self.parse_type_ref() {
                        list.push(t);
                    }
                }
            }
            self.expect(&TokenKind::Greater);
            list
        } else {
            vec![]
        };
        TraitRef { name, args }
    }

    pub(crate) fn parse_trait_ref_list(&mut self) -> Vec<TraitRef> {
        let mut list = vec![self.parse_trait_ref()];
        while self.consume(&TokenKind::Comma).is_some() {
            list.push(self.parse_trait_ref());
        }
        list
    }

    // ========================================================================
    // Fields
    // ========================================================================

    pub(crate) fn parse_field_list(&mut self) -> Option<Vec<Field>> {
        if self.check(&TokenKind::RParen)
            || self.check(&TokenKind::RBrace)
        {
            return Some(vec![]);
        }
        let mut fields = vec![self.parse_field()];
        while self.consume(&TokenKind::Comma).is_some() {
            fields.push(self.parse_field());
        }
        Some(fields)
    }

    pub(crate) fn parse_field(&mut self) -> Field {
        let mutable = self.consume(&TokenKind::Mut).is_some();

        if self.check_ident() {
            let ahead = self.tokens.get(self.pos + 1).map(|t| &t.kind);
            if ahead == Some(&TokenKind::Colon) {
                let name = self.expect_ident().unwrap_or_default();
                self.advance();
                let type_ = self.parse_type_ref().unwrap_or(TypeRef::Named {
                    name: "???".into(),
                    args: vec![],
                });
                let default = if self.consume(&TokenKind::Equals).is_some() {
                    self.parse_expression()
                } else {
                    None
                };
                return Field {
                    mutable,
                    name,
                    type_,
                    default,
                };
            }
        }

        let type_ = self.parse_type_ref().unwrap_or(TypeRef::Named {
            name: "???".into(),
            args: vec![],
        });
        Field {
            mutable,
            name: String::new(),
            type_,
            default: None,
        }
    }
}
