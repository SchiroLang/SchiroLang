use schiro_lexer::lexer::Lexer;
use schiro_lexer::token::{Token, TokenKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let t = lexer.next_token();
        let is_eof = t.kind == TokenKind::Eof;
        if !is_eof {
            tokens.push(t);
        }
        if is_eof {
            break;
        }
    }
    tokens
}

fn kinds(source: &str) -> Vec<TokenKind> {
    collect(source).into_iter().map(|t| t.kind).collect()
}

fn assert_single_kind(source: &str, expected: TokenKind) {
    let mut lexer = Lexer::new(source);
    let t = lexer.next_token();
    assert_eq!(
        t.kind, expected,
        "source {source:?}: expected {expected:?}, got {:?}",
        t.kind
    );
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::Eof,
        "source {source:?}: expected EOF after single token"
    );
}

fn assert_kinds(source: &str, expected: &[TokenKind]) {
    let actual = kinds(source);
    assert_eq!(
        actual.len(),
        expected.len(),
        "source {source:?}: length mismatch\n  expected ({}) {expected:?}\n  actual   ({}) {actual:?}",
        expected.len(),
        actual.len(),
    );
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            a, e,
            "source {source:?}: mismatch at index {i}: expected {e:?}, got {a:?}"
        );
    }
}

fn assert_error(source: &str) {
    let mut lexer = Lexer::new(source);
    let t = lexer.next_token();
    assert!(
        matches!(t.kind, TokenKind::Error(_)),
        "source {source:?}: expected Error, got {:?}",
        t.kind
    );
}

fn assert_error_msg(source: &str, substr: &str) {
    let mut lexer = Lexer::new(source);
    let t = lexer.next_token();
    match &t.kind {
        TokenKind::Error(msg) => assert!(
            msg.contains(substr),
            "source {source:?}: error {msg:?} does not contain {substr:?}"
        ),
        other => panic!(
            "source {source:?}: expected Error containing {substr:?}, got {other:?}"
        ),
    }
}

fn assert_token(source: &str, expected_kind: TokenKind, expected_line: usize, expected_col: usize) {
    let mut lexer = Lexer::new(source);
    let t = lexer.next_token();
    assert_eq!(t.kind, expected_kind);
    assert_eq!(t.line, expected_line);
    assert_eq!(t.column, expected_col);
}

// ---------------------------------------------------------------------------
// Empty & whitespace
// ---------------------------------------------------------------------------

#[test]
fn test_empty_source() {
    assert_eq!(kinds(""), vec![]);
}

