use schiro_ast::*;
use schiro_lexer::token::TokenKind;

use crate::parser::ParseError;
use crate::parser::Parser;

impl Parser {
    // ========================================================================
    // Top-level declarations
    // ========================================================================

    pub(crate) fn parse_top_level_decl(&mut self) -> Option<TopLevelDecl> {
        match self.peek_kind()? {
            TokenKind::Type => {
                let decl = self.parse_type_decl()?;
                Some(TopLevelDecl::TypeDef(decl))
            }
            TokenKind::Abstract | TokenKind::Class => {
                let decl = self.parse_class_decl()?;
                Some(TopLevelDecl::Class(decl))
            }
            TokenKind::Static | TokenKind::Fn => {
                let decl = self.parse_fn_decl()?;
                Some(TopLevelDecl::Fn(decl))
            }
            TokenKind::Trait => {
                let decl = self.parse_trait_decl()?;
                Some(TopLevelDecl::Trait(decl))
            }
            TokenKind::Impl => {
                let decl = self.parse_impl_block()?;
                Some(TopLevelDecl::Impl(decl))
            }
            TokenKind::Let => {
                let decl = self.parse_static_decl()?;
                Some(TopLevelDecl::Static(decl))
            }
            _ => {
                let t = self.advance();
                if let Some(tok) = t {
                    self.errors.push(ParseError::new(
                        format!("unexpected token in top-level decl: {}", tok.kind),
                        &tok,
                    ));
                }
                None
            }
        }
    }

    // ========================================================================
    // Type declarations (sum types)
    // ========================================================================

    fn parse_type_decl(&mut self) -> Option<TypeDef> {
        self.advance()?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params();
        self.expect(&TokenKind::Equals)?;
        let sum_type = self.parse_sum_type()?;
        self.expect(&TokenKind::Semicolon);
        Some(TypeDef { name, params, sum_type })
    }

    fn parse_sum_type(&mut self) -> Option<SumType> {
        let mut variants = vec![self.parse_variant()?];
        while self.consume(&TokenKind::Pipe).is_some() {
            variants.push(self.parse_variant()?);
        }
        Some(SumType { variants })
    }

    fn parse_variant(&mut self) -> Option<Variant> {
        let name = self.expect_ident()?;
        let fields = if self.consume(&TokenKind::LParen).is_some() {
            let f = self.parse_field_list();
            self.expect(&TokenKind::RParen);
            f
        } else {
            None
        };
        let trait_impls = if self.consume(&TokenKind::Impl).is_some() {
            Some(self.parse_trait_ref_list())
        } else {
            None
        };
        Some(Variant { name, fields, trait_impls })
    }

    // ========================================================================
    // Type parameters & constraints
    // ========================================================================

    pub(crate) fn parse_type_params(&mut self) -> TypeParams {
        if self.consume(&TokenKind::Less).is_some() {
            let mut params = vec![self.parse_type_param()];
            while self.consume(&TokenKind::Comma).is_some() {
                params.push(self.parse_type_param());
            }
            self.expect(&TokenKind::Greater);
            TypeParams { params }
        } else {
            TypeParams { params: vec![] }
        }
    }

    fn parse_type_param(&mut self) -> TypeParam {
        let name = self.expect_ident().unwrap_or_default();
        let constraints = if self.consume(&TokenKind::Colon).is_some() {
            Some(self.parse_constraint_list())
        } else {
            None
        };
        TypeParam { name, constraints }
    }

    fn parse_constraint_list(&mut self) -> Vec<TraitRef> {
        let mut list = vec![self.parse_trait_ref()];
        while self.consume(&TokenKind::Plus).is_some() {
            list.push(self.parse_trait_ref());
        }
        list
    }

    // ========================================================================
    // Class declarations
    // ========================================================================

    fn parse_class_decl(&mut self) -> Option<ClassDecl> {
        let abstract_ = self.consume(&TokenKind::Abstract).is_some();
        self.expect(&TokenKind::Class)?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params();
        let primary_constructor = if self.consume(&TokenKind::LParen).is_some() {
            let fields = self.parse_field_list();
            self.expect(&TokenKind::RParen);
            fields
        } else {
            None
        };
        let extends = if self.consume(&TokenKind::Extends).is_some() {
            self.parse_type_ref().map(Box::new)
        } else {
            None
        };
        let impls = if self.consume(&TokenKind::Impl).is_some() {
            Some(self.parse_trait_ref_list())
        } else {
            None
        };
        let body = self.parse_class_body();
        Some(ClassDecl {
            abstract_,
            name,
            params,
            primary_constructor,
            extends,
            impls,
            body,
        })
    }

