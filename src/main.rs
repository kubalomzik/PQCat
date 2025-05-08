use clap::{Parser, Subcommand};
mod algorithm_runner;
mod attacks;
mod code_generator;
mod codes;
mod types;

use algorithm_runner::run_algorithm;
use types::{CodeParams, PartitionParams};

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
        #[arg(short, long, default_value = "hamming")]
        code_type: String, // Type of code: "random", "hamming", or "goppa"
    },
    Stern {
        #[arg(short, long, default_value_t = 15)]
        n: usize,
        #[arg(short, long, default_value_t = 11)]
        k: usize,
        #[arg(short, long, default_value_t = 1)]
        w: usize,
        #[arg(short, long, default_value = "hamming")]
        code_type: String,
    },
    LeeBrickell {
        #[arg(short, long, default_value_t = 23)]
        n: usize,
        #[arg(short, long, default_value_t = 12)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(short, long, default_value = "random")]
        code_type: String,
    },
    BallCollision {
        #[arg(short, long, default_value_t = 23)]
        n: usize,
        #[arg(short, long, default_value_t = 12)]
        k: usize,
        #[arg(short, long, default_value_t = 3)]
        w: usize,
        #[arg(short, long, default_value = "random")]
        code_type: String,
    },
    Mmt {
        // fails to decode with these values
        #[arg(short, long, default_value_t = 31)]
        n: usize,
        #[arg(short, long, default_value_t = 15)]
        k: usize,
        #[arg(short, long, default_value_t = 4)]
        w: usize,
        #[arg(short, long, default_value = "random")]
        code_type: String,
        #[arg(short, long, default_value_t = 2)]
        p: usize,
        #[arg(long, default_value_t = 256)]
        l1: usize, // Error split 1
        #[arg(long, default_value_t = 256)]
        l2: usize, // Error split 2
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prange { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm("prange", code_params, None);
        }
        Commands::Stern { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm("stern", code_params, None);
        }
        Commands::LeeBrickell { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm("lee_brickell", code_params, None);
        }
        Commands::BallCollision { n, k, w, code_type } => {
            let code_params = CodeParams { n, k, w, code_type };
            run_algorithm("ball_collision", code_params, None);
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
            run_algorithm("mmt", code_params, Some(partition_params));
        }
    }
}