#[test]
fn test_only_whitespace() {
    assert_eq!(kinds("   \t\r\n\n  \t"), vec![]);
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn test_keywords_import_as_type() {
    assert_single_kind("import", TokenKind::Import);
    assert_single_kind("as", TokenKind::As);
    assert_single_kind("type", TokenKind::Type);
}

#[test]
fn test_keywords_class_oo() {
    assert_single_kind("abstract", TokenKind::Abstract);
    assert_single_kind("class", TokenKind::Class);
    assert_single_kind("extends", TokenKind::Extends);
}

#[test]
fn test_keywords_impl_for() {
    assert_single_kind("impl", TokenKind::Impl);
    assert_single_kind("for", TokenKind::For);
}

#[test]
fn test_keywords_fn_static_virtual_override() {
    assert_single_kind("fn", TokenKind::Fn);
    assert_single_kind("new", TokenKind::New);
    assert_single_kind("static", TokenKind::Static);
    assert_single_kind("virtual", TokenKind::Virtual);
    assert_single_kind("override", TokenKind::Override);
}

#[test]
fn test_keywords_trait_prop_get_set() {
    assert_single_kind("trait", TokenKind::Trait);
    assert_single_kind("prop", TokenKind::Prop);
    assert_single_kind("get", TokenKind::Get);
    assert_single_kind("set", TokenKind::Set);
}

#[test]
fn test_keywords_control_flow() {
    assert_single_kind("let", TokenKind::Let);
    assert_single_kind("if", TokenKind::If);
    assert_single_kind("else", TokenKind::Else);
    assert_single_kind("match", TokenKind::Match);
    assert_single_kind("loop", TokenKind::Loop);
    assert_single_kind("while", TokenKind::While);
    assert_single_kind("for", TokenKind::For);
    assert_single_kind("break", TokenKind::Break);
    assert_single_kind("continue", TokenKind::Continue);
    assert_single_kind("return", TokenKind::Return);
}

#[test]
fn test_keywords_super_self() {
    assert_single_kind("super", TokenKind::Super);
    assert_single_kind("self", TokenKind::Self_);
}

#[test]
fn test_keywords_visibility() {
    assert_single_kind("public", TokenKind::Public);
    assert_single_kind("protected", TokenKind::Protected);
    assert_single_kind("private", TokenKind::Private);
}

#[test]
fn test_keywords_mut_true_false_null_selftype() {
    assert_single_kind("mut", TokenKind::Mut);
    assert_single_kind("true", TokenKind::True);
    assert_single_kind("false", TokenKind::False);
    assert_single_kind("null", TokenKind::Null);
    assert_single_kind("Self", TokenKind::SelfType);
}

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

#[test]
fn test_identifier_simple() {
    assert_single_kind("foo", TokenKind::Identifier("foo".into()));
    assert_single_kind("bar123", TokenKind::Identifier("bar123".into()));
    assert_single_kind("_private", TokenKind::Identifier("_private".into()));
    assert_single_kind("__hidden", TokenKind::Identifier("__hidden".into()));
    assert_single_kind("a", TokenKind::Identifier("a".into()));
}

#[test]
fn test_identifier_unicode() {
    assert_single_kind("français", TokenKind::Identifier("français".into()));
    assert_single_kind("名称", TokenKind::Identifier("名称".into()));
    assert_single_kind("π", TokenKind::Identifier("π".into()));
}

#[test]
fn test_identifier_contains_digits_and_underscores() {
    assert_single_kind("my_var_1", TokenKind::Identifier("my_var_1".into()));
    assert_single_kind("_123", TokenKind::Identifier("_123".into()));
}

#[test]
fn test_keyword_vs_identifier_boundary() {
    // 'imports' should be an identifier, not keyword import + s
    assert_single_kind("imports", TokenKind::Identifier("imports".into()));
    assert_single_kind("classic", TokenKind::Identifier("classic".into()));
    assert_single_kind("ifonly", TokenKind::Identifier("ifonly".into()));
    assert_single_kind("returning", TokenKind::Identifier("returning".into()));
}

// ---------------------------------------------------------------------------
// Numbers
// ---------------------------------------------------------------------------

#[test]
fn test_integer() {
    assert_single_kind("0", TokenKind::IntLiteral("0".into()));
    assert_single_kind("42", TokenKind::IntLiteral("42".into()));
    assert_single_kind("1234567890", TokenKind::IntLiteral("1234567890".into()));
}

#[test]
fn test_float_simple() {
    assert_single_kind("3.14", TokenKind::FloatLiteral("3.14".into()));
    assert_single_kind("0.5", TokenKind::FloatLiteral("0.5".into()));
    assert_single_kind("10.0", TokenKind::FloatLiteral("10.0".into()));
}

#[test]
fn test_float_exponent() {
    assert_single_kind("1e10", TokenKind::FloatLiteral("1e10".into()));
    assert_single_kind("2.5e3", TokenKind::FloatLiteral("2.5e3".into()));
    assert_single_kind("1E-5", TokenKind::FloatLiteral("1E-5".into()));
    assert_single_kind("1.0e+2", TokenKind::FloatLiteral("1.0e+2".into()));
    assert_single_kind("1e0", TokenKind::FloatLiteral("1e0".into()));
}

#[test]
fn test_float_number_after_dot_required() {
    // "42." without following digit is Dot token after Int
    let mut lexer = Lexer::new("42.");
    let t1 = lexer.next_token();
    assert_eq!(t1.kind, TokenKind::IntLiteral("42".into()));
    let t2 = lexer.next_token();
    assert_eq!(t2.kind, TokenKind::Dot);
}

#[test]
fn test_float_invalid_exponent() {
    assert_error_msg("1e", "exponent");
    assert_error_msg("1E+", "exponent");
}

// ---------------------------------------------------------------------------
// Strings
// ---------------------------------------------------------------------------

#[test]
fn test_string_empty() {
    assert_single_kind("\"\"", TokenKind::StringLiteral("".into()));
}

#[test]
fn test_string_basic() {
    assert_single_kind(
        "\"hello world\"",
        TokenKind::StringLiteral("hello world".into()),
    );
}

#[test]
fn test_string_escape_sequences() {
    assert_single_kind(
        "\"\\n\"",
        TokenKind::StringLiteral("\n".into()),
    );
    assert_single_kind(
        "\"\\t\"",
        TokenKind::StringLiteral("\t".into()),
    );
    assert_single_kind(
        "\"\\r\"",
        TokenKind::StringLiteral("\r".into()),
    );
    assert_single_kind(
        "\"\\0\"",
        TokenKind::StringLiteral("\0".into()),
    );
    assert_single_kind(
        "\"\\\\\"",
        TokenKind::StringLiteral("\\".into()),
    );
    assert_single_kind(
        "\"\\\"\"",
        TokenKind::StringLiteral("\"".into()),
    );
    assert_single_kind(
        "\"\\'\"",
        TokenKind::StringLiteral("'".into()),
    );
}

#[test]
fn test_string_hex_escape() {
    assert_single_kind(
        "\"\\x41\"",
        TokenKind::StringLiteral("A".into()),
    );
    assert_single_kind(
        "\"\\x7e\"",
        TokenKind::StringLiteral("~".into()),
    );
}

#[test]
fn test_string_unicode_escape() {
    assert_single_kind(
        "\"\\u{0}\"",
        TokenKind::StringLiteral("\0".into()),
    );
    assert_single_kind(
        "\"\\u{1F600}\"",
        TokenKind::StringLiteral("\u{1F600}".into()),
    );
    assert_single_kind(
        "\"\\u{41}\"",
        TokenKind::StringLiteral("A".into()),
    );
}

#[test]
fn test_string_unterminated() {
    assert_error_msg("\"hello", "unterminated");
}

#[test]
fn test_string_newline_unterminated() {
    assert_error_msg("\"hello\n", "unterminated");
}

#[test]
fn test_string_mixed() {
    assert_single_kind(
        "\"hello\\nworld\\t!\"",
        TokenKind::StringLiteral("hello\nworld\t!".into()),
    );
}

// ---------------------------------------------------------------------------
// Chars
// ---------------------------------------------------------------------------

#[test]
fn test_char_simple() {
    assert_single_kind("'a'", TokenKind::CharLiteral('a'));
    assert_single_kind("' '", TokenKind::CharLiteral(' '));
    assert_single_kind("'0'", TokenKind::CharLiteral('0'));
    assert_single_kind("'_'", TokenKind::CharLiteral('_'));
}

#[test]
fn test_char_escape() {
    assert_single_kind("'\\n'", TokenKind::CharLiteral('\n'));
    assert_single_kind("'\\t'", TokenKind::CharLiteral('\t'));
    assert_single_kind("'\\\\'", TokenKind::CharLiteral('\\'));
    assert_single_kind("'\\''", TokenKind::CharLiteral('\''));
    assert_single_kind("'\\x41'", TokenKind::CharLiteral('A'));
    assert_single_kind("'\\u{1F600}'", TokenKind::CharLiteral('\u{1F600}'));
}

#[test]
fn test_char_empty() {
    assert_error_msg("''", "empty");
}

#[test]
fn test_char_unterminated() {
    assert_error_msg("'a", "unterminated");
    assert_error("'\\");
}

#[test]
fn test_char_multi_byte() {
    assert_single_kind("'é'", TokenKind::CharLiteral('é'));
}

// ---------------------------------------------------------------------------
// Delimiters
// ---------------------------------------------------------------------------

#[test]
fn test_delimiters() {
    assert_single_kind(";", TokenKind::Semicolon);
    assert_single_kind(":", TokenKind::Colon);
    assert_single_kind(",", TokenKind::Comma);
    assert_single_kind(".", TokenKind::Dot);
    assert_single_kind("(", TokenKind::LParen);
    assert_single_kind(")", TokenKind::RParen);
    assert_single_kind("{", TokenKind::LBrace);
    assert_single_kind("}", TokenKind::RBrace);
    assert_single_kind("[", TokenKind::LBracket);
    assert_single_kind("]", TokenKind::RBracket);
}

// ---------------------------------------------------------------------------
// Operators — single char
// ---------------------------------------------------------------------------

#[test]
fn test_operators_single() {
    assert_single_kind("|", TokenKind::Pipe);
    assert_single_kind("&", TokenKind::Amp);
    assert_single_kind("!", TokenKind::Bang);
    assert_single_kind("-", TokenKind::Minus);
    assert_single_kind("+", TokenKind::Plus);
    assert_single_kind("*", TokenKind::Star);
    assert_single_kind("/", TokenKind::Slash);
    assert_single_kind("%", TokenKind::Percent);
    assert_single_kind("=", TokenKind::Equals);
    assert_single_kind("<", TokenKind::Less);
    assert_single_kind(">", TokenKind::Greater);
    assert_single_kind("?", TokenKind::Question);
    assert_single_kind("_", TokenKind::Underscore);
}

// ---------------------------------------------------------------------------
// Operators — multi char
// ---------------------------------------------------------------------------

#[test]
fn test_operators_multi_char() {
    assert_single_kind("||", TokenKind::PipePipe);
    assert_single_kind("&&", TokenKind::AmpAmp);
    assert_single_kind("==", TokenKind::EqEq);
    assert_single_kind("!=", TokenKind::BangEq);
    assert_single_kind("<=", TokenKind::LessEq);
    assert_single_kind(">=", TokenKind::GreaterEq);
    assert_single_kind("->", TokenKind::Arrow);
    assert_single_kind("=>", TokenKind::FatArrow);
    assert_single_kind("..", TokenKind::DotDot);
    assert_single_kind("|>", TokenKind::PipeGreater);
    assert_single_kind("&mut", TokenKind::AmpMut);
}

// ---------------------------------------------------------------------------
// &mut edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_amp_mut_standalone() {
    assert_single_kind("&mut", TokenKind::AmpMut);
}

