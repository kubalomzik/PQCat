use crate::algorithm_runner::run_single_benchmark;
use crate::code_generator::generate_code;
use crate::types::{Algorithm, BenchmarkConfig, BenchmarkResult, BenchmarkStats, CodeParams, PartitionParams};
use csv::Writer;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn ensure_results_directory() {
    if !Path::new("./results").exists() {
        fs::create_dir("./results").expect("Failed to create results directory");
        fs::create_dir("./results/txt").expect("Failed to create txt directory");
        fs::create_dir("./results/csv").expect("Failed to create csv directory");
    }
}

pub fn create_output_files(config: &BenchmarkConfig) -> (Writer<File>, String) {
    let csv_path = format!(
        "./results/csv/{}_{}_n{}_k{}_w{}.csv",
        config.algorithm, config.code_type, config.n, config.k, config.w
    );

    let file = File::create(&csv_path).expect("Failed to create CSV file");
    let mut writer = Writer::from_writer(file);

    writer
        .write_record(["Run", "Time (μs)", "Memory (KiB)", "Result", "Best Syndrome Distance"])
        .expect("Failed to write CSV headers");

    let txt_filename = format!(
        "./results/txt/{}_{}_n{}_k{}_w{}.txt",
        config.algorithm, config.code_type, config.n, config.k, config.w
    );

    (writer, txt_filename)
}

pub fn execute_benchmark_runs(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    let code_params = CodeParams {
        n: config.n,
        k: config.k,
        w: config.w,
        code_type: config.code_type,
    };

    let partition_params = if config.algorithm == Algorithm::Mmt {
        Some(PartitionParams {
            p: config.p,
            l1: config.l1,
            l2: config.l2,
        })
    } else {
        None
    };

    println!(
        "Generating {} code (n={}, k={}, w={})...",
        config.code_type, config.n, config.k, config.w
    );
    let (g, h, goppa_params) = generate_code(config.n, config.k, config.w, config.code_type);
    println!("Code generated. Starting {} runs...", config.runs);

    let mut results = Vec::with_capacity(config.runs);
    for run in 1..=config.runs {
        let result = run_single_benchmark(
            config.algorithm,
            &code_params,
            &partition_params,
            &g,
            &h,
            &goppa_params,
        );

        println!(
            "Run {}/{}: Time = {} μs, Memory = {} KiB, Result = {}, Best Syndrome Distance = {}",
            run,
            config.runs,
            result.duration,
            result.memory,
            if result.success { "success" } else { "fail" },
            result.best_syndrome_distance
        );

        results.push(result);
    }

    results
}

