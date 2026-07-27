use std::collections::HashMap;

use inkwell::values::BasicValue;
use schiro_ast::*;

use crate::context::LlvmContext;
use crate::expr::{ExprGen, LocalEntry};
use crate::types::{LlvmType, TypeMapper};

pub struct FuncGen<'a, 'ctx> {
    pub ctx: &'a LlvmContext<'ctx>,
    pub mapper: &'a TypeMapper<'ctx>,
}

impl<'a, 'ctx> FuncGen<'a, 'ctx> {
    pub fn new(ctx: &'a LlvmContext<'ctx>, mapper: &'a TypeMapper<'ctx>) -> Self {
        Self { ctx, mapper }
    }

    pub fn compile_fn_decl(&self, fd: &FnDecl) {
        let ret_ty = fd.return_type.as_ref()
            .map(|t| schiro_semantic::Ty::from_type_ref(t, &[]))
            .unwrap_or(schiro_semantic::Ty::Named("Void".into()));
        let ret_ll = self.mapper.map_return(&ret_ty);

        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = fd
            .parameters
            .iter()
            .map(|p| {
                let ty = schiro_semantic::Ty::from_type_ref(&p.type_, &[]);
                self.mapper.map(&ty).as_basic_metadata_type_enum()
                    .unwrap_or_else(|| self.ctx.context.i32_type().into())
            })
            .collect();

        let fn_type = {
            match &ret_ll {
                LlvmType::Int(t) => t.fn_type(&param_types, false),
                LlvmType::Float(t) => t.fn_type(&param_types, false),
                LlvmType::Void(t) => t.fn_type(&param_types, false),
                LlvmType::Bool(t) => t.fn_type(&param_types, false),
                _ => self.ctx.context.i32_type().fn_type(&param_types, false),
            }
        };

        let function = self.ctx.module.add_function(&fd.name, fn_type, None);
        let entry = self.ctx.context.append_basic_block(function, "entry");
        self.ctx.builder.position_at_end(entry);

        let mut locals: HashMap<String, LocalEntry<'ctx>> = HashMap::new();
        for (i, param) in fd.parameters.iter().enumerate() {
            if let Some(param_value) = function.get_nth_param(i as u32) {
                let param_ty = schiro_semantic::Ty::from_type_ref(&param.type_, &[]);
                let llvm_ty = self.mapper.map(&param_ty);
                if let Some(basic_ty) = llvm_ty.as_basic_type_enum() {
                    if let Ok(alloca) = self.ctx.builder.build_alloca(basic_ty, &param.name) {
                        let _ = self.ctx.builder.build_store(alloca, param_value);
                        locals.insert(param.name.clone(), (alloca, basic_ty));
                    }
                }
            }
        }

        if let BlockOrSemi::Block(body) = &fd.body {
            self.compile_block(body, &mut locals);
        }

        self.build_term_return(&ret_ll);
    }

    fn build_term_return(&self, ret_ll: &LlvmType<'ctx>) {
        let block = match self.ctx.builder.get_insert_block() {
            Some(b) => b,
            None => return,
        };
        if block.get_terminator().is_some() { return; }
        match ret_ll {
            LlvmType::Void(_) => { let _ = self.ctx.builder.build_return(None); }
            LlvmType::Int(t) => { let _ = self.ctx.builder.build_return(Some(&t.const_zero() as &dyn BasicValue)); }
            _ => { let _ = self.ctx.builder.build_return(None); }
        }
    }

    fn compile_block(&self, stmts: &[Statement], locals: &mut HashMap<String, LocalEntry<'ctx>>) {
        let mut expr_gen = ExprGen::new(self.ctx, self.mapper, locals);
        for stmt in stmts {
            self.compile_statement(stmt, &mut expr_gen);
        }
    }