#[test]
fn test_amp_then_identifier_mutable() {
    // &mutable should be Amp + Identifier("mutable")
    assert_kinds("&mutable", &[TokenKind::Amp, TokenKind::Identifier("mutable".into())]);
}

#[test]
fn test_amp_mut_with_suffix() {
    // &mutx should be Amp + Identifier("mutx")
    assert_kinds("&mutx", &[TokenKind::Amp, TokenKind::Identifier("mutx".into())]);
}

#[test]
fn test_amp_mut_among_other_tokens() {
    assert_kinds(
        "&mut x",
        &[TokenKind::AmpMut, TokenKind::Identifier("x".into())],
    );
}

#[test]
fn test_amp_fallback() {
    // & alone should be Amp
    assert_single_kind("&", TokenKind::Amp);
    // && should be AmpAmp
    assert_single_kind("&&", TokenKind::AmpAmp);
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn test_line_comment() {
    assert_eq!(kinds("// this is a comment\nfoo"), vec![TokenKind::Identifier("foo".into())]);
}

#[test]
fn test_line_comment_eof() {
    assert_eq!(kinds("// just a comment"), vec![]);
}

#[test]
fn test_block_comment_simple() {
    assert_eq!(kinds("/* comment */ foo"), vec![TokenKind::Identifier("foo".into())]);
}

#[test]
fn test_block_comment_multiline() {
    assert_eq!(
        kinds("/* line1\nline2\nline3 */ 42"),
        vec![TokenKind::IntLiteral("42".into())]
    );
}

#[test]
fn test_block_comment_nested() {
    assert_eq!(
        kinds("/* outer /* inner */ still outer */ x"),
        vec![TokenKind::Identifier("x".into())]
    );
}

#[test]
fn test_block_comment_unterminated() {
    let tokens = kinds("/* oops");
    // The unterminated comment should eat everything silently,
    // and we should just get EOF (no extra tokens)
    assert!(tokens.is_empty());
}

#[test]
fn test_comment_between_tokens() {
    assert_kinds(
        "a /* comment */ + b",
        &[
            TokenKind::Identifier("a".into()),
            TokenKind::Plus,
            TokenKind::Identifier("b".into()),
        ],
    );
}

#[test]
fn test_only_comments() {
    assert_eq!(kinds("// line\n/* block */"), vec![]);
}

// ---------------------------------------------------------------------------
// Slash vs comment disambiguation
// ---------------------------------------------------------------------------

#[test]
fn test_slash_operator() {
    assert_single_kind("/", TokenKind::Slash);
}

#[test]
fn test_slash_eq() {
    // /= is not an operator in the grammar, so it should be Slash + Equals
    assert_kinds("/=", &[TokenKind::Slash, TokenKind::Equals]);
}

// ---------------------------------------------------------------------------
// Sequences
// ---------------------------------------------------------------------------

#[test]
fn test_expression_sequence() {
    assert_kinds(
        "a + b * 3",
        &[
            TokenKind::Identifier("a".into()),
            TokenKind::Plus,
            TokenKind::Identifier("b".into()),
            TokenKind::Star,
            TokenKind::IntLiteral("3".into()),
        ],
    );
}

#[test]
fn test_comparison_chain() {
    assert_kinds(
        "x == y && a != b",
        &[
            TokenKind::Identifier("x".into()),
            TokenKind::EqEq,
            TokenKind::Identifier("y".into()),
            TokenKind::AmpAmp,
            TokenKind::Identifier("a".into()),
            TokenKind::BangEq,
            TokenKind::Identifier("b".into()),
        ],
    );
}

#[test]
fn test_pipe_forward() {
    assert_kinds(
        "x |> f |> g",
        &[
            TokenKind::Identifier("x".into()),
            TokenKind::PipeGreater,
            TokenKind::Identifier("f".into()),
            TokenKind::PipeGreater,
            TokenKind::Identifier("g".into()),
        ],
    );
}

#[test]
fn test_range() {
    assert_kinds(
        "0 .. n",
        &[
            TokenKind::IntLiteral("0".into()),
            TokenKind::DotDot,
            TokenKind::Identifier("n".into()),
        ],
    );
    // without spaces
    assert_kinds(
        "0..n",
        &[
            TokenKind::IntLiteral("0".into()),
            TokenKind::DotDot,
            TokenKind::Identifier("n".into()),
        ],
    );
}

#[test]
fn test_arrow_and_fat_arrow() {
    assert_kinds(
        "a -> b => c",
        &[
            TokenKind::Identifier("a".into()),
            TokenKind::Arrow,
            TokenKind::Identifier("b".into()),
            TokenKind::FatArrow,
            TokenKind::Identifier("c".into()),
        ],
    );
}

#[test]
fn test_if_else_sequence() {
    assert_kinds(
        "if x { y } else { z }",
        &[
            TokenKind::If,
            TokenKind::Identifier("x".into()),
            TokenKind::LBrace,
            TokenKind::Identifier("y".into()),
            TokenKind::RBrace,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::Identifier("z".into()),
            TokenKind::RBrace,
        ],
    );
}

#[test]
fn test_function_declaration() {
    assert_kinds(
        "fn add(a: Int, b: Int) -> Int { return a + b }",
        &[
            TokenKind::Fn,
            TokenKind::Identifier("add".into()),
            TokenKind::LParen,
            TokenKind::Identifier("a".into()),
            TokenKind::Colon,
            TokenKind::Identifier("Int".into()),
            TokenKind::Comma,
            TokenKind::Identifier("b".into()),
            TokenKind::Colon,
            TokenKind::Identifier("Int".into()),
            TokenKind::RParen,
            TokenKind::Arrow,
            TokenKind::Identifier("Int".into()),
            TokenKind::LBrace,
            TokenKind::Return,
            TokenKind::Identifier("a".into()),
            TokenKind::Plus,
            TokenKind::Identifier("b".into()),
            TokenKind::RBrace,
        ],
    );
}

#[test]
fn test_class_declaration() {
    assert_kinds(
        "class Vector2(x: Int, y: Int): { fn length(self) -> Float {} }",
        &[
            TokenKind::Class,
            TokenKind::Identifier("Vector2".into()),
            TokenKind::LParen,
            TokenKind::Identifier("x".into()),
            TokenKind::Colon,
            TokenKind::Identifier("Int".into()),
            TokenKind::Comma,
            TokenKind::Identifier("y".into()),
            TokenKind::Colon,
            TokenKind::Identifier("Int".into()),
            TokenKind::RParen,
            TokenKind::Colon,
            TokenKind::LBrace,
            TokenKind::Fn,
            TokenKind::Identifier("length".into()),
            TokenKind::LParen,
            TokenKind::Self_,
            TokenKind::RParen,
            TokenKind::Arrow,
            TokenKind::Identifier("Float".into()),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::RBrace,
        ],
    );
}

#[test]
fn test_import_statement() {
    assert_kinds(
        "import io.console as console;",
        &[
            TokenKind::Import,
            TokenKind::Identifier("io".into()),
            TokenKind::Dot,
            TokenKind::Identifier("console".into()),
            TokenKind::As,
            TokenKind::Identifier("console".into()),
            TokenKind::Semicolon,
        ],
    );
}

#[test]
fn test_type_declaration() {
    assert_kinds(
        "type Option<T> = Some(T) | None;",
        &[
            TokenKind::Type,
            TokenKind::Identifier("Option".into()),
            TokenKind::Less,
            TokenKind::Identifier("T".into()),
            TokenKind::Greater,
            TokenKind::Equals,
            TokenKind::Identifier("Some".into()),
            TokenKind::LParen,
            TokenKind::Identifier("T".into()),
            TokenKind::RParen,
            TokenKind::Pipe,
            TokenKind::Identifier("None".into()),
            TokenKind::Semicolon,
        ],
    );
}

#[test]
fn test_lambda() {
    assert_kinds(
        "|x: Int| x + 1",
        &[
            TokenKind::Pipe,
            TokenKind::Identifier("x".into()),
            TokenKind::Colon,
            TokenKind::Identifier("Int".into()),
            TokenKind::Pipe,
            TokenKind::Identifier("x".into()),
            TokenKind::Plus,
            TokenKind::IntLiteral("1".into()),
        ],
    );
}

#[test]
fn test_match_expression() {
    assert_kinds(
        "match x: { 1 => true, _ => false }",
        &[
            TokenKind::Match,
            TokenKind::Identifier("x".into()),
            TokenKind::Colon,
            TokenKind::LBrace,
            TokenKind::IntLiteral("1".into()),
            TokenKind::FatArrow,
            TokenKind::True,
            TokenKind::Comma,
            TokenKind::Underscore,
            TokenKind::FatArrow,
            TokenKind::False,
            TokenKind::RBrace,
        ],
    );
}

#[test]
fn test_unwrap_operators() {
    assert_kinds(
        "x? y!",
        &[
            TokenKind::Identifier("x".into()),
            TokenKind::Question,
            TokenKind::Identifier("y".into()),
            TokenKind::Bang,
        ],
    );
}

#[test]
fn test_field_access_and_call() {
    assert_kinds(
        "obj.field(args)",
        &[
            TokenKind::Identifier("obj".into()),
            TokenKind::Dot,
            TokenKind::Identifier("field".into()),
            TokenKind::LParen,
            TokenKind::Identifier("args".into()),
            TokenKind::RParen,
        ],
    );
}

// ---------------------------------------------------------------------------
// Line / Column tracking
// ---------------------------------------------------------------------------

#[test]
fn test_line_column_simple() {
    assert_token("foo", TokenKind::Identifier("foo".into()), 1, 1);

    let mut lexer = Lexer::new("a\nb");
    let t1 = lexer.next_token();
    assert_eq!(t1.kind, TokenKind::Identifier("a".into()));
    assert_eq!(t1.line, 1);
    assert_eq!(t1.column, 1);

    let t2 = lexer.next_token();
    assert_eq!(t2.kind, TokenKind::Identifier("b".into()));
    assert_eq!(t2.line, 2);
    assert_eq!(t2.column, 1);
}

#[test]
fn test_line_column_with_spaces() {
    let mut lexer = Lexer::new("  hello\n  world");
    let t1 = lexer.next_token();
    assert_eq!(t1.kind, TokenKind::Identifier("hello".into()));
    assert_eq!(t1.line, 1);
    assert_eq!(t1.column, 3);

    let t2 = lexer.next_token();
    assert_eq!(t2.kind, TokenKind::Identifier("world".into()));
    assert_eq!(t2.line, 2);
    assert_eq!(t2.column, 3);
}

#[test]
fn test_line_column_multi_line_comment() {
    let mut lexer = Lexer::new("/*\ncomment\n*/ x");
    let t = lexer.next_token();
    assert_eq!(t.kind, TokenKind::Identifier("x".into()));
    assert_eq!(t.line, 3);
    assert_eq!(t.column, 4);
}

// ---------------------------------------------------------------------------
// Error recovery
// ---------------------------------------------------------------------------

#[test]
fn test_unexpected_character() {
    assert_error("@");
    assert_error("`");
    assert_error("#");
    assert_error("~");
}

#[test]
fn test_error_among_valid_tokens() {
    let collected = collect("a @ b");
    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0].kind, TokenKind::Identifier("a".into()));
    assert!(matches!(collected[1].kind, TokenKind::Error(_)));
    assert_eq!(collected[2].kind, TokenKind::Identifier("b".into()));
}

