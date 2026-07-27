use std::path::Path;
use std::process::Command;

use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

use crate::context::LlvmContext;

pub fn emit_object(llvm: &LlvmContext, path: &Path) -> Result<(), String> {
    let triple = TargetMachine::get_default_triple();

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("target init failed: {e}"))?;
    let target = Target::from_triple(&triple).map_err(|e| format!("get target: {e}"))?;

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("failed to create TargetMachine")?;

    machine
        .write_to_file(&llvm.module, FileType::Object, path)
        .map_err(|e| format!("write object failed: {e}"))?;

    Ok(())
}

pub fn link_exe(object_path: &Path, output_path: &Path) -> Result<(), String> {
    let status = Command::new("cc")
        .arg(object_path)
        .arg("-o")
        .arg(output_path)
        .status()
        .map_err(|e| format!("failed to invoke linker: {e}"))?;

    if !status.success() {
        return Err("linker returned non-zero exit status".into());
    }
    Ok(())
}

pub fn compile_to_exe(llvm: &LlvmContext, output_path: &Path) -> Result<(), String> {
    let obj_path = output_path.with_extension("o");
    emit_object(llvm, &obj_path)?;
    link_exe(&obj_path, output_path)?;
    let _ = std::fs::remove_file(&obj_path);
    Ok(())
}
