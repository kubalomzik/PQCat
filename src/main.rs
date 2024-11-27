use clap::{Parser, Subcommand};
use std::time::Instant;

mod attacks;
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
        #[arg(short, long, default_value_t = 6)]
        n: usize, // Codeword length (number of bits)
        #[arg(short, long, default_value_t = 3)]
        k: usize, // Message length (number of bits)
        #[arg(short, long, default_value_t = 2)]
        w: usize, // Weight of the error vector (number of errors)
    },
    Stern {
        #[arg(short, long, default_value_t = 6)]
        n: usize,
        #[arg(short, long, default_value_t = 3)]
        k: usize,
        #[arg(short, long, default_value_t = 2)]
        w: usize,
    },
    LeeBrickell {
        #[arg(short, long, default_value_t = 6)]
        n: usize,
        #[arg(short, long, default_value_t = 3)]
        k: usize,
        #[arg(short, long, default_value_t = 2)]
        w: usize,
    },
    BallCollision {
        #[arg(short, long, default_value_t = 6)]
        n: usize,
        #[arg(short, long, default_value_t = 3)]
        k: usize,
        #[arg(short, long, default_value_t = 2)]
        w: usize,
    },
    Mmt {
        #[arg(short, long, default_value_t = 6)]
        n: usize,
        #[arg(short, long, default_value_t = 3)]
        k: usize,
        #[arg(short, long, default_value_t = 2)]
        w: usize,
        #[arg(short, long, default_value_t = 2)]
        p: usize, // Parameter p (partitions)
        #[arg(long, default_value_t = 2)]
        l1: usize, // Parameter l1
        #[arg(long, default_value_t = 2)]
        l2: usize, // Parameter l2
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prange { n, k, w } => {
            let start = Instant::now();
            prange::run(n, k, w);
            let duration = start.elapsed();
            println!("Prange's algorithm completed in {:?}", duration);
        }
        Commands::Stern { n, k, w } => {
            let start = Instant::now();
            stern::run(n, k, w);
            let duration = start.elapsed();
            println!("Stern's algorithm completed in {:?}", duration);
        }
        Commands::LeeBrickell { n, k, w } => {
            let start = Instant::now();
            lee_brickell::run(n, k, w);
            let duration = start.elapsed();
            println!("Lee-Brickell algorithm completed in {:?}", duration);
        }
        Commands::BallCollision { n, k, w } => {
            let start = Instant::now();
            ball_collision::run(n, k, w);
            let duration = start.elapsed();
            println!("Ball Collision algorithm completed in {:?}", duration);
        }
        Commands::Mmt { n, k, w, p, l1, l2 } => {
            let start = Instant::now();
            mmt::run(n, k, w, p, l1, l2);
            let duration = start.elapsed();
            println!("MMT algorithm completed in {:?}", duration);
        }
    }
}