    fn parse_class_body(&mut self) -> ClassBody {
        if self.consume(&TokenKind::Semicolon).is_some() {
            return ClassBody::Semi;
        }
        self.expect(&TokenKind::Colon);
        if self.consume(&TokenKind::LBrace).is_some() {
            let mut members = Vec::new();
            while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
                if let Some(m) = self.parse_class_member() {
                    members.push(m);
                }
            }
            self.expect(&TokenKind::RBrace);
            ClassBody::Brace(members)
        } else {
            let mut members = Vec::new();
            loop {
                if let Some(m) = self.parse_class_member() {
                    members.push(m);
                }
                if self.consume(&TokenKind::Semicolon).is_none() {
                    break;
                }
            }
            ClassBody::Inline(members)
        }
    }

    fn parse_class_member(&mut self) -> Option<ClassMember> {
        let visibility = self.parse_visibility();
        if self.check(&TokenKind::Fn) && self.peek_ahead_is_new() {
            self.advance();
            let ctor = self.parse_constructor_decl()?;
            return Some(ClassMember {
                visibility,
                kind: ClassMemberKind::Constructor(ctor),
            });
        }
        if self.check(&TokenKind::Fn) {
            let decl = self.parse_fn_decl()?;
            return Some(ClassMember {
                visibility,
                kind: ClassMemberKind::Fn(decl),
            });
        }
        if self.check(&TokenKind::Prop) {
            let decl = self.parse_prop_decl()?;
            return Some(ClassMember {
                visibility,
                kind: ClassMemberKind::Prop(decl),
            });
        }
        let decl = self.parse_field_decl()?;
        Some(ClassMember {
            visibility,
            kind: ClassMemberKind::Field(decl),
        })
    }

    fn parse_visibility(&mut self) -> Option<Visibility> {
        match self.peek_kind()? {
            TokenKind::Public => {
                self.advance();
                Some(Visibility::Public)
            }
            TokenKind::Protected => {
                self.advance();
                Some(Visibility::Protected)
            }
            TokenKind::Private => {
                self.advance();
                Some(Visibility::Private)
            }
            _ => None,
        }
    }

    fn parse_field_decl(&mut self) -> Option<FieldDecl> {
        let mutable = self.consume(&TokenKind::Mut).is_some();
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let type_ = self.parse_type_ref()?;
        let default = if self.consume(&TokenKind::Equals).is_some() {
            self.parse_expression()
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon);
        Some(FieldDecl {
            mutable,
            name,
            type_,
            default,
        })
    }

    // ========================================================================
    // Constructor
    // ========================================================================

    fn parse_constructor_decl(&mut self) -> Option<ConstructorDecl> {
        self.expect(&TokenKind::New)?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list();
        self.expect(&TokenKind::RParen)?;
        let delegate = if self.consume(&TokenKind::Colon).is_some() {
            self.parse_expression().map(Box::new)
        } else {
            None
        };
        let body = self.parse_block().unwrap_or_default();
        Some(ConstructorDecl {
            params,
            delegate,
            body,
        })
    }

    // ========================================================================
    // Functions
    // ========================================================================

    pub(crate) fn parse_fn_decl(&mut self) -> Option<FnDecl> {
        let static_ = self.consume(&TokenKind::Static).is_some();
        let modifier = if self.consume(&TokenKind::Virtual).is_some() {
            Some(FnModifier::Virtual)
        } else if self.consume(&TokenKind::Override).is_some() {
            Some(FnModifier::Override)
        } else if self.consume(&TokenKind::Abstract).is_some() {
            Some(FnModifier::Abstract)
        } else {
            None
        };
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params();
        self.expect(&TokenKind::LParen)?;
        let parameters = self.parse_param_list();
        self.expect(&TokenKind::RParen)?;
        let return_type = if self.consume(&TokenKind::Arrow).is_some() {
            self.parse_type_ref()
        } else {
            None
        };
        let body = if self.consume(&TokenKind::Semicolon).is_some() {
            BlockOrSemi::Semi
        } else {
            BlockOrSemi::Block(self.parse_block().unwrap_or_default())
        };
        Some(FnDecl {
            static_,
            modifier,
            name,
            params,
            parameters,
            return_type,
            body,
        })
    }

    pub(crate) fn parse_param_list(&mut self) -> Vec<Param> {
        if self.check(&TokenKind::RParen) {
            return vec![];
        }
        let mut list = vec![self.parse_param()];
        while self.consume(&TokenKind::Comma).is_some() {
            list.push(self.parse_param());
        }
        list
    }

    pub(crate) fn parse_param(&mut self) -> Param {
        let mutable = self.consume(&TokenKind::Mut).is_some();
        if self.check(&TokenKind::Self_) {
            self.advance();
            return Param {
                mutable,
                name: "self".into(),
                type_: TypeRef::Named {
                    name: "Self".into(),
                    args: vec![],
                },
                default: None,
            };
        }
        let name = self.expect_ident().unwrap_or_default();
        self.expect(&TokenKind::Colon);
        let type_ = self.parse_type_ref().unwrap_or(TypeRef::Named {
            name: "???".into(),
            args: vec![],
        });
        let default = if self.consume(&TokenKind::Equals).is_some() {
            self.parse_expression()
        } else {
            None
        };
        Param {
            mutable,
            name,
            type_,
            default,
        }
    }

    // ========================================================================
    // Traits
    // ========================================================================

    fn parse_trait_decl(&mut self) -> Option<TraitDecl> {
        self.advance()?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params();
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::LBrace)?;
        let mut members = Vec::new();
        while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
            if let Some(m) = self.parse_trait_member() {
                members.push(m);
            }
        }
        self.expect(&TokenKind::RBrace);
        Some(TraitDecl { name, params, members })
    }

    fn parse_trait_member(&mut self) -> Option<TraitMember> {
        if self.check(&TokenKind::Fn) {
            Some(TraitMember::Fn(self.parse_fn_signature()?))
        } else if self.check(&TokenKind::Prop) {
            Some(TraitMember::Prop(self.parse_prop_signature()?))
        } else {
            let t = self.advance()?;
            self.errors.push(ParseError::new(
                format!("expected trait member (fn/prop), found {}", t.kind),
                &t,
            ));
            None
        }
    }

    fn parse_fn_signature(&mut self) -> Option<FnSignature> {
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list();
        self.expect(&TokenKind::RParen)?;
        let return_type = if self.consume(&TokenKind::Arrow).is_some() {
            self.parse_type_ref()
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon);
        Some(FnSignature {
            name,
            params,
            return_type,
        })
    }

    fn parse_prop_signature(&mut self) -> Option<PropSignature> {
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let type_ = self.parse_type_ref()?;
        self.consume(&TokenKind::Semicolon);
        Some(PropSignature { name, type_ })
    }

    // ========================================================================
    // Impl blocks
    // ========================================================================

    fn parse_impl_block(&mut self) -> Option<ImplBlock> {
        self.advance()?;
        let first = self.parse_trait_ref();
        if self.consume(&TokenKind::For).is_some() {
            let type_ = self.parse_type_ref()?;
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::LBrace)?;
            let mut members = Vec::new();
            while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
                if let Some(m) = self.parse_class_member() {
                    members.push(m);
                }
            }
            self.expect(&TokenKind::RBrace);
            Some(ImplBlock::TraitImpl {
                trait_: first,
                for_: type_,
                members,
            })
        } else {
            let type_ = TypeRef::Named {
                name: first.name,
                args: first.args,
            };
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::LBrace)?;
            let mut members = Vec::new();
            while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
                if let Some(m) = self.parse_class_member() {
                    members.push(m);
                }
            }
            self.expect(&TokenKind::RBrace);
            Some(ImplBlock::Inherent { type_, members })
        }
    }

    // ========================================================================
    // Properties
    // ========================================================================

    fn parse_prop_decl(&mut self) -> Option<PropDecl> {
        self.advance()?;
        let name = self.expect_ident()?;
        let type_ = if self.consume(&TokenKind::Colon).is_some() {
            self.parse_type_ref()
        } else {
            None
        };
        let accessors = self.parse_prop_accessors()?;
        Some(PropDecl {
            name,
            type_,
            accessors,
        })
    }

    fn parse_prop_accessors(&mut self) -> Option<PropAccessors> {
        if self.consume(&TokenKind::Equals).is_some() {
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon);
            Some(PropAccessors::Expression(expr))
        } else if self.consume(&TokenKind::Colon).is_some() {
            self.expect(&TokenKind::LBrace)?;
            let mut get = None;
            let mut set = None;
            loop {
                if self.consume(&TokenKind::Get).is_some() {
                    get = if self.check(&TokenKind::LBrace) {
                        self.parse_block()
                    } else {
                        Some(vec![])
                    };
                } else if self.consume(&TokenKind::Set).is_some() {
                    self.expect(&TokenKind::LParen);
                    let name = self.expect_ident().unwrap_or_default();
                    self.expect(&TokenKind::RParen);
                    let block = self.parse_block().unwrap_or_default();
                    set = Some((name, block));
                } else if self.check(&TokenKind::RBrace) {
                    break;
                } else {
                    let t = self.advance()?;
                    self.errors.push(ParseError::new(
                        format!("expected get/set in prop accessor, found {}", t.kind),
                        &t,
                    ));
                }
            }
            self.expect(&TokenKind::RBrace);
            Some(PropAccessors::Braces { get, set })
        } else {
            let t = self.peek().cloned().unwrap();
            self.errors.push(ParseError::new(
                "expected '=' or ':' for property accessors",
                &t,
            ));
            None
        }
    }

    // ========================================================================
    // Statics
    // ========================================================================

    fn parse_static_decl(&mut self) -> Option<StaticDecl> {
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let type_ = self.parse_type_ref()?;
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon);
        Some(StaticDecl { name, type_, value })
    }
}
