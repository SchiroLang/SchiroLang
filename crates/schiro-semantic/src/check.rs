use schiro_ast::*;

use crate::error::{ErrorKind, SemanticError};
use crate::scope::{SymbolTable, Ty};

pub struct TypeChecker {
    pub errors: Vec<SemanticError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn check(&mut self, cu: &CompilationUnit, symbols: &SymbolTable) {
        for decl in &cu.declarations {
            if let TopLevelDecl::Fn(fd) = decl {
                self.check_fn_decl(fd, symbols);
            }
        }
    }

    fn check_fn_decl(&mut self, fd: &FnDecl, symbols: &SymbolTable) {
        let generics: Vec<String> = fd.params.params.iter().map(|p| p.name.clone()).collect();

        if let BlockOrSemi::Block(body) = &fd.body {
            for stmt in body {
                self.check_statement(stmt, symbols, &generics);
            }
        }
    }

    fn check_statement(&mut self, stmt: &Statement, symbols: &SymbolTable, generics: &[String]) {
        match stmt {
            Statement::Let(let_decl) => {
                if let Some(type_) = &let_decl.type_ {
                    let _ = Ty::from_type_ref(type_, generics);
                }
            }
            Statement::Assignment(assign) => {
                if let LValue::Variable(name) = &assign.lvalue {
                    if let Some(sym) = symbols.lookup(name) {
                        if !sym.mutable {
                            self.errors.push(SemanticError {
                                kind: ErrorKind::CannotMutate(name.clone()),
                                span: None,
                                line: 0,
                                column: 0,
                            });
                        }
                    }
                }
            }
            Statement::If(if_expr) => self.check_if_expr(if_expr, symbols, generics),
            Statement::Match(match_expr) => self.check_match_expr(match_expr, symbols, generics),
            Statement::Loop(loop_expr) => {
                for s in &loop_expr.body {
                    self.check_statement(s, symbols, generics);
                }
            }
            Statement::While(while_expr) => {
                for s in &while_expr.body {
                    self.check_statement(s, symbols, generics);
                }
            }
            Statement::For(for_expr) => {
                for s in &for_expr.body {
                    self.check_statement(s, symbols, generics);
                }
            }
            Statement::Expression(_)
            | Statement::Return(_)
            | Statement::Break(_)
            | Statement::Continue
            | Statement::SuperCall(_) => {}
            Statement::Block(block) => {
                for s in block {
                    self.check_statement(s, symbols, generics);
                }
            }
        }
    }

    fn check_if_expr(&mut self, if_expr: &IfExpr, symbols: &SymbolTable, generics: &[String]) {
        for s in &if_expr.then_block {
            self.check_statement(s, symbols, generics);
        }
        if let Some(else_branch) = &if_expr.else_branch {
            match else_branch.as_ref() {
                ElseBranch::If(inner) => self.check_if_expr(inner, symbols, generics),
                ElseBranch::Block(b) => {
                    for s in b {
                        self.check_statement(s, symbols, generics);
                    }
                }
            }
        }
    }

    fn check_match_expr(&mut self, match_expr: &MatchExpr, _symbols: &SymbolTable, _generics: &[String]) {
        for arm in &match_expr.arms {
            if let Some(guard) = &arm.guard {
                let _ = guard;
            }
        }
    }
}
