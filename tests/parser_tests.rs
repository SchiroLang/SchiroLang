use schiro_lexer::lexer::Lexer;
use schiro_parser::Parser;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn parse(source: &str) -> schiro_ast::CompilationUnit {
    let tokens: Vec<_> = Lexer::new(source).collect();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    if !parser.errors.is_empty() {
        for e in &parser.errors {
            eprintln!("parse error [{}:{}]: {}", e.line, e.column, e.message);
        }
        panic!("parse failed with {} error(s)", parser.errors.len());
    }
    ast
}

fn parse_with_errors(source: &str) -> (schiro_ast::CompilationUnit, usize) {
    let tokens: Vec<_> = Lexer::new(source).collect();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    (ast, parser.errors.len())
}

// ============================================================================
// Empty / minimal
// ============================================================================

#[test]
fn test_empty_source() {
    let cu = parse("");
    assert!(cu.imports.is_empty());
    assert!(cu.declarations.is_empty());
}

#[test]
fn test_only_imports() {
    let cu = parse("import foo; import bar.baz as qux;");
    assert_eq!(cu.imports.len(), 2);
    assert_eq!(cu.imports[0].path, vec!["foo"]);
    assert_eq!(cu.imports[0].alias, None);
    assert_eq!(cu.imports[1].path, vec!["bar", "baz"]);
    assert_eq!(cu.imports[1].alias, Some("qux".into()));
}

// ============================================================================
// Type declarations
// ============================================================================

#[test]
fn test_type_decl_simple() {
    let cu = parse("type Foo = Bar;");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::TypeDef(td) => {
            assert_eq!(td.name, "Foo");
            assert!(td.params.params.is_empty());
            assert_eq!(td.sum_type.variants.len(), 1);
            assert_eq!(td.sum_type.variants[0].name, "Bar");
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn test_type_decl_multi_variant() {
    let cu = parse("type Option<T> = Some(T) | None;");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::TypeDef(td) => {
            assert_eq!(td.name, "Option");
            assert_eq!(td.params.params.len(), 1);
            assert_eq!(td.params.params[0].name, "T");
            assert!(td.params.params[0].constraints.is_none());
            assert_eq!(td.sum_type.variants.len(), 2);
            assert_eq!(td.sum_type.variants[0].name, "Some");
            assert!(td.sum_type.variants[0].fields.is_some());
            assert_eq!(td.sum_type.variants[1].name, "None");
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn test_type_decl_with_variant_impl() {
    let cu = parse("type Shape = Circle(Float) impl Drawable | Rect(Float, Float);");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::TypeDef(td) => {
            let v0 = &td.sum_type.variants[0];
            assert_eq!(v0.name, "Circle");
            assert!(v0.trait_impls.is_some());
            assert_eq!(v0.trait_impls.as_ref().unwrap().len(), 1);
            assert_eq!(v0.trait_impls.as_ref().unwrap()[0].name, "Drawable");
        }
        _ => panic!("expected TypeDef"),
    }
}

// ============================================================================
// Class declarations
// ============================================================================

#[test]
fn test_class_empty() {
    let cu = parse("class Foo;");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            assert!(!c.abstract_);
            assert_eq!(c.name, "Foo");
            assert!(c.primary_constructor.is_none());
            assert!(c.extends.is_none());
            assert!(c.impls.is_none());
            assert!(matches!(c.body, schiro_ast::ClassBody::Semi));
        }
        _ => panic!("expected Class"),
    }
}

#[test]
fn test_class_with_constructor_and_body() {
    let src = "class Point(x: Int, y: Int): { fn area(self) -> Int { return 0; } }";
    let cu = parse(src);
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            assert_eq!(c.name, "Point");
            let fields = c.primary_constructor.as_ref().unwrap();
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[1].name, "y");
        }
        _ => panic!("expected Class"),
    }
}

#[test]
fn test_class_abstract() {
    let cu = parse("abstract class Foo;");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            assert!(c.abstract_);
        }
        _ => panic!("expected Class"),
    }
}

