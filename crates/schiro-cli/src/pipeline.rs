use std::path::Path;

use inkwell::context::Context;
use schiro_codegen::CodeGen;
use schiro_lexer::lexer::Lexer;
use schiro_parser::Parser;
use schiro_semantic::SemanticAnalysis;

pub enum PipelineError {
    Parse(String),
    Semantic(String),
    Codegen(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::Parse(msg) => write!(f, "parse error: {msg}"),
            PipelineError::Semantic(msg) => write!(f, "semantic error: {msg}"),
            PipelineError::Codegen(msg) => write!(f, "codegen error: {msg}"),
        }
    }
}



fn lex_parse(source: &str) -> std::result::Result<(schiro_ast::CompilationUnit, SemaInfo), Vec<PipelineError>> {
    let tokens: Vec<_> = Lexer::new(source).collect();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    if !parser.errors.is_empty() {
        let mut msg = String::new();
        for e in &parser.errors {
            msg.push_str(&format!("[{}:{}] {}\n", e.line, e.column, e.message));
        }
        return Err(vec![PipelineError::Parse(msg)]);
    }

    let mut semantic = SemanticAnalysis::new();
    semantic.analyze(&ast);
    if !semantic.errors().is_empty() {
        let mut msg = String::new();
        for e in semantic.errors() {
            msg.push_str(&format!("{}\n", e.kind));
        }
        return Err(vec![PipelineError::Semantic(msg)]);
    }

    let has_main = ast.declarations.iter().any(|d| {
        matches!(d, schiro_ast::TopLevelDecl::Fn(f) if f.name == "main")
    });

    Ok((ast, SemaInfo { has_main }))
}

struct SemaInfo {
    has_main: bool,
}

pub fn check_only(source: &str) -> std::result::Result<(), Vec<PipelineError>> {
    let (_, _info) = lex_parse(source)?;
    Ok(())
}

pub fn emit_ir(source: &str) -> std::result::Result<String, Vec<PipelineError>> {
    let (ast, _) = lex_parse(source)?;

    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "module");
    let ir = codegen.compile(&ast);

    Ok(ir)
}

pub fn compile_to_exe(
    source: &str,
    output: &Path,
    verbose: bool,
) -> std::result::Result<String, Vec<PipelineError>> {
    let (ast, info) = lex_parse(source)?;

    if !info.has_main {
        return Err(vec![PipelineError::Semantic(
            "no main() function found — required for executable".to_string(),
        )]);
    }

    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "module");
    let ir = codegen.compile(&ast);

    if verbose {
        println!("=== LLVM IR ===");
        println!("{ir}");
    }

    if !codegen.verify() {
        return Err(vec![PipelineError::Codegen(
            "LLVM module verification failed".into(),
        )]);
    }

    codegen
        .compile_to_exe(&ast, output)
        .map_err(|e| vec![PipelineError::Codegen(e)])?;

    Ok(ir)
}
