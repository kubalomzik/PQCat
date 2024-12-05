use clap::{Parser, Subcommand};
use std::time::Instant;

mod attacks;
mod codes;
mod utils;

use attacks::{ball_collision, lee_brickell, mmt, prange, stern};

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
        #[arg(short, long, default_value_t = 7)]
        n: usize, // Codeword length (number of bits)
        #[arg(short, long, default_value_t = 4)]
        k: usize, // Message length (number of bits)
        #[arg(short, long, default_value_t = 1)]
        w: usize, // Weight of the error vector (number of errors)
        #[arg(short, long, default_value = "hamming")]
        code_type: String, // Type of code: "random", "hamming", or "goppa"
    },
    Stern {
        #[arg(short, long, default_value_t = 7)]
        n: usize,
        #[arg(short, long, default_value_t = 4)]
        k: usize,
        #[arg(short, long, default_value_t = 1)]
        w: usize,
        #[arg(short, long, default_value = "hamming")]
        code_type: String,
    },
    LeeBrickell {
        #[arg(short, long, default_value_t = 7)]
        n: usize,
        #[arg(short, long, default_value_t = 4)]
        k: usize,
        #[arg(short, long, default_value_t = 1)]
        w: usize,
        #[arg(short, long, default_value = "hamming")]
        code_type: String,
    },
    BallCollision {
        #[arg(short, long, default_value_t = 7)]
        n: usize,
        #[arg(short, long, default_value_t = 4)]
        k: usize,
        #[arg(short, long, default_value_t = 1)]
        w: usize,
        #[arg(short, long, default_value = "hamming")]
        code_type: String,
    },
    Mmt {
        // fails to decode with these values
        #[arg(short, long, default_value_t = 7)]
        n: usize,
        #[arg(short, long, default_value_t = 4)]
        k: usize,
        #[arg(short, long, default_value_t = 1)]
        w: usize,
        #[arg(short, long, default_value = "hamming")]
        code_type: String,
        /*
        Parameter p (partitions) ~= log_2(n)
        - Smaller p reduces computational cost but may miss solutions
        - Larger p increases accuracy but grows computational cost exponentially
        */
        #[arg(short, long, default_value_t = 2)]
        p: usize,
        #[arg(long, default_value_t = 2)]
        l1: usize, // Error split 1
        #[arg(long, default_value_t = 2)]
        l2: usize, // Error split 2
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prange { n, k, w, code_type } => {
            let start = Instant::now();
            prange::run(n, k, w, code_type);
            let duration = start.elapsed();
            println!("Time {:?}", duration);
        }
        Commands::Stern { n, k, w, code_type } => {
            let start = Instant::now();
            stern::run(n, k, w, code_type);
            let duration = start.elapsed();
            println!("Time: {:?}", duration);
        }
        Commands::LeeBrickell { n, k, w, code_type } => {
            let start = Instant::now();
            lee_brickell::run(n, k, w, code_type);
            let duration = start.elapsed();
            println!("Time: {:?}", duration);
        }
        Commands::BallCollision { n, k, w, code_type } => {
            let start = Instant::now();
            ball_collision::run(n, k, w, code_type);
            let duration = start.elapsed();
            println!("Time: {:?}", duration);
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
            let start = Instant::now();
            mmt::run(n, k, w, code_type, p, l1, l2);
            let duration = start.elapsed();
            println!("Time: {:?}", duration);
        }
    }
}
