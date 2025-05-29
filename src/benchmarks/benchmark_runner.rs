use crate::benchmarks::benchmark_utils::{
    calculate_statistics, create_output_files, ensure_results_directory, execute_benchmark_runs,
    print_summary, write_results_to_file,
};
use crate::types::BenchmarkConfig;

#[allow(dead_code)]
pub fn run_benchmark(config: BenchmarkConfig) {
    // Setup phase
    ensure_results_directory();
    let (mut writer, txt_filename) = create_output_files(&config);

    // Execution phase
    let results = execute_benchmark_runs(&config);

    // Analysis and reporting phase
    let stats = calculate_statistics(&results);
    write_results_to_file(&mut writer, &txt_filename, &config, &stats);
    print_summary(&config, &stats);
}

// ==================== BATCH TEST FUNCTIONS ====================

#[allow(dead_code)]
pub fn run_all_tests_for_algorithm(algorithm: &str, runs: usize) {
    println!("Running all tests for algorithm: {}", algorithm);

    // Skip incompatible algorithm-code combinations
    match algorithm {
        "patterson" => run_all_goppa_tests(algorithm, runs),
        "mmt" => run_all_mmt_tests(runs),
        _ => {
            // For all other ISD algorithms
            run_all_hamming_tests(algorithm, runs);
            run_all_goppa_tests(algorithm, runs);
            run_all_qc_tests(algorithm, runs);
        }
    }
}

pub fn run_all_hamming_tests(algorithm: &str, runs: usize) {
    println!("Running Hamming code tests for {}", algorithm);

    // Test 1: Scaling code size
    for i in 0..4 {
        let config = BenchmarkConfig::hamming_scaling_size(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }

    // Test 2: Scaling error weight
    for i in 0..4 {
        let config = BenchmarkConfig::hamming_scaling_weight(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }
}

pub fn run_all_goppa_tests(algorithm: &str, runs: usize) {
    println!("Running Goppa code tests for {}", algorithm);

    // Test 1: Scaling code size
    for i in 0..4 {
        let config = BenchmarkConfig::goppa_scaling_size(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }

    // Test 2: Scaling error correction capability
    for i in 0..4 {
        let config = BenchmarkConfig::goppa_scaling_weight(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }
}

pub fn run_all_qc_tests(algorithm: &str, runs: usize) {
    println!("Running Quasi-Cyclic code tests for {}", algorithm);

    // Test 1: Scaling code size
    for i in 0..4 {
        let config = BenchmarkConfig::qc_scaling_size(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }

    // Test 2: Scaling error weight
    for i in 0..4 {
        let config = BenchmarkConfig::qc_scaling_weight(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }
}

// Run all MMT tests with special parameters
pub fn run_all_mmt_tests(runs: usize) {
    println!("Running MMT tests with special parameters");

    // MMT with Hamming codes
    for i in 0..4 {
        let params = [(7, 4, 1), (15, 11, 1), (31, 26, 1), (63, 57, 1)];
        let (n, k, w) = params[i];
        let config = BenchmarkConfig::mmt_config(n, k, w, "hamming").with_runs(runs);
        run_benchmark(config);
    }

    // MMT with Goppa codes
    for i in 0..4 {
        let params = [(16, 10, 2), (32, 22, 2), (64, 52, 2), (128, 112, 2)];
        let (n, k, w) = params[i];
        let config = BenchmarkConfig::mmt_config(n, k, w, "goppa").with_runs(runs);
        run_benchmark(config);
    }

    // MMT with Quasi-Cyclic codes
    for i in 0..4 {
        let params = [(30, 20, 2), (60, 40, 2), (90, 60, 2), (120, 80, 2)];
        let (n, k, w) = params[i];
        let config = BenchmarkConfig::mmt_config(n, k, w, "qc").with_runs(runs);
        run_benchmark(config);
    }
}

#[allow(dead_code)]
pub fn run_all_benchmarks(runs: usize) {
    let algorithms = [
        "prange",
        "stern",
        "lee_brickell",
        "ball_collision",
        "bjmm",
        "patterson",
    ];

    for &algorithm in &algorithms {
        run_all_tests_for_algorithm(algorithm, runs);
    }

    // Run MMT separately
    run_all_mmt_tests(runs);
}
