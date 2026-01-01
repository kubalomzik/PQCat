use clap::{Parser, Subcommand};
mod algorithm_runner;
mod algorithms;
mod benchmarks;
mod code_generator;
mod codes;
mod types;

use algorithm_runner::run_algorithm;
use types::{Algorithm, CodeParams, CodePreset, CodeType, PartitionParams};

#[derive(Parser)]
#[command(name = "pqcat")]
#[command(about = "Run classical attacks on code-based cryptosystems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Prange {
        #[arg(short, long, default_value_t = 15)]
        n: usize, // Codeword length (number of bits)
        #[arg(short, long, default_value_t = 11)]
        k: usize, // Message length (number of bits)
        #[arg(short, long, default_value_t = 1)]
        w: usize, // Weight of the error vector (number of errors)
        #[arg(short, long, value_enum, default_value_t = CodeType::Hamming)]
        code_type: CodeType, // Type of code: "random", "hamming", "goppa", or "qc"
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
    },
    Stern {
        #[arg(short, long, default_value_t = 15)]
        n: usize,
        #[arg(short, long, default_value_t = 11)]
        k: usize,
        #[arg(short, long, default_value_t = 1)]
        w: usize,
        #[arg(short, long, value_enum, default_value_t = CodeType::Hamming)]
        code_type: CodeType,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
    },
    FiniaszSendrier {
        #[arg(short, long, default_value_t = 23)]
        n: usize,
        #[arg(short, long, default_value_t = 12)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(short, long, value_enum, default_value_t = CodeType::Random)]
        code_type: CodeType,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
    },
    LeeBrickell {
        #[arg(short, long, default_value_t = 23)]
        n: usize,
        #[arg(short, long, default_value_t = 12)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(short, long, value_enum, default_value_t = CodeType::Random)]
        code_type: CodeType,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
    },
    BallCollision {
        #[arg(short, long, default_value_t = 23)]
        n: usize,
        #[arg(short, long, default_value_t = 12)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(short, long, value_enum, default_value_t = CodeType::Random)]
        code_type: CodeType,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
    },
    Mmt {
        #[arg(short, long, default_value_t = 31)]
        n: usize,
        #[arg(short, long, default_value_t = 15)]
        k: usize,
        #[arg(short, long, default_value_t = 4)]
        w: usize,
        #[arg(short, long, value_enum, default_value_t = CodeType::Random)]
        code_type: CodeType,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
        #[arg(short, long, default_value_t = 2)]
        p: usize,
        #[arg(long, default_value_t = 256)]
        l1: usize, // Error split 1
        #[arg(long, default_value_t = 256)]
        l2: usize, // Error split 2
    },
    Bjmm {
        #[arg(short, long, default_value_t = 23)]
        n: usize,
        #[arg(short, long, default_value_t = 12)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(short, long, value_enum, default_value_t = CodeType::Random)]
        code_type: CodeType,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
    },
    Patterson {
        #[arg(short, long, default_value_t = 31)]
        n: usize,
        #[arg(short, long, default_value_t = 16)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(long, value_enum)]
        preset: Option<CodePreset>,
        // Code type is fixed to "goppa" for Patterson
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prange {
            n,
            k,
            w,
            code_type,
            preset,
        } => {
            let Some(code_params) =
                resolve_code_params(CodeParams { n, k, w, code_type }, preset, None, "prange")
            else {
                return;
            };
            run_algorithm(Algorithm::Prange, code_params, None);
        }
        Commands::Stern {
            n,
            k,
            w,
            code_type,
            preset,
        } => {
            let Some(code_params) =
                resolve_code_params(CodeParams { n, k, w, code_type }, preset, None, "stern")
            else {
                return;
            };
            run_algorithm(Algorithm::Stern, code_params, None);
        }
        Commands::FiniaszSendrier {
            n,
            k,
            w,
            code_type,
            preset,
        } => {
            let Some(code_params) = resolve_code_params(
                CodeParams { n, k, w, code_type },
                preset,
                None,
                "finiasz-sendrier",
            ) else {
                return;
            };
            run_algorithm(Algorithm::FiniaszSendrier, code_params, None);
        }
        Commands::LeeBrickell {
            n,
            k,
            w,
            code_type,
            preset,
        } => {
            let Some(code_params) = resolve_code_params(
                CodeParams { n, k, w, code_type },
                preset,
                None,
                "lee-brickell",
            ) else {
                return;
            };
            run_algorithm(Algorithm::LeeBrickell, code_params, None);
        }
        Commands::BallCollision {
            n,
            k,
            w,
            code_type,
            preset,
        } => {
            let Some(code_params) = resolve_code_params(
                CodeParams { n, k, w, code_type },
                preset,
                None,
                "ball-collision",
            ) else {
                return;
            };
            run_algorithm(Algorithm::BallCollision, code_params, None);
        }
        Commands::Mmt {
            n,
            k,
            w,
            code_type,
            preset,
            p,
            l1,
            l2,
        } => {
            let Some(code_params) =
                resolve_code_params(CodeParams { n, k, w, code_type }, preset, None, "mmt")
            else {
                return;
            };
            let partition_params = PartitionParams {
                p: Some(p),
                l1: Some(l1),
                l2: Some(l2),
            };
            run_algorithm(Algorithm::Mmt, code_params, Some(partition_params));
        }
        Commands::Bjmm {
            n,
            k,
            w,
            code_type,
            preset,
        } => {
            let Some(code_params) =
                resolve_code_params(CodeParams { n, k, w, code_type }, preset, None, "bjmm")
            else {
                return;
            };
            run_algorithm(Algorithm::Bjmm, code_params, None);
        }
        Commands::Patterson { n, k, w, preset } => {
            let Some(code_params) = resolve_code_params(
                CodeParams {
                    n,
                    k,
                    w,
                    code_type: CodeType::Goppa,
                },
                preset,
                Some(CodeType::Goppa),
                "patterson",
            ) else {
                return;
            };
            run_algorithm(Algorithm::Patterson, code_params, None);
        }
    }
}

fn resolve_code_params(
    mut params: CodeParams,
    preset: Option<CodePreset>,
    required_code_type: Option<CodeType>,
    command_label: &str,
) -> Option<CodeParams> {
    if let Some(preset) = preset {
        let preset_params = params.apply_preset(preset);
        if let Some(required) = required_code_type {
            if preset_params.code_type != required {
                eprintln!(
                    "Preset {} is incompatible with {} command (requires code type {}).",
                    preset, command_label, required
                );
                return None;
            }
        }
        println!(
            "Using preset {}: code-type={}, n={}, k={}, w={}",
            preset, preset_params.code_type, preset_params.n, preset_params.k, preset_params.w
        );
    }

    if let Some(required) = required_code_type {
        if params.code_type != required {
            eprintln!(
                "Command {} requires code type {} but {} was provided.",
                command_label, required, params.code_type
            );
            return None;
        }
    }

    Some(params)
}
