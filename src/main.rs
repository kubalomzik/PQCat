use clap::{Parser, Subcommand};
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

fn run_algorithm(
    n: usize,
    k: usize,
    w: usize,
    code_type: String,
    algorithm_name: &str,
    p: Option<usize>,
    l1: Option<usize>,
    l2: Option<usize>,
) {
    match algorithm_name {
        "prange" => prange::run(n, k, w, code_type),
        "stern" => stern::run(n, k, w, code_type),
        "lee_brickell" => lee_brickell::run(n, k, w, code_type),
        "ball_collision" => ball_collision::run(n, k, w, code_type),
        "mmt" => mmt::run(n, k, w, code_type, p.unwrap(), l1.unwrap(), l2.unwrap()),
        _ => return,
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prange { n, k, w, code_type } => {
            run_algorithm(n, k, w, code_type, "prange", None, None, None);
        }
        Commands::Stern { n, k, w, code_type } => {
            run_algorithm(n, k, w, code_type, "stern", None, None, None);
        }
        Commands::LeeBrickell { n, k, w, code_type } => {
            run_algorithm(n, k, w, code_type, "lee_brickell", None, None, None);
        }
        Commands::BallCollision { n, k, w, code_type } => {
            run_algorithm(n, k, w, code_type, "ball_collision", None, None, None);
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
            run_algorithm(
                n,
                k,
                w,
                code_type,
                "ball_collision",
                Some(p),
                Some(l1),
                Some(l2),
            );
        }
    }
}
