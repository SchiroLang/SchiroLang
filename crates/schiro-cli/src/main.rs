use std::path::PathBuf;

use clap::{Parser as ClapParser, Subcommand};

mod pipeline;

#[derive(ClapParser)]
#[command(name = "schiro", version, about = "SchiroLang compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile source to executable
    Build {
        /// Input .schiro file
        file: PathBuf,

        /// Output executable path (default: same name without .schiro)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Only emit LLVM IR (no executable)
        #[arg(long)]
        emit_ir: bool,

        /// Print LLVM IR to stdout during compilation
        #[arg(long)]
        verbose: bool,
    },
    /// Parse & type-check only
    Check {
        /// Input .schiro file
        file: PathBuf,
    },
    /// Print LLVM IR and exit
    EmitIr {
        /// Input .schiro file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Build { file, output, emit_ir, verbose } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {e}", file.display());
                    std::process::exit(1);
                }
            };

            let out_path = output.unwrap_or_else(|| {
                let mut p = file.clone();
                p.set_extension("");
                p
            });

            match pipeline::compile_to_exe(&source, &out_path, emit_ir || verbose) {
                Ok(ir) => {
                    if verbose || emit_ir {
                        println!("{ir}");
                    }
                    if !emit_ir {
                        println!("✓ wrote {}", out_path.display());
                    }
                }
                Err(errors) => {
                    for e in errors {
                        eprintln!("error: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Command::Check { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {e}", file.display());
                    std::process::exit(1);
                }
            };

            match pipeline::check_only(&source) {
                Ok(()) => println!("✓ {}: OK", file.display()),
                Err(errors) => {
                    for e in errors {
                        eprintln!("error: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Command::EmitIr { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {e}", file.display());
                    std::process::exit(1);
                }
            };

            match pipeline::emit_ir(&source) {
                Ok(ir) => println!("{ir}"),
                Err(errors) => {
                    for e in errors {
                        eprintln!("error: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