pub fn calculate_statistics(results: &[BenchmarkResult]) -> BenchmarkStats {
    let completed_runs = results.len();

    if completed_runs == 0 {
        return BenchmarkStats {
            median_time: 0.0,
            median_memory: 0.0,
            success_rate: 0.0,
            successful_runs: 0,
            completed_runs: 0,
            time_ci_lower: 0.0,
            time_ci_upper: 0.0,
            memory_ci_lower: 0.0,
            memory_ci_upper: 0.0,
            median_syndrome_distance: 0.0,
        };
    }

    // Extract and sort durations and memory values
    let mut durations: Vec<u64> = results.iter().map(|r| r.duration).collect();
    let mut memories: Vec<u64> = results.iter().map(|r| r.memory).collect();
    let mut syndrome_distances: Vec<u64> = results.iter().map(|r| r.best_syndrome_distance).collect();

    durations.sort();
    memories.sort();
    syndrome_distances.sort();

    // Calculate medians
    let median_time = if completed_runs % 2 == 0 {
        let mid = completed_runs / 2;
        (durations[mid - 1] + durations[mid]) as f64 / 2.0
    } else {
        durations[completed_runs / 2] as f64
    };

    let median_memory = if completed_runs % 2 == 0 {
        let mid = completed_runs / 2;
        (memories[mid - 1] + memories[mid]) as f64 / 2.0
    } else {
        memories[completed_runs / 2] as f64
    };

    let median_syndrome_distance = if completed_runs % 2 == 0 {
        let mid = completed_runs / 2;
        (syndrome_distances[mid - 1] + syndrome_distances[mid]) as f64 / 2.0
    } else {
        syndrome_distances[completed_runs / 2] as f64
    };

    // Calculate 95% confidence interval indices
    // For sample size n, approximately positions n/2 ± 1.96*sqrt(n)/2
    let lower_idx = (completed_runs / 2)
        .saturating_sub((1.96 * (completed_runs as f64).sqrt() / 2.0).round() as usize);

    let upper_idx = std::cmp::min(
        completed_runs / 2 + (1.96 * (completed_runs as f64).sqrt() / 2.0).round() as usize,
        completed_runs - 1,
    );

    // For exactly 100 runs, this would be indices 40 and 60 (0-indexed)
    let time_ci_lower = durations[lower_idx] as f64;
    let time_ci_upper = durations[upper_idx] as f64;
    let memory_ci_lower = memories[lower_idx] as f64;
    let memory_ci_upper = memories[upper_idx] as f64;

    // Calculate differences for error bars
    let time_ci_lower_diff = median_time - time_ci_lower;
    let time_ci_upper_diff = time_ci_upper - median_time;
    let memory_ci_lower_diff = median_memory - memory_ci_lower;
    let memory_ci_upper_diff = memory_ci_upper - median_memory;

    let successful_runs = results.iter().filter(|r| r.success).count();

    BenchmarkStats {
        median_time,
        median_memory,
        success_rate: (successful_runs as f64 / completed_runs as f64) * 100.0,
        successful_runs,
        completed_runs,
        time_ci_lower: time_ci_lower_diff,
        time_ci_upper: time_ci_upper_diff,
        memory_ci_lower: memory_ci_lower_diff,
        memory_ci_upper: memory_ci_upper_diff,
        median_syndrome_distance,
    }
}

pub fn write_results_to_file(
    writer: &mut Writer<File>,
    txt_filename: &str,
    config: &BenchmarkConfig,
    stats: &BenchmarkStats,
) {
    writer.flush().expect("Failed to flush CSV writer");

    let mut txt_file = File::create(txt_filename).expect("Failed to create TXT file");

    writeln!(txt_file, "Algorithm: {}", config.algorithm).unwrap();
    writeln!(txt_file, "Code Type: {}", config.code_type).unwrap();
    writeln!(
        txt_file,
        "Parameters: n={}, k={}, w={}",
        config.n, config.k, config.w
    )
    .unwrap();
    writeln!(
        txt_file,
        "Runs Completed: {}/{}",
        stats.completed_runs, config.runs
    )
    .unwrap();
    writeln!(
        txt_file,
        "Median Time: {:.2} μs (95% CI: {:.2} - {:.2})",
        stats.median_time, stats.time_ci_lower, stats.time_ci_upper
    )
    .unwrap();
    writeln!(
        txt_file,
        "Median Memory: {:.2} KiB (95% CI: {:.2} - {:.2})",
        stats.median_memory, stats.memory_ci_lower, stats.memory_ci_upper
    )
    .unwrap();
    writeln!(
        txt_file,
        "Success Rate: {:.2}% ({} of {} runs)",
        stats.success_rate, stats.successful_runs, stats.completed_runs
    )
    .unwrap();
    writeln!(
        txt_file,
        "Median Best Syndrome Distance: {:.1}",
        stats.median_syndrome_distance
    )
    .unwrap();
}

pub fn print_summary(config: &BenchmarkConfig, stats: &BenchmarkStats) {
    println!("\nBENCHMARK SUMMARY");
    println!("Algorithm: {}", config.algorithm);
    println!(
        "Code: {} (n={}, k={}, w={})",
        config.code_type, config.n, config.k, config.w
    );
    println!(
        "Median Time: {:.2} μs (95% CI: {:.2} - {:.2})",
        stats.median_time, stats.time_ci_lower, stats.time_ci_upper
    );
    println!(
        "Median Memory: {:.2} KiB (95% CI: {:.2} - {:.2})",
        stats.median_memory, stats.memory_ci_lower, stats.memory_ci_upper
    );
    println!(
        "Success Rate: {:.2}% ({}/{})\n",
        stats.success_rate, stats.successful_runs, stats.completed_runs
    );
    println!(
        "Median Best Syndrome Distance: {:.1}\n\n",
        stats.median_syndrome_distance
    );
}
