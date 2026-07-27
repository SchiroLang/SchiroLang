use std::path::Path;

use inkwell::context::Context;
use schiro_codegen::CodeGen;
use schiro_lexer::lexer::Lexer;
use schiro_parser::Parser;
use schiro_semantic::SemanticAnalysis;

fn main() {
    let source = r#"
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn main() -> Int {
    let x: Int = 40;
    let y: Int = 2;
    return add(x, y);
}
"#;

    // 1. Lexer
    let tokens: Vec<_> = Lexer::new(source).collect();

    // 2. Parser
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    if !parser.errors.is_empty() {
        for e in &parser.errors {
            eprintln!("parse error [{}:{}]: {}", e.line, e.column, e.message);
        }
        std::process::exit(1);
    }
    println!("✓ Parsed successfully");

    // 3. Semantic analysis
    let mut semantic = SemanticAnalysis::new();
    semantic.analyze(&ast);
    if !semantic.errors().is_empty() {
        for e in semantic.errors() {
            eprintln!("semantic error: {}", e.kind);
        }
        std::process::exit(1);
    }
    println!("✓ Semantic analysis passed");

    // 4. Codegen → LLVM IR
    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "test");
    let ir = codegen.compile(&ast);
    println!("\n=== LLVM IR ===");
    println!("{ir}");

    // 5. Verify module
    if codegen.verify() {
        println!("✓ Module verified successfully");
    } else {
        println!("✗ Module verification failed");
    }

    // 6. Compile to executable
    let output = Path::new("/tmp/schiro_test_output");
    match codegen.compile_to_exe(&ast, output) {
        Ok(()) => {
            println!("\n✓ Executable written to: {}", output.display());
            println!("  Run it: {}", output.display());
        }
        Err(e) => {
            eprintln!("\n✗ Failed to compile executable: {e}");
            eprintln!("  (LLVM IR was generated successfully above)");
        }
    }
}