#[test]
fn test_class_extends_impl() {
    let cu = parse("class Dog extends Animal impl Pet, Mammal: { }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            assert!(c.extends.is_some());
            assert!(c.impls.is_some());
            assert_eq!(c.impls.as_ref().unwrap().len(), 2);
        }
        _ => panic!("expected Class"),
    }
}

// ============================================================================
// Function declarations
// ============================================================================

#[test]
fn test_fn_decl_no_params() {
    let cu = parse("fn foo() -> Int { return 42; }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Fn(f) => {
            assert_eq!(f.name, "foo");
            assert!(f.parameters.is_empty());
            assert!(f.return_type.is_some());
            assert!(matches!(f.body, schiro_ast::BlockOrSemi::Block(_)));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_fn_decl_with_params() {
    let cu = parse("fn add(a: Int, b: Int) -> Int { return a + b; }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Fn(f) => {
            assert_eq!(f.name, "add");
            assert_eq!(f.parameters.len(), 2);
            assert_eq!(f.parameters[0].name, "a");
            assert_eq!(f.parameters[1].name, "b");
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_fn_decl_semicolon() {
    let cu = parse("fn foo();");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Fn(f) => {
            assert!(matches!(f.body, schiro_ast::BlockOrSemi::Semi));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_fn_static_virtual() {
    let cu = parse("static fn create() -> Self { return new(); }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Fn(f) => {
            assert!(f.static_);
        }
        _ => panic!("expected Fn"),
    }
}

// ============================================================================
// Trait declarations
// ============================================================================

#[test]
fn test_trait_decl() {
    let cu = parse("trait ToString: { fn to_string() -> String; }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Trait(t) => {
            assert_eq!(t.name, "ToString");
            assert_eq!(t.members.len(), 1);
            match &t.members[0] {
                schiro_ast::TraitMember::Fn(sig) => {
                    assert_eq!(sig.name, "to_string");
                    assert!(sig.return_type.is_some());
                }
                _ => panic!("expected Fn trait member"),
            }
        }
        _ => panic!("expected Trait"),
    }
}

#[test]
fn test_trait_with_prop() {
    let cu = parse("trait HasArea: { fn area() -> Float; prop name: String; }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Trait(t) => {
            assert_eq!(t.members.len(), 2);
            assert!(matches!(t.members[1], schiro_ast::TraitMember::Prop(_)));
        }
        _ => panic!("expected Trait"),
    }
}

// ============================================================================
// Impl blocks
// ============================================================================

#[test]
fn test_inherent_impl() {
    let cu = parse("impl Foo: { fn bar() -> Int { return 1; } }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Impl(imp) => {
            assert!(matches!(imp, schiro_ast::ImplBlock::Inherent { .. }));
        }
        _ => panic!("expected Impl"),
    }
}

#[test]
fn test_trait_impl() {
    let cu = parse("impl ToString for Int: { fn to_string() -> String { return \"42\"; } }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Impl(imp) => {
            assert!(matches!(imp, schiro_ast::ImplBlock::TraitImpl { .. }));
        }
        _ => panic!("expected Impl"),
    }
}

// ============================================================================
// Properties
// ============================================================================

#[test]
fn test_prop_expression() {
    let cu = parse("class Foo: { prop name = \"bar\"; }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            let members = match &c.body {
                schiro_ast::ClassBody::Brace(m) => m,
                _ => panic!("expected Brace body"),
            };
            assert_eq!(members.len(), 1);
            match &members[0].kind {
                schiro_ast::ClassMemberKind::Prop(p) => {
                    assert_eq!(p.name, "name");
                    assert!(matches!(p.accessors, schiro_ast::PropAccessors::Expression(_)));
                }
                _ => panic!("expected Prop"),
            }
        }
        _ => panic!("expected Class"),
    }
}

