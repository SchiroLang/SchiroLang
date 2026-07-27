use schiro_ast::*;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<usize>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub mutable: bool,
    pub defined_at: (usize, usize),
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable(Ty),
    Parameter(Ty),
    Function(FnDecl),
    Constructor(ConstructorDecl),
    Type(Ty),
    Trait(TraitDecl),
    Module(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Named(String),
    Generic(String),
    Function(Vec<Ty>, Box<Ty>),
    Ref(Box<Ty>),
    Mut(Box<Ty>),
    Array(Box<Ty>),
    Tuple(Vec<Ty>),
    Self_,
    Class { name: String, args: Vec<Ty> },
    Unknown,
    Error,
}

impl Ty {
    pub fn from_type_ref(tref: &TypeRef, generics: &[String]) -> Self {
        match tref {
            TypeRef::Named { name, args: _args } => {
                if generics.contains(name) {
                    return Ty::Generic(name.clone());
                }
                Ty::Named(name.clone())
            }
            TypeRef::Ref(inner) => Ty::Ref(Box::new(Ty::from_type_ref(inner, generics))),
            TypeRef::Mut(inner) => Ty::Mut(Box::new(Ty::from_type_ref(inner, generics))),
            TypeRef::Array(inner) => Ty::Array(Box::new(Ty::from_type_ref(inner, generics))),
            TypeRef::Tuple(ts) => {
                Ty::Tuple(ts.iter().map(|t| Ty::from_type_ref(t, generics)).collect())
            }
            TypeRef::Function { param_types, return_type } => {
                let params: Vec<Ty> = param_types
                    .iter()
                    .map(|t| Ty::from_type_ref(t, generics))
                    .collect();
                Ty::Function(params, Box::new(Ty::from_type_ref(return_type, generics)))
            }
            TypeRef::Self_ => Ty::Self_,
        }
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { scopes: vec![Scope { parent: None, symbols: Vec::new() }] }
    }

    pub fn enter_scope(&mut self) -> usize {
        let id = self.scopes.len();
        self.scopes.push(Scope { parent: Some(self.current_scope()), symbols: Vec::new() });
        id
    }

    pub fn leave_scope(&mut self) {
        let id = self.current_scope();
        // parent reference kept for lookup chain
        let _ = self.scopes[id].parent;
    }

    pub fn current_scope(&self) -> usize {
        self.scopes.len() - 1
    }

    pub fn define(&mut self, name: String, kind: SymbolKind, mutable: bool, line: usize, column: usize) {
        let id = self.current_scope();
        self.scopes[id].symbols.push(Symbol {
            name,
            kind,
            mutable,
            defined_at: (line, column),
        });
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut id = self.current_scope();
        loop {
            for sym in &self.scopes[id].symbols {
                if sym.name == name {
                    return Some(sym);
                }
            }
            match &self.scopes[id].parent {
                Some(p) => id = *p,
                None => return None,
            }
        }
    }

    pub fn lookup_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        let id = self.current_scope();
        self.scopes[id].symbols.iter().find(|s| s.name == name)
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Named(n) => write!(f, "{n}"),
            Ty::Generic(n) => write!(f, "{n}"),
            Ty::Function(params, ret) => {
                let p: Vec<String> = params.iter().map(|t| format!("{t}")).collect();
                write!(f, "({}) -> {ret}", p.join(", "))
            }
            Ty::Ref(inner) => write!(f, "&{inner}"),
            Ty::Mut(inner) => write!(f, "mut {inner}"),
            Ty::Array(inner) => write!(f, "[{inner}]"),
            Ty::Tuple(ts) => {
                let p: Vec<String> = ts.iter().map(|t| format!("{t}")).collect();
                write!(f, "({})", p.join(", "))
            }
            Ty::Self_ => write!(f, "Self"),
            Ty::Class { name, args } => {
                if args.is_empty() {
                    write!(f, "{name}")
                } else {
                    let a: Vec<String> = args.iter().map(|t| format!("{t}")).collect();
                    write!(f, "{name}<{}>", a.join(", "))
                }
            }
            Ty::Unknown => write!(f, "?"),
            Ty::Error => write!(f, "!!ERROR!!"),
        }
    }
}
