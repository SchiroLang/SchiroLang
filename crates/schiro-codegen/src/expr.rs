use std::collections::HashMap;

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, IntValue, PointerValue};
use schiro_ast::*;

use crate::context::LlvmContext;
use crate::types::TypeMapper;

pub type LocalEntry<'ctx> = (PointerValue<'ctx>, BasicTypeEnum<'ctx>);

pub struct ExprGen<'a, 'ctx> {
    pub ctx: &'a LlvmContext<'ctx>,
    pub mapper: &'a TypeMapper<'ctx>,
    pub locals: &'a mut HashMap<String, LocalEntry<'ctx>>,
}

impl<'a, 'ctx> ExprGen<'a, 'ctx> {
    pub fn new(
        ctx: &'a LlvmContext<'ctx>,
        mapper: &'a TypeMapper<'ctx>,
        locals: &'a mut HashMap<String, LocalEntry<'ctx>>,
    ) -> Self {
        Self { ctx, mapper, locals }
    }

    fn load(&self, ptr: PointerValue<'ctx>, ty: BasicTypeEnum<'ctx>, name: &str) -> Option<BasicValueEnum<'ctx>> {
        self.ctx.builder.build_load(ty, ptr, name).ok()
    }

    pub fn gen(&mut self, expr: &Expression) -> Option<BasicValueEnum<'ctx>> {
        match expr {
            Expression::Literal(lit) => self.gen_literal(lit),
            Expression::Identifier(name) => self.gen_identifier(name),
            Expression::Add(l, r) => self.gen_binary_int(l, r, |a, b| {
                self.ctx.builder.build_int_add(*a, *b, "addtmp").ok()
            }),
            Expression::Sub(l, r) => self.gen_binary_int(l, r, |a, b| {
                self.ctx.builder.build_int_sub(*a, *b, "subtmp").ok()
            }),
            Expression::Mul(l, r) => self.gen_binary_int(l, r, |a, b| {
                self.ctx.builder.build_int_mul(*a, *b, "multmp").ok()
            }),
            Expression::Neg(inner) => {
                let val = self.gen(inner)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        let zero = self.ctx.context.i32_type().const_zero();
                        self.ctx.builder.build_int_sub(zero, v, "negtmp").ok().map(|v| v.into())
                    }
                    BasicValueEnum::FloatValue(v) => {
                        self.ctx.builder.build_float_neg(v, "negtmp").ok().map(|v| v.into())
                    }
                    _ => None,
                }
            }
            Expression::Not(inner) => {
                let val = self.gen(inner)?;
                match val {
                    BasicValueEnum::IntValue(v) => {
                        self.ctx.builder.build_not(v, "nottmp").ok().map(|v| v.into())
                    }
                    _ => None,
                }
            }
            Expression::Call(callee, args) => self.gen_call(callee, args),
            Expression::Paren(inner) => self.gen(inner),
            _ => None,
        }
    }

    fn gen_literal(&mut self, lit: &Literal) -> Option<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(s) => {
                let val: i64 = s.parse().unwrap_or(0);
                Some(self.ctx.context.i32_type().const_int(val as u64, false).into())
            }
            Literal::Float(s) => {
                let val: f64 = s.parse().unwrap_or(0.0);
                Some(self.ctx.context.f64_type().const_float(val).into())
            }
            Literal::Bool(b) => {
                Some(self.ctx.context.bool_type().const_int(*b as u64, false).into())
            }
            Literal::String(s) => {
                let gv = self.ctx.builder.build_global_string_ptr(s, "strtmp").ok()?;
                Some(gv.as_pointer_value().into())
            }
            Literal::Char(c) => {
                Some(self.ctx.context.i32_type().const_int(*c as u64, false).into())
            }
            Literal::Null => {
                let ptr_ty = self.ctx.context.ptr_type(inkwell::AddressSpace::default());
                Some(ptr_ty.const_null().into())
            }
        }
    }

    fn gen_identifier(&mut self, name: &str) -> Option<BasicValueEnum<'ctx>> {
        if let Some(&(ptr, ty)) = self.locals.get(name) {
            self.load(ptr, ty, name)
        } else if let Some(global) = self.ctx.module.get_global(name) {
            let int_ty: BasicTypeEnum<'ctx> = self.ctx.context.i32_type().into();
            self.load(global.as_pointer_value(), int_ty, name)
        } else {
            None
        }
    }

    fn gen_binary_int<F>(
        &mut self,
        left: &Expression,
        right: &Expression,
        op: F,
    ) -> Option<BasicValueEnum<'ctx>>
    where
        F: FnOnce(&IntValue<'ctx>, &IntValue<'ctx>) -> Option<IntValue<'ctx>>,
    {
        let l = self.gen(left)?;
        let r = self.gen(right)?;
        match (l, r) {
            (BasicValueEnum::IntValue(a), BasicValueEnum::IntValue(b)) => {
                op(&a, &b).map(|v| v.into())
            }
            _ => None,
        }
    }

    fn gen_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Option<BasicValueEnum<'ctx>> {
        let callee_name = match callee {
            Expression::Identifier(name) => name.clone(),
            _ => return None,
        };
        let function = self.ctx.module.get_function(&callee_name)?;
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.gen(arg)?);
        }
        let meta_args: Vec<BasicMetadataValueEnum<'ctx>> = arg_values.iter()
            .map(|v| (*v).into())
            .collect();
        let call = self.ctx.builder.build_call(function, &meta_args, "calltmp").ok()?;
        Some(call.try_as_basic_value().unwrap_basic())
    }
}
