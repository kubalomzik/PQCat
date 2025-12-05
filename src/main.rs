use clap::{Parser, Subcommand};
mod algorithm_runner;
mod algorithms;
mod benchmarks;
mod code_generator;
mod codes;
mod types;

use algorithm_runner::run_algorithm;
use types::{Algorithm, CodeParams, CodeType, PartitionParams};

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
    },
    Patterson {
        #[arg(short, long, default_value_t = 31)]
        n: usize,
        #[arg(short, long, default_value_t = 16)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        // Code type is fixed to "goppa" for Patterson
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prange { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm(Algorithm::Prange, code_params, None);
        }
        Commands::Stern { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm(Algorithm::Stern, code_params, None);
        }
        Commands::LeeBrickell { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm(Algorithm::LeeBrickell, code_params, None);
        }
        Commands::BallCollision { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm(Algorithm::BallCollision, code_params, None);
        }
        Commands::Mmt {
            n,
            k,
            w,
            code_type,
            p,
            l1,
            l2,
        } => {
            let code_params = CodeParams { n, k, w, code_type };
            let partition_params = PartitionParams {
                p: Some(p),
                l1: Some(l1),
                l2: Some(l2),
            };
            run_algorithm(Algorithm::Mmt, code_params, Some(partition_params));
        }
        Commands::Bjmm { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm(Algorithm::Bjmm, code_params, None);
        }
        Commands::Patterson { n, k, w } => {
            let code_params = CodeParams {
                n,
                k,
                w,
                code_type: CodeType::Goppa,
            };
            run_algorithm(Algorithm::Patterson, code_params, None);
        }
    }
}