// ---------------------------------------------------------------------------
// Span correctness
// ---------------------------------------------------------------------------

#[test]
fn test_span_simple() {
    let mut lexer = Lexer::new("hello");
    let t = lexer.next_token();
    assert_eq!(t.span.start, 0);
    assert_eq!(t.span.end, 5);
}

#[test]
fn test_span_with_whitespace() {
    let mut lexer = Lexer::new("  foo  ");
    let t = lexer.next_token();
    assert_eq!(t.span.start, 2);
    assert_eq!(t.span.end, 5);
}

#[test]
fn test_span_multi_token() {
    let tokens = collect("a + 42");
    assert_eq!(tokens[0].span, schiro_lexer::token::Span::new(0, 1));
    assert_eq!(tokens[1].span, schiro_lexer::token::Span::new(2, 3));
    assert_eq!(tokens[2].span, schiro_lexer::token::Span::new(4, 6));
}

// ---------------------------------------------------------------------------
// Real-world snippet
// ---------------------------------------------------------------------------

#[test]
fn test_trait_impl_snippet() {
    let source = "trait ToString: {\n    fn to_string() -> String;\n}\n\nimpl ToString for Int: {\n    fn to_string() -> String { \"42\" }\n}";
    let tokens = collect(source);
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Trait,
            &TokenKind::Identifier("ToString".into()),
            &TokenKind::Colon,
            &TokenKind::LBrace,
            &TokenKind::Fn,
            &TokenKind::Identifier("to_string".into()),
            &TokenKind::LParen,
            &TokenKind::RParen,
            &TokenKind::Arrow,
            &TokenKind::Identifier("String".into()),
            &TokenKind::Semicolon,
            &TokenKind::RBrace,
            &TokenKind::Impl,
            &TokenKind::Identifier("ToString".into()),
            &TokenKind::For,
            &TokenKind::Identifier("Int".into()),
            &TokenKind::Colon,
            &TokenKind::LBrace,
            &TokenKind::Fn,
            &TokenKind::Identifier("to_string".into()),
            &TokenKind::LParen,
            &TokenKind::RParen,
            &TokenKind::Arrow,
            &TokenKind::Identifier("String".into()),
            &TokenKind::LBrace,
            &TokenKind::StringLiteral("42".into()),
            &TokenKind::RBrace,
            &TokenKind::RBrace,
        ],
    );
}

// ---------------------------------------------------------------------------
// Iterator impl
// ---------------------------------------------------------------------------

#[test]
fn test_iterator_produces_non_eof_tokens() {
    let lexer = Lexer::new("a b c");
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 3);
}

#[test]
fn test_iterator_empty() {
    let lexer = Lexer::new("");
    let tokens: Vec<_> = lexer.collect();
    assert!(tokens.is_empty());
}
