pub mod context;
pub mod emit;
pub mod expr;
pub mod func;
pub mod types;

use std::path::Path;

use schiro_ast::CompilationUnit;

pub use context::LlvmContext;
pub use types::TypeMapper;

pub struct CodeGen<'ctx> {
    pub llvm: LlvmContext<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx inkwell::context::Context, module_name: &str) -> Self {
        Self { llvm: LlvmContext::new(context, module_name) }
    }

    pub fn compile(&mut self, cu: &CompilationUnit) -> String {
        let mapper = TypeMapper::new(self.llvm.context);
        let func_gen = func::FuncGen::new(&self.llvm, &mapper);
        for decl in &cu.declarations {
            if let schiro_ast::TopLevelDecl::Fn(fd) = decl {
                func_gen.compile_fn_decl(fd);
            }
        }
        self.llvm.print_to_string()
    }

    pub fn compile_to_exe(&mut self, cu: &CompilationUnit, output: &Path) -> Result<(), String> {
        self.compile(cu);
        emit::compile_to_exe(&self.llvm, output)
    }

    pub fn emit_ir(&self) -> String {
        self.llvm.print_to_string()
    }

    pub fn verify(&self) -> bool {
        self.llvm.module.verify().is_ok()
    }
}
