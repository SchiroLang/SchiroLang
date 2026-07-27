use schiro_ast::*;

use crate::error::{ErrorKind, SemanticError};
use crate::scope::{SymbolKind, SymbolTable, Ty};

pub struct Resolver {
    pub symbols: SymbolTable,
    pub errors: Vec<SemanticError>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    pub fn resolve(&mut self, cu: &CompilationUnit) {
        // Phase 1: collect all top-level declarations
        for decl in &cu.declarations {
            match decl {
                TopLevelDecl::TypeDef(td) => {
                    let ty = Ty::Named(td.name.clone());
                    self.symbols.define(
                        td.name.clone(),
                        SymbolKind::Type(ty),
                        false,
                        0, 0,
                    );
                }
                TopLevelDecl::Class(cd) => {
                    let generics: Vec<String> = cd.params.params.iter().map(|p| p.name.clone()).collect();
                    let ty = Ty::from_type_ref(
                        &TypeRef::Named {
                            name: cd.name.clone(),
                            args: generics.iter().map(|g| TypeRef::Named {
                                name: g.clone(),
                                args: vec![],
                            }).collect(),
                        },
                        &[],
                    );
                    self.symbols.define(
                        cd.name.clone(),
                        SymbolKind::Type(ty),
                        false,
                        0, 0,
                    );
                }
                TopLevelDecl::Trait(td) => {
                    self.symbols.define(
                        td.name.clone(),
                        SymbolKind::Trait(td.clone()),
                        false,
                        0, 0,
                    );
                }
                TopLevelDecl::Fn(fd) => {
                    self.symbols.define(
                        fd.name.clone(),
                        SymbolKind::Function(fd.clone()),
                        false,
                        0, 0,
                    );
                }
                TopLevelDecl::Impl(_imp) => {
                    // recorded later during type checking
                }
                TopLevelDecl::Static(sd) => {
                    let generics: Vec<String> = vec![];
                    let ty = Ty::from_type_ref(&sd.type_, &generics);
                    self.symbols.define(
                        sd.name.clone(),
                        SymbolKind::Variable(ty),
                        false,
                        0, 0,
                    );
                }
            }
        }

        // Phase 2: resolve function bodies
        for decl in &cu.declarations {
            if let TopLevelDecl::Fn(fd) = decl {
                self.resolve_fn_decl(fd);
            }
        }
    }

    fn resolve_fn_decl(&mut self, fd: &FnDecl) {
        let generics: Vec<String> = fd.params.params.iter().map(|p| p.name.clone()).collect();

        self.symbols.enter_scope();
        for param in &fd.parameters {
            let ty = Ty::from_type_ref(&param.type_, &generics);
            let kind = if param.name == "self" {
                SymbolKind::Variable(ty.clone())
            } else {
                SymbolKind::Parameter(ty.clone())
            };
            self.symbols.define(
                param.name.clone(),
                kind,
                param.mutable,
                0, 0,
            );
        }

        if let BlockOrSemi::Block(body) = &fd.body {
            self.resolve_block(body, &generics);
        }

        self.symbols.leave_scope();
    }

    fn resolve_block(&mut self, stmts: &[Statement], generics: &[String]) {
        self.symbols.enter_scope();
        for stmt in stmts {
            self.resolve_statement(stmt, generics);
        }
        self.symbols.leave_scope();
    }

    fn resolve_statement(&mut self, stmt: &Statement, generics: &[String]) {
        match stmt {
            Statement::Let(let_decl) => {
                let ty = let_decl.type_.as_ref().map(|t| Ty::from_type_ref(t, generics))
                    .unwrap_or(Ty::Unknown);
                self.symbols.define(
                    pattern_name(&let_decl.pattern).unwrap_or("_".into()),
                    SymbolKind::Variable(ty),
                    false,
                    0, 0,
                );
            }
            Statement::Assignment(assign) => {
                self.resolve_lvalue(&assign.lvalue, generics);
            }
            Statement::If(if_expr) => {
                self.resolve_if_expr(if_expr, generics);
            }
            Statement::Match(match_expr) => {
                self.resolve_match_expr(match_expr, generics);
            }
            Statement::Loop(loop_expr) => {
                self.resolve_block(&loop_expr.body, generics);
            }
            Statement::While(while_expr) => {
                self.resolve_block(&while_expr.body, generics);
            }
            Statement::For(for_expr) => {
                let pname = pattern_name(&for_expr.pattern).unwrap_or("_".into());
                self.symbols.define(pname, SymbolKind::Variable(Ty::Unknown), false, 0, 0);
                self.resolve_block(&for_expr.body, generics);
            }
            Statement::Expression(_)
            | Statement::Return(_)
            | Statement::Break(_)
            | Statement::Continue
            | Statement::SuperCall(_) => {}
            Statement::Block(block) => {
                self.resolve_block(block, generics);
            }
        }
    }

    fn resolve_if_expr(&mut self, if_expr: &IfExpr, generics: &[String]) {
        self.resolve_block(&if_expr.then_block, generics);
        if let Some(else_branch) = &if_expr.else_branch {
            match else_branch.as_ref() {
                ElseBranch::If(inner) => self.resolve_if_expr(inner, generics),
                ElseBranch::Block(b) => self.resolve_block(b, generics),
            }
        }
    }

    fn resolve_match_expr(&mut self, match_expr: &MatchExpr, generics: &[String]) {
        for arm in &match_expr.arms {
            self.symbols.enter_scope();
            bind_pattern(&arm.pattern, generics, self);
            if let Some(guard) = &arm.guard {
                let _ = guard;
            }
            self.resolve_block(&[], generics);
            self.symbols.leave_scope();
        }
    }

    fn resolve_lvalue(&mut self, lvalue: &LValue, generics: &[String]) {
        match lvalue {
            LValue::Variable(name) => {
                if self.symbols.lookup(name).is_none() {
                    self.errors.push(SemanticError {
                        kind: ErrorKind::UndefinedVariable(name.clone()),
                        span: None,
                        line: 0,
                        column: 0,
                    });
                }
            }
            LValue::Field(lv, _field) => self.resolve_lvalue(lv, generics),
            LValue::Index(lv, _index) => self.resolve_lvalue(lv, generics),
        }
    }
}

fn pattern_name(p: &Pattern) -> Option<String> {
    match p {
        Pattern::Identifier(name) => Some(name.clone()),
        Pattern::Wildcard => None,
        Pattern::Literal(_) => None,
        Pattern::DestructureVariant { patterns, .. } => {
            patterns.first().and_then(pattern_name)
        }
        Pattern::DestructureTuple(patterns) => {
            patterns.first().and_then(pattern_name)
        }
        Pattern::Or(left, _) => pattern_name(left),
    }
}

fn bind_pattern(p: &Pattern, generics: &[String], resolver: &mut Resolver) {
    match p {
        Pattern::Identifier(name) => {
            resolver.symbols.define(
                name.clone(),
                SymbolKind::Variable(Ty::Unknown),
                false,
                0, 0,
            );
        }
        Pattern::Wildcard => {}
        Pattern::Literal(_) => {}
        Pattern::DestructureVariant { patterns, .. } => {
            for pat in patterns {
                bind_pattern(pat, generics, resolver);
            }
        }
        Pattern::DestructureTuple(patterns) => {
            for pat in patterns {
                bind_pattern(pat, generics, resolver);
            }
        }
        Pattern::Or(left, right) => {
            bind_pattern(left, generics, resolver);
            bind_pattern(right, generics, resolver);
        }
    }
}
