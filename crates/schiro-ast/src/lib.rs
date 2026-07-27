use std::fmt;

// ============================================================================
// Span
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

// ============================================================================
// Compilation unit
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    pub imports: Vec<ImportDirective>,
    pub declarations: Vec<TopLevelDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDirective {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

// ============================================================================
// Top-level declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDecl {
    TypeDef(TypeDef),
    Class(ClassDecl),
    Fn(FnDecl),
    Trait(TraitDecl),
    Impl(ImplBlock),
    Static(StaticDecl),
}

// ============================================================================
// Type definitions (sum types / algebraic data types)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub params: TypeParams,
    pub sum_type: SumType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SumType {
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Option<FieldList>,
    pub trait_impls: Option<Vec<TraitRef>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParams {
    pub params: Vec<TypeParam>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub constraints: Option<Vec<TraitRef>>,
}

// ============================================================================
// Type references
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Named {
        name: String,
        args: Vec<TypeRef>,
    },
    Ref(Box<TypeRef>),
    Mut(Box<TypeRef>),
    Array(Box<TypeRef>),
    Tuple(Vec<TypeRef>),
    Function {
        param_types: Vec<TypeRef>,
        return_type: Box<TypeRef>,
    },
    Self_,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub name: String,
    pub args: Vec<TypeRef>,
}

// ============================================================================
// Class declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub abstract_: bool,
    pub name: String,
    pub params: TypeParams,
    pub primary_constructor: Option<FieldList>,
    pub extends: Option<Box<TypeRef>>,
    pub impls: Option<Vec<TraitRef>>,
    pub body: ClassBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassBody {
    Brace(Vec<ClassMember>),
    Inline(Vec<ClassMember>),
    Semi,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMember {
    pub visibility: Option<Visibility>,
    pub kind: ClassMemberKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMemberKind {
    Fn(FnDecl),
    Prop(PropDecl),
    Field(FieldDecl),
    Constructor(ConstructorDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub mutable: bool,
    pub name: String,
    pub type_: TypeRef,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub mutable: bool,
    pub name: String,
    pub type_: TypeRef,
    pub default: Option<Expression>,
}

pub type FieldList = Vec<Field>;

// ============================================================================
// Constructor
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorDecl {
    pub params: Vec<Param>,
    pub delegate: Option<Box<Expression>>,
    pub body: Block,
}

// ============================================================================
// Functions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub static_: bool,
    pub modifier: Option<FnModifier>,
    pub name: String,
    pub params: TypeParams,
    pub parameters: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: BlockOrSemi,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FnModifier {
    Virtual,
    Override,
    Abstract,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockOrSemi {
    Block(Block),
    Semi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub mutable: bool,
    pub name: String,
    pub type_: TypeRef,
    pub default: Option<Expression>,
}

pub type Block = Vec<Statement>;

// ============================================================================
// Traits
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub params: TypeParams,
    pub members: Vec<TraitMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitMember {
    Fn(FnSignature),
    Prop(PropSignature),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropSignature {
    pub name: String,
    pub type_: TypeRef,
}

// ============================================================================
// Impl blocks
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ImplBlock {
    TraitImpl {
        trait_: TraitRef,
        for_: TypeRef,
        members: Vec<ClassMember>,
    },
    Inherent {
        type_: TypeRef,
        members: Vec<ClassMember>,
    },
}

// ============================================================================
// Properties
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct PropDecl {
    pub name: String,
    pub type_: Option<TypeRef>,
    pub accessors: PropAccessors,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropAccessors {
    Braces {
        get: Option<Block>,
        set: Option<(String, Block)>,
    },
    Expression(Expression),
}

// ============================================================================
// Static declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct StaticDecl {
    pub name: String,
    pub type_: TypeRef,
    pub value: Expression,
}

// ============================================================================
// Statements
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(LetDecl),
    Assignment(Assignment),
    If(IfExpr),
    Match(MatchExpr),
    Loop(LoopExpr),
    While(WhileExpr),
    For(ForExpr),
    Return(Option<Expression>),
    Break(Option<Expression>),
    Continue,
    SuperCall(Vec<Expression>),
    Expression(Expression),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetDecl {
    pub pattern: Pattern,
    pub type_: Option<TypeRef>,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub lvalue: LValue,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LValue {
    Variable(String),
    Field(Box<LValue>, String),
    Index(Box<LValue>, Box<Expression>),
}

// ============================================================================
// Expressions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Arrow(Box<Expression>, Box<Expression>),

    // Logical
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),

    // Comparison
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    Less(Box<Expression>, Box<Expression>),
    Greater(Box<Expression>, Box<Expression>),
    LessEq(Box<Expression>, Box<Expression>),
    GreaterEq(Box<Expression>, Box<Expression>),

    // Pipe
    Pipe(Box<Expression>, Box<Expression>),

    // Range
    Range(Box<Expression>, Box<Expression>),

    // Arithmetic
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),

    // Unary
    Neg(Box<Expression>),
    Not(Box<Expression>),
    Ref(Box<Expression>),
    MutRef(Box<Expression>),

    // Suffix operations
    FieldAccess(Box<Expression>, String),
    Call(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Unwrap(Box<Expression>),
    ForceUnwrap(Box<Expression>),

    // Primary
    Literal(Literal),
    Identifier(String),
    Self_,
    Super_,
    Block(Block),
    If(IfExpr),
    Match(MatchExpr),
    Loop(LoopExpr),
    While(WhileExpr),
    For(ForExpr),
    Lambda {
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Block,
    },
    Paren(Box<Expression>),
    Array(Vec<Expression>),
}

// ============================================================================
// Control flow
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expression>,
    pub then_block: Block,
    pub else_branch: Option<Box<ElseBranch>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElseBranch {
    If(IfExpr),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub value: Box<Expression>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Identifier(String),
    Wildcard,
    Literal(Literal),
    DestructureVariant {
        name: String,
        patterns: Vec<Pattern>,
    },
    DestructureTuple(Vec<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpr {
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileExpr {
    pub condition: Box<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForExpr {
    pub pattern: Pattern,
    pub iterable: Box<Expression>,
    pub body: Block,
}

// ============================================================================
// Literals
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(String),
    Float(String),
    String(String),
    Char(char),
    Bool(bool),
    Null,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(s) => write!(f, "{s}"),
            Literal::Float(s) => write!(f, "{s}"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Char(c) => write!(f, "'{c}'"),
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::Null => write!(f, "null"),
        }
    }
}