    fn compile_statement(&self, stmt: &Statement, expr_gen: &mut ExprGen<'a, 'ctx>) {
        match stmt {
            Statement::Let(let_decl) => {
                if let Some(val) = expr_gen.gen(&let_decl.value) {
                    let ty = val.get_type();
                    if let Ok(ptr) = self.ctx.builder.build_alloca(ty, "let") {
                        let _ = self.ctx.builder.build_store(ptr, val);
                        if let Pattern::Identifier(name) = &let_decl.pattern {
                            expr_gen.locals.insert(name.clone(), (ptr, ty));
                        }
                    }
                }
            }
            Statement::Assignment(assign) => {
                let name = match &assign.lvalue {
                    LValue::Variable(n) => n.clone(),
                    _ => return,
                };
                if let Some(val) = expr_gen.gen(&assign.value) {
                    if let Some(&(ptr, _)) = expr_gen.locals.get(&name) {
                        let _ = self.ctx.builder.build_store(ptr, val);
                    }
                }
            }
            Statement::Return(ret) => {
                if let Some(expr) = ret {
                    if let Some(val) = expr_gen.gen(expr) {
                        let _ = self.ctx.builder.build_return(Some(&val));
                    }
                } else {
                    let _ = self.ctx.builder.build_return(None);
                }
            }
            Statement::If(if_expr) => self.compile_if(if_expr, expr_gen),
            Statement::While(while_expr) => {
                let parent = self.ctx.builder.get_insert_block().unwrap();
                let function = parent.get_parent().unwrap();
                let cond_bb = self.ctx.context.append_basic_block(function, "while_cond");
                let body_bb = self.ctx.context.append_basic_block(function, "while_body");
                let end_bb = self.ctx.context.append_basic_block(function, "while_end");

                let _ = self.ctx.builder.build_unconditional_branch(cond_bb);
                self.ctx.builder.position_at_end(cond_bb);
                let cond = expr_gen.gen(&while_expr.condition);
                if let Some(inkwell::values::BasicValueEnum::IntValue(cond_val)) = cond {
                    let _ = self.ctx.builder.build_conditional_branch(cond_val, body_bb, end_bb);
                } else {
                    let _ = self.ctx.builder.build_unconditional_branch(body_bb);
                }

                self.ctx.builder.position_at_end(body_bb);
                self.compile_block(&while_expr.body, expr_gen.locals);
                let _ = self.ctx.builder.build_unconditional_branch(cond_bb);

                self.ctx.builder.position_at_end(end_bb);
            }
            Statement::Loop(loop_expr) => {
                let parent = self.ctx.builder.get_insert_block().unwrap();
                let function = parent.get_parent().unwrap();
                let body_bb = self.ctx.context.append_basic_block(function, "loop_body");
                let end_bb = self.ctx.context.append_basic_block(function, "loop_end");

                let _ = self.ctx.builder.build_unconditional_branch(body_bb);
                self.ctx.builder.position_at_end(body_bb);
                self.compile_block(&loop_expr.body, expr_gen.locals);
                let _ = self.ctx.builder.build_unconditional_branch(body_bb);

                self.ctx.builder.position_at_end(end_bb);
            }
            Statement::Block(block) => self.compile_block(block, expr_gen.locals),
            Statement::Expression(_) | Statement::Break(_) | Statement::Continue
            | Statement::SuperCall(_) | Statement::Match(_) | Statement::For(_) => {}
        }
    }

    fn compile_if(&self, if_expr: &IfExpr, expr_gen: &mut ExprGen<'a, 'ctx>) {
        let parent = self.ctx.builder.get_insert_block().unwrap();
        let function = parent.get_parent().unwrap();
        let then_bb = self.ctx.context.append_basic_block(function, "if_then");
        let else_bb = self.ctx.context.append_basic_block(function, "if_else");
        let end_bb = self.ctx.context.append_basic_block(function, "if_end");

        let cond = expr_gen.gen(&if_expr.condition);
        if let Some(inkwell::values::BasicValueEnum::IntValue(cond_val)) = cond {
            let _ = self.ctx.builder.build_conditional_branch(cond_val, then_bb, else_bb);
        } else {
            let _ = self.ctx.builder.build_unconditional_branch(then_bb);
        }

        self.ctx.builder.position_at_end(then_bb);
        self.compile_block(&if_expr.then_block, expr_gen.locals);
        self.maybe_br(end_bb);

        self.ctx.builder.position_at_end(else_bb);
        match &if_expr.else_branch {
            Some(else_branch) => match else_branch.as_ref() {
                ElseBranch::If(inner) => self.compile_if(inner, expr_gen),
                ElseBranch::Block(b) => {
                    self.compile_block(b, expr_gen.locals);
                    self.maybe_br(end_bb);
                }
            },
            None => self.maybe_br(end_bb),
        }

        self.ctx.builder.position_at_end(end_bb);
    }

    fn maybe_br(&self, target: inkwell::basic_block::BasicBlock<'ctx>) {
        let block = match self.ctx.builder.get_insert_block() {
            Some(b) => b,
            None => return,
        };
        if block.get_terminator().is_none() {
            let _ = self.ctx.builder.build_unconditional_branch(target);
        }
    }
}
