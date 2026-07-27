use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, line: usize, column: usize) -> Self {
        Self { kind, span, line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Import,
    As,
    Type,
    Abstract,
    Class,
    Extends,
    Impl,
    For,
    Fn,
    New,
    Static,
    Virtual,
    Override,
    Trait,
    Prop,
    Get,
    Set,
    Let,
    If,
    Else,
    Match,
    Loop,
    While,
    Break,
    Continue,
    Return,
    Super,
    Self_,
    Public,
    Protected,
    Private,
    Mut,
    True,
    False,
    Null,
    SelfType,

    // Delimiters
    Semicolon,
    Colon,
    Comma,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Single-char operators
    Pipe,
    Amp,
    Bang,
    Minus,
    Plus,
    Star,
    Slash,
    Percent,
    Equals,
    Less,
    Greater,
    Question,
    Underscore,

    // Multi-char operators
    PipePipe,
    AmpAmp,
    EqEq,
    BangEq,
    LessEq,
    GreaterEq,
    Arrow,
    FatArrow,
    DotDot,
    PipeGreater,
    AmpMut,

    // Literals
    IntLiteral(String),
    FloatLiteral(String),
    StringLiteral(String),
    CharLiteral(char),

    // Identifiers
    Identifier(String),

    // Special
    Error(String),
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Import => write!(f, "import"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Abstract => write!(f, "abstract"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Extends => write!(f, "extends"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Static => write!(f, "static"),
            TokenKind::Virtual => write!(f, "virtual"),
            TokenKind::Override => write!(f, "override"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Prop => write!(f, "prop"),
            TokenKind::Get => write!(f, "get"),
            TokenKind::Set => write!(f, "set"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::Self_ => write!(f, "self"),
            TokenKind::Public => write!(f, "public"),
            TokenKind::Protected => write!(f, "protected"),
            TokenKind::Private => write!(f, "private"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::SelfType => write!(f, "Self"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Amp => write!(f, "&"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::AmpAmp => write!(f, "&&"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::BangEq => write!(f, "!="),
            TokenKind::LessEq => write!(f, "<="),
            TokenKind::GreaterEq => write!(f, ">="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::PipeGreater => write!(f, "|>"),
            TokenKind::AmpMut => write!(f, "&mut"),
            TokenKind::IntLiteral(s) => write!(f, "{s}"),
            TokenKind::FloatLiteral(s) => write!(f, "{s}"),
            TokenKind::StringLiteral(s) => write!(f, "\"{s}\""),
            TokenKind::CharLiteral(c) => write!(f, "'{c}'"),
            TokenKind::Identifier(s) => write!(f, "{s}"),
            TokenKind::Error(msg) => write!(f, "error({msg})"),
            TokenKind::Eof => write!(f, "eof"),
        }
    }
}
