use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FloatType, IntType, PointerType, StructType, VoidType};
use schiro_semantic::Ty;

pub struct TypeMapper<'ctx> {
    pub context: &'ctx Context,
}

#[derive(Debug, Clone)]
pub enum LlvmType<'ctx> {
    Int(IntType<'ctx>),
    Float(FloatType<'ctx>),
    Bool(IntType<'ctx>),
    Pointer(PointerType<'ctx>),
    Struct(StructType<'ctx>),
    Void(VoidType<'ctx>),
    Func,
}

impl<'ctx> LlvmType<'ctx> {
    pub fn as_basic_type_enum(&self) -> Option<BasicTypeEnum<'ctx>> {
        match self {
            LlvmType::Int(t) => Some((*t).into()),
            LlvmType::Float(t) => Some((*t).into()),
            LlvmType::Bool(t) => Some((*t).into()),
            LlvmType::Pointer(t) => Some((*t).into()),
            LlvmType::Struct(t) => Some((*t).into()),
            LlvmType::Void(_) | LlvmType::Func => None,
        }
    }

    pub fn as_basic_metadata_type_enum(&self) -> Option<BasicMetadataTypeEnum<'ctx>> {
        match self {
            LlvmType::Int(t) => Some((*t).into()),
            LlvmType::Float(t) => Some((*t).into()),
            LlvmType::Bool(t) => Some((*t).into()),
            LlvmType::Pointer(t) => Some((*t).into()),
            LlvmType::Struct(t) => Some((*t).into()),
            LlvmType::Void(_) | LlvmType::Func => None,
        }
    }
}

impl<'ctx> TypeMapper<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    pub fn map(&self, ty: &Ty) -> LlvmType<'ctx> {
        match ty {
            Ty::Named(name) => self.map_named(name),
            Ty::Generic(_) => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            Ty::Function(_, _) => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            Ty::Ref(_) | Ty::Mut(_) => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            Ty::Array(_) => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            Ty::Tuple(_) => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            Ty::Self_ => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            Ty::Class { name, .. } => self.map_named(name),
            Ty::Unknown | Ty::Error => LlvmType::Int(self.context.i32_type()),
        }
    }

    pub fn map_return(&self, ty: &Ty) -> LlvmType<'ctx> {
        if *ty == Ty::Named("Void".into()) || *ty == Ty::Named("void".into()) {
            return LlvmType::Void(self.context.void_type());
        }
        self.map(ty)
    }

    fn map_named(&self, name: &str) -> LlvmType<'ctx> {
        match name {
            "Int" | "int" | "i32" => LlvmType::Int(self.context.i32_type()),
            "i64" => LlvmType::Int(self.context.i64_type()),
            "i8" => LlvmType::Int(self.context.i8_type()),
            "Float" | "float" | "f32" => LlvmType::Float(self.context.f32_type()),
            "f64" | "Double" | "double" => LlvmType::Float(self.context.f64_type()),
            "Bool" | "bool" => LlvmType::Bool(self.context.bool_type()),
            "String" | "string" | "str" => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
            "Void" | "void" | "Unit" | "unit" => LlvmType::Void(self.context.void_type()),
            _ => LlvmType::Pointer(self.context.ptr_type(inkwell::AddressSpace::default())),
        }
    }
}
