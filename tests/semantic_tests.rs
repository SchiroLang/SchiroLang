use schiro_lexer::lexer::Lexer;
use schiro_parser::Parser;
use schiro_semantic::SemanticAnalysis;

fn analyze(source: &str) -> (Vec<schiro_semantic::SemanticError>, usize) {
    let tokens: Vec<_> = Lexer::new(source).collect();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut semantic = SemanticAnalysis::new();
    semantic.analyze(&ast);
    let errors = semantic.errors();
    let decl_count = ast.declarations.len();
    (errors, decl_count)
}

fn analyze_ok(source: &str) -> usize {
    let (errors, count) = analyze(source);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("semantic error: {}", e.kind);
        }
        panic!("expected no semantic errors, got {}", errors.len());
    }
    count
}

#[test]
fn test_empty() {
    let (errors, count) = analyze("");
    assert!(errors.is_empty());
    assert_eq!(count, 0);
}

#[test]
fn test_fn_no_body() {
    analyze_ok("fn foo();");
}

#[test]
fn test_fn_empty_body() {
    analyze_ok("fn foo() {}");
}

#[test]
fn test_fn_with_return() {
    analyze_ok("fn foo() -> Int { return 42; }");
}

#[test]
fn test_fn_with_params() {
    analyze_ok("fn add(a: Int, b: Int) -> Int { return a + b; }");
}

#[test]
fn test_class_empty() {
    analyze_ok("class Foo;");
}

#[test]
fn test_class_with_body() {
    analyze_ok("class Foo: { fn bar() {} }");
}

#[test]
fn test_type_decl() {
    analyze_ok("type Option<T> = Some(T) | None;");
}

#[test]
fn test_trait_decl() {
    analyze_ok("trait ToString: { fn to_string() -> String; }");
}

#[test]
fn test_static_decl() {
    analyze_ok("let VERSION: Int = 1;");
}

#[test]
fn test_if_else() {
    analyze_ok("fn foo() { if true { return 1; } else { return 2; } }");
}

#[test]
fn test_loop_break() {
    analyze_ok("fn foo() { loop { break; } }");
}

#[test]
fn test_while() {
    analyze_ok("fn foo() { while true { } }");
}

#[test]
fn test_for_loop() {
    analyze_ok("fn foo() { for x in items { } }");
}

#[test]
fn test_match() {
    analyze_ok("fn foo() { match x: { 1 => true, _ => false } }");
}

#[test]
fn test_let_inference() {
    analyze_ok("fn foo() { let x = 42; }");
}

#[test]
fn test_let_with_type() {
    analyze_ok("fn foo() { let x: Int = 42; }");
}

#[test]
fn test_let_destructure() {
    analyze_ok("fn foo() { let [x, y] = point; }");
}

#[test]
fn test_nested_blocks() {
    analyze_ok("fn foo() { { { let x = 1; } } }");
}

#[test]
fn test_many_fn_decls() {
    analyze_ok("fn a() {} fn b() {} fn c() {}");
}

#[test]
fn test_class_with_constructor() {
    analyze_ok("class Point(x: Int, y: Int): { fn new(x: Int, y: Int) {} }");
}

#[test]
fn test_visibility() {
    analyze_ok("class Foo: { public fn a() {} private fn b() {} }");
}

#[test]
fn test_import() {
    analyze_ok("import foo.bar; fn main() {}");
}

#[test]
fn test_complex_program() {
    let src = r#"
type Option<T> = Some(T) | None;

trait ToString: {
    fn to_string() -> String;
}

class Vector2(x: Float, y: Float): {
    fn length(self) -> Float {
        return 0.0;
    }
}

impl ToString for Vector2: {
    fn to_string() -> String { return "vec"; }
}

fn main() -> Int {
    let v = Vector2(3.0, 4.0);
    return 0;
}
"#;
    let count = analyze_ok(src);
    assert_eq!(count, 5);
}

#[test]
fn test_assignment_immutable() {
    let source = "fn foo(x: Int) { x = 2; }";
    let (errors, _) = analyze(source);
    assert!(!errors.is_empty(), "expected error for immutable assignment");
    let has_cannot_mutate = errors.iter().any(|e| {
        matches!(&e.kind, schiro_semantic::ErrorKind::CannotMutate(_))
    });
    assert!(has_cannot_mutate, "expected CannotMutate error");
}

#[test]
fn test_mutable_assignment() {
    analyze_ok("fn foo(mut x: Int) { x = 2; }");
}

#[test]
fn test_generic_type_decl() {
    analyze_ok("type Pair<A, B> = Pair(A, B);");
}

#[test]
fn test_trait_with_prop() {
    analyze_ok("trait HasArea: { fn area() -> Float; prop name: String; }");
}

#[test]
fn test_abstract_class() {
    analyze_ok("abstract class Animal;");
}

#[test]
fn test_fn_return_no_value() {
    analyze_ok("fn foo() { return; }");
}

#[test]
fn test_break_no_value() {
    analyze_ok("fn foo() { loop { break; } }");
}

#[test]
fn test_continue() {
    analyze_ok("fn foo() { loop { continue; } }");
}
