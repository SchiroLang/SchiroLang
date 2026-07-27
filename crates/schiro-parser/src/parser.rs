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
    fn new(msg: impl Into<String>, t: &Token) -> Self {
        Self {
            message: msg.into(),
            span: t.span,
            line: t.line,
            column: t.column,
        }
    }
}

// ============================================================================
// Parser
// ============================================================================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(t)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn consume(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            self.advance()
        } else {
            None
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            self.advance()
        } else {
            let found = self.peek().cloned().unwrap_or_else(|| {
                Token::new(TokenKind::Eof, Span::empty(), 0, 0)
            });
            let msg = format!("expected {kind}, found {kind_fmt}", kind_fmt = found.kind);
            let err = ParseError::new(msg, &found);
            self.errors.push(err);
            None
        }
    }

    fn check_ident(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Identifier(_)))
    }

    fn expect_ident(&mut self) -> Option<String> {
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
        self.advance()?; // consume 'import'
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
    // Top-level declarations
    // ========================================================================

    fn parse_top_level_decl(&mut self) -> Option<TopLevelDecl> {
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
    // Type declarations (type aliases / sum types)
    // ========================================================================

    fn parse_type_decl(&mut self) -> Option<TypeDef> {
        self.advance()?; // 'type'
        let name = self.expect_ident()?;
        let params = self.parse_type_params();
        self.expect(&TokenKind::Equals)?;
        let sum_type = self.parse_sum_type()?;
        self.expect(&TokenKind::Semicolon);
        Some(TypeDef {
            name,
            params,
            sum_type,
        })
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
        Some(Variant {
            name,
            fields,
            trait_impls,
        })
    }

    // ========================================================================
    // Type parameters & constraints
    // ========================================================================

    fn parse_type_params(&mut self) -> TypeParams {
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
    // Type references (Pratt-style)
    // ========================================================================

    fn parse_type_ref(&mut self) -> Option<TypeRef> {
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

    fn parse_trait_ref(&mut self) -> TraitRef {
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

    fn parse_trait_ref_list(&mut self) -> Vec<TraitRef> {
        let mut list = vec![self.parse_trait_ref()];
        while self.consume(&TokenKind::Comma).is_some() {
            list.push(self.parse_trait_ref());
        }
        list
    }

    // ========================================================================
    // Fields
    // ========================================================================

    fn parse_field_list(&mut self) -> Option<Vec<Field>> {
        if self.check(&TokenKind::RParen) || self.check(&TokenKind::RBrace) || self.check(&TokenKind::RParen) {
            return Some(vec![]);
        }
        let mut fields = vec![self.parse_field()];
        while self.consume(&TokenKind::Comma).is_some() {
            fields.push(self.parse_field());
        }
        Some(fields)
    }

    fn parse_field(&mut self) -> Field {
        let mutable = self.consume(&TokenKind::Mut).is_some();

        // peek ahead: if identifier followed by ':', it's a named field
        if self.check_ident() {
            let ahead = self.tokens.get(self.pos + 1).map(|t| &t.kind);
            if ahead == Some(&TokenKind::Colon) {
                let name = self.expect_ident().unwrap_or_default();
                self.advance(); // ':'
                let type_ = self.parse_type_ref().unwrap_or(TypeRef::Named {
                    name: "???".into(),
                    args: vec![],
                });
                let default = if self.consume(&TokenKind::Equals).is_some() {
                    self.parse_expression()
                } else {
                    None
                };
                return Field { mutable, name, type_, default };
            }
        }

        // positional field (just type, no name)
        let type_ = self.parse_type_ref().unwrap_or(TypeRef::Named {
            name: "???".into(),
            args: vec![],
        });
        Field { mutable, name: String::new(), type_, default: None }
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
            self.advance(); // 'fn'
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
        // field decl
        let decl = self.parse_field_decl()?;
        Some(ClassMember {
            visibility,
            kind: ClassMemberKind::Field(decl),
        })
    }

    fn peek_ahead_is_new(&self) -> bool {
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

    fn parse_fn_decl(&mut self) -> Option<FnDecl> {
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

    fn parse_param_list(&mut self) -> Vec<Param> {
        if self.check(&TokenKind::RParen) {
            return vec![];
        }
        let mut list = vec![self.parse_param()];
        while self.consume(&TokenKind::Comma).is_some() {
            list.push(self.parse_param());
        }
        list
    }

    fn parse_param(&mut self) -> Param {
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
    // Block
    // ========================================================================

    fn parse_block(&mut self) -> Option<Block> {
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

    // ========================================================================
    // Traits
    // ========================================================================

    fn parse_trait_decl(&mut self) -> Option<TraitDecl> {
        self.advance()?; // 'trait'
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
        Some(TraitDecl {
            name,
            params,
            members,
        })
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
        self.advance()?; // 'fn'
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
        self.advance()?; // 'prop'
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let type_ = self.parse_type_ref()?;
        // optional semicolon
        self.consume(&TokenKind::Semicolon);
        Some(PropSignature { name, type_ })
    }

    // ========================================================================
    // Impl blocks
    // ========================================================================

    fn parse_impl_block(&mut self) -> Option<ImplBlock> {
        self.advance()?; // 'impl'
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
            Some(ImplBlock::Inherent {
                type_,
                members,
            })
        }
    }

    // ========================================================================
    // Properties
    // ========================================================================

    fn parse_prop_decl(&mut self) -> Option<PropDecl> {
        self.advance()?; // 'prop'
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
                    if self.check(&TokenKind::LBrace) || (self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RBrace)) {
                        // get with optional block
                        if self.check(&TokenKind::LBrace) {
                            get = self.parse_block();
                        } else {
                            get = Some(vec![]);
                        }
                    } else {
                        get = Some(vec![]);
                    }
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
        self.advance()?; // 'let'
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let type_ = self.parse_type_ref()?;
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon);
        Some(StaticDecl {
            name,
            type_,
            value,
        })
    }

    // ========================================================================
    // Statements
    // ========================================================================

    fn parse_statement(&mut self) -> Option<Statement> {
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
                // Could be SuperCall or expression
                if self.peek_ahead_is_lparen() {
                    self.parse_super_call().map(Statement::SuperCall)
                } else {
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon);
                    Some(Statement::Expression(expr))
                }
            }
            _ => {
                // Try to parse as assignment or expression
                if let Some(stmt) = self.try_parse_assignment() {
                    return Some(stmt);
                }
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::Semicolon);
                Some(Statement::Expression(expr))
            }
        }
    }

    fn peek_ahead_is_lparen(&self) -> bool {
        self.tokens.get(self.pos + 1)
            .map_or(false, |t| t.kind == TokenKind::LParen)
    }

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

    fn parse_let_decl(&mut self) -> Option<LetDecl> {
        self.advance()?; // 'let'
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

    fn parse_return_stmt(&mut self) -> Option<Option<Expression>> {
        self.advance()?; // 'return'
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
        self.advance()?; // 'break'
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
        self.advance()?; // 'super'
        self.expect(&TokenKind::LParen)?;
        let args = self.parse_arg_list();
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Semicolon);
        Some(args)
    }

    // ========================================================================
    // Expressions (Pratt parser)
    // ========================================================================

    fn parse_expression(&mut self) -> Option<Expression> {
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
            // comparison
            TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::Less
            | TokenKind::Greater
            | TokenKind::LessEq
            | TokenKind::GreaterEq => (3, Fixity::Left),
            // pipe
            TokenKind::PipeGreater => (4, Fixity::Left),
            // range
            TokenKind::DotDot => (5, Fixity::Left),
            // additive
            TokenKind::Plus | TokenKind::Minus => (6, Fixity::Left),
            // multiplicative
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (7, Fixity::Left),
            // suffix
            TokenKind::Dot => (9, Fixity::Suffix),
            TokenKind::LParen => (9, Fixity::Suffix),
            TokenKind::LBracket => (9, Fixity::Suffix),
            TokenKind::Question => (9, Fixity::Suffix),
            TokenKind::Bang => (9, Fixity::Suffix),
            // logical — these are handled in prefix, not as infix
            // actually they ARE infix:
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

    fn parse_arg_list(&mut self) -> Vec<Expression> {
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

    fn parse_prefix_expr(&mut self) -> Option<Expression> {
        let tok = self.advance()?;
        match &tok.kind {
            // unary operators
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
            // primary
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
            // paren
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen);
                Some(Expression::Paren(Box::new(expr)))
            }
            // array literal
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
            // block expression
            TokenKind::LBrace => {
                self.pos -= 1;
                let block = self.parse_block()?;
                Some(Expression::Block(block))
            }
            // if expression
            TokenKind::If => {
                let expr = self.parse_if_expr()?;
                Some(Expression::If(expr))
            }
            // match expression
            TokenKind::Match => {
                let expr = self.parse_match_expr()?;
                Some(Expression::Match(expr))
            }
            // loop expression
            TokenKind::Loop => {
                let expr = self.parse_loop_expr()?;
                Some(Expression::Loop(expr))
            }
            // while expression
            TokenKind::While => {
                let expr = self.parse_while_expr()?;
                Some(Expression::While(expr))
            }
            // for expression
            TokenKind::For => {
                let expr = self.parse_for_expr()?;
                Some(Expression::For(expr))
            }
            // lambda: |params| [-> ret] block
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
            // Empty params: ||
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

    fn parse_if_expr(&mut self) -> Option<IfExpr> {
        // 'if' already consumed by prefix
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

    fn parse_match_expr(&mut self) -> Option<MatchExpr> {
        // 'match' already consumed by prefix
        let value = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && self.peek().is_some() {
            if let Some(arm) = self.parse_match_arm() {
                arms.push(arm);
            }
            // skip optional comma
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

    fn parse_loop_expr(&mut self) -> Option<LoopExpr> {
        // 'loop' already consumed
        let body = self.parse_block().unwrap_or_default();
        Some(LoopExpr { body })
    }

    fn parse_while_expr(&mut self) -> Option<WhileExpr> {
        // 'while' already consumed
        let condition = Box::new(self.parse_expression()?);
        let body = self.parse_block().unwrap_or_default();
        Some(WhileExpr { condition, body })
    }

    fn parse_for_expr(&mut self) -> Option<ForExpr> {
        // 'for' already consumed
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

    // ========================================================================
    // Patterns
    // ========================================================================

    fn parse_pattern(&mut self) -> Option<Pattern> {
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
                // Could be a variable pattern or a variant destructure
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
                // tuple destructure: [a, b, c]
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

#[derive(Clone, Copy)]
enum Fixity {
    Left,
    Suffix,
}