#[test]
fn test_prop_accessors() {
    let cu = parse("class Foo: { prop length: Int: { get { return 42; } set(v) { } } }");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            let members = match &c.body {
                schiro_ast::ClassBody::Brace(m) => m,
                _ => panic!("expected Brace body"),
            };
            match &members[0].kind {
                schiro_ast::ClassMemberKind::Prop(p) => {
                    assert_eq!(p.name, "length");
                    assert!(p.type_.is_some());
                    match &p.accessors {
                        schiro_ast::PropAccessors::Braces { get, set } => {
                            assert!(get.is_some());
                            assert!(set.is_some());
                        }
                        _ => panic!("expected Braces accessors"),
                    }
                }
                _ => panic!("expected Prop"),
            }
        }
        _ => panic!("expected Class"),
    }
}

// ============================================================================
// Static declarations
// ============================================================================

#[test]
fn test_static_decl() {
    let cu = parse("let VERSION: Int = 1;");
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Static(s) => {
            assert_eq!(s.name, "VERSION");
            assert_eq!(s.value, schiro_ast::Expression::Literal(schiro_ast::Literal::Int("1".into())));
        }
        _ => panic!("expected Static"),
    }
}

// ============================================================================
// Statements
// ============================================================================

#[test]
fn test_let_statement() {
    let cu = parse("fn foo() { let x: Int = 42; }");
    // just checking no parse error
}

#[test]
fn test_assignment() {
    let cu = parse("fn foo() { x = 42; }");
    // just checking no parse error
}

#[test]
fn test_return_none() {
    let cu = parse("fn foo() { return; }");
    // just checking no parse error
}

#[test]
fn test_return_value() {
    let cu = parse("fn foo() -> Int { return 42; }");
    // just checking no parse error
}

#[test]
fn test_break_continue() {
    let cu = parse("fn foo() { loop { break; continue; } }");
    // just checking no parse error
}

// ============================================================================
// Expressions
// ============================================================================

#[test]
fn test_literals() {
    let cu = parse("fn foo() {
        let a = 42;
        let b = 3.14;
        let c = \"hello\";
        let d = 'x';
        let e = true;
        let f = false;
        let g = null;
    }");
    // just checking no parse error
}

#[test]
fn test_arithmetic() {
    let cu = parse("fn foo() { let r = a + b * c - d / e % f; }");
}

#[test]
fn test_comparison() {
    let cu = parse("fn foo() { let r = a == b && c != d || e < f; }");
}

#[test]
fn test_pipe() {
    let cu = parse("fn foo() { let r = x |> f |> g; }");
}

#[test]
fn test_range() {
    let cu = parse("fn foo() { let r = 0..n; }");
}

#[test]
fn test_unary() {
    let cu = parse("fn foo() { let a = -x; let b = !flag; let c = &val; let d = &mut val; }");
}

#[test]
fn test_field_call_index() {
    let cu = parse("fn foo() { let r = obj.field(args)[idx]?; }");
}

#[test]
fn test_lambda() {
    let cu = parse("fn foo() { let f = |x: Int| { x + 1; }; }");
}

#[test]
fn test_lambda_with_return() {
    let cu = parse("fn foo() { let f = |a: Int, b: Int| -> Int { return a + b; }; }");
}

// ============================================================================
// Control flow
// ============================================================================

#[test]
fn test_if_else() {
    let cu = parse("fn foo() { if x { return 1; } else { return 2; } }");
}

#[test]
fn test_if_else_if() {
    let cu = parse("fn foo() { if x { } else if y { } else { } }");
}

#[test]
fn test_match() {
    let cu = parse("fn foo() { match x: { 1 => true, _ => false } }");
}

#[test]
fn test_match_with_guard() {
    let cu = parse("fn foo() { match x: { n if n > 0 => true, _ => false } }");
}

#[test]
fn test_loop() {
    let cu = parse("fn foo() { loop { break; } }");
}

#[test]
fn test_while() {
    let cu = parse("fn foo() { while condition { } }");
}

