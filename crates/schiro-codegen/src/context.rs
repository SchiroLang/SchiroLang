use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

pub struct LlvmContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> LlvmContext<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        Self { context, module, builder }
    }

    pub fn print_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }
}
