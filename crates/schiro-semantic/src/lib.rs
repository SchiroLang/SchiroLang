mod error;
mod scope;
mod resolve;
mod r#check;

use schiro_ast::CompilationUnit;

pub use error::{ErrorKind, SemanticError};
pub use scope::{Scope, Symbol, SymbolKind, SymbolTable, Ty};

pub struct SemanticAnalysis {
    pub resolver: resolve::Resolver,
    pub checker: r#check::TypeChecker,
}

impl SemanticAnalysis {
    pub fn new() -> Self {
        Self {
            resolver: resolve::Resolver::new(),
            checker: r#check::TypeChecker::new(),
        }
    }

    pub fn analyze(&mut self, cu: &CompilationUnit) {
        self.resolver.resolve(cu);
        self.checker.check(cu, &self.resolver.symbols);
    }

    pub fn errors(&self) -> Vec<SemanticError> {
        let mut all = self.resolver.errors.clone();
        all.extend(self.checker.errors.clone());
        all
    }
}
