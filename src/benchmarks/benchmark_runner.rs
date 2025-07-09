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

// TODO: untangle MMT-specific tests so that there's less boilerplate
pub fn run_all_mmt_tests(runs: usize) {
    run_all_hamming_tests_mmt(runs);
    run_all_goppa_tests_mmt(runs);
    run_all_qc_tests_mmt(runs);
}

pub fn run_all_hamming_tests_mmt(runs: usize) {
    // Test 1: Scaling code size
    for i in 0..4 {
        let config = BenchmarkConfig::hamming_scaling_size(i)
            .with_algorithm("mmt")
            .with_runs(runs)
            .with_mmt_params(2, 256, 256); // Default MMT parameters
        run_benchmark(config);
    }

    // Test 2: Scaling error weight
    for i in 0..4 {
        let config = BenchmarkConfig::hamming_scaling_weight(i)
            .with_algorithm("mmt")
            .with_runs(runs)
            .with_mmt_params(2, 256, 256); // Default MMT parameters
        run_benchmark(config);
    }
}

// MMT-specific version of Goppa tests that includes MMT parameters
pub fn run_all_goppa_tests_mmt(runs: usize) {
    // Test 1: Scaling code size
    for i in 0..4 {
        let config = BenchmarkConfig::goppa_scaling_size(i)
            .with_algorithm("mmt")
            .with_runs(runs)
            .with_mmt_params(2, 256, 256); // Default MMT parameters
        run_benchmark(config);
    }

    // Test 2: Scaling error correction capability
    for i in 0..4 {
        let config = BenchmarkConfig::goppa_scaling_weight(i)
            .with_algorithm("mmt")
            .with_runs(runs)
            .with_mmt_params(2, 256, 256); // Default MMT parameters
        run_benchmark(config);
    }
}

// MMT-specific version of QC tests that includes MMT parameters
pub fn run_all_qc_tests_mmt(runs: usize) {
    // Test 1: Scaling code size
    for i in 0..4 {
        let config = BenchmarkConfig::qc_scaling_size(i)
            .with_algorithm("mmt")
            .with_runs(runs)
            .with_mmt_params(2, 256, 256); // Default MMT parameters
        run_benchmark(config);
    }

    // Test 2: Scaling error weight
    for i in 0..4 {
        let config = BenchmarkConfig::qc_scaling_weight(i)
            .with_algorithm("mmt")
            .with_runs(runs)
            .with_mmt_params(2, 256, 256); // Default MMT parameters
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

#[allow(dead_code)]
pub fn run_real_world_test(algorithm: &str, runs: usize) {
    for i in 0..2 {
        // Just test first two levels as higher ones might be too slow
        let config = BenchmarkConfig::real_world_goppa(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);

        let config = BenchmarkConfig::real_world_qc(i)
            .with_algorithm(algorithm)
            .with_runs(runs);
        run_benchmark(config);
    }
}