#[test]
fn test_for() {
    let cu = parse("fn foo() { for x in items { } }");
}

#[test]
fn test_for_destructure() {
    let cu = parse("fn foo() { for [x, y] in pairs { } }");
}

// ============================================================================
// Patterns
// ============================================================================

#[test]
fn test_let_destructure() {
    let cu = parse("fn foo() { let [x, y] = point; }");
}

#[test]
fn test_let_variant() {
    let cu = parse("fn foo() { let Some(val) = opt; }");
}

#[test]
fn test_match_or_pattern() {
    let cu = parse("fn foo() { match x: { 1 | 2 => true, _ => false } }");
}

// ============================================================================
// Type references
// ============================================================================

#[test]
fn test_fn_type() {
    let cu = parse("fn foo(f: Int -> String) {}");
}

#[test]
fn test_ref_type() {
    let cu = parse("fn foo(p: &Foo) {}");
}

#[test]
fn test_mut_type() {
    let cu = parse("fn foo(p: mut Foo) {}");
}

#[test]
fn test_array_type() {
    let cu = parse("fn foo(p: [Int]) {}");
}

#[test]
fn test_tuple_type() {
    let cu = parse("fn foo(p: (Int, String)) {}");
}

#[test]
fn test_generic_type() {
    let cu = parse("fn foo(p: Option<Int>) {}");
}

#[test]
fn test_self_type() {
    let cu = parse("fn foo() -> Self { return self; }");
}

// ============================================================================
// Error handling
// ============================================================================

#[test]
fn test_unclosed_block() {
    let (cu, errors) = parse_with_errors("fn foo() { let x = 1; ");
    assert!(errors > 0, "expected parse errors");
    // The parser should still produce a result
    assert_eq!(cu.declarations.len(), 1);
}

#[test]
fn test_missing_semicolon() {
    let (cu, errors) = parse_with_errors("fn foo() { let x = 1 }");
    assert!(errors > 0, "expected parse errors");
}

#[test]
fn test_invalid_top_level() {
    let (cu, errors) = parse_with_errors("invalid_stuff");
    assert!(errors > 0, "expected parse errors");
}

// ============================================================================
// Real-world snippets
// ============================================================================

#[test]
fn test_complex_program() {
    let src = r#"
import io.console as console;

type Option<T> = Some(T) | None;

trait ToString: {
    fn to_string() -> String;
}

class Vector2(x: Float, y: Float): {
    fn length(self) -> Float {
        return (self.x * self.x + self.y * self.y).sqrt();
    }
    fn to_string(self) -> String {
        return "Vector2(" + self.x + ", " + self.y + ")";
    }
}

impl ToString for Vector2: {
    fn to_string() -> String { return self.to_string(); }
}

fn main() -> Int {
    let v = Vector2(3.0, 4.0);
    let len = v.length();
    console.print(len);
    return 0;
}
"#;
    let cu = parse(src);
    assert_eq!(cu.imports.len(), 1);
    assert_eq!(cu.declarations.len(), 5); // Option, ToString, Vector2, impl, main
}

#[test]
fn test_abstract_class_with_constructor() {
    let src = "abstract class Animal(name: String): { fn speak(self) -> String; }";
    let cu = parse(src);
    match &cu.declarations[0] {
        schiro_ast::TopLevelDecl::Class(c) => {
            assert!(c.abstract_);
            assert!(c.primary_constructor.is_some());
        }
        _ => panic!("expected Class"),
    }
}

#[test]
fn test_visibility() {
    let src = "class Foo: {
        public fn pub_fn() {}
        private fn priv_fn() {}
        protected fn prot_fn() {}
    }";
    let cu = parse(src);
}

#[test]
fn test_constructor_with_delegate() {
    let src = "class Child(x: Int): {
        fn new(x: Int): super(x) {}
    }";
    let cu = parse(src);
}

#[test]
fn test_nested_block() {
    let src = "fn foo() { { { let x = 1; } } }";
    let cu = parse(src);
}
