use crate::types::{BenchmarkConfig, BenchmarkResult, BenchmarkStats};
use csv::Writer;
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use sysinfo::System;

pub fn extract_time(output: &str) -> Option<u64> {
    let re = Regex::new(r"Time:\s*(\d+)\s*μs").unwrap();
    if let Some(captures) = re.captures(output) {
        if let Some(time_str) = captures.get(1) {
            return time_str.as_str().parse::<u64>().ok();
        }
    }
    None
}

pub fn extract_memory(output: &str) -> Option<u64> {
    let re = Regex::new(r"Peak memory:\s*(\d+)\s*KiB").unwrap();
    if let Some(captures) = re.captures(output) {
        if let Some(mem_str) = captures.get(1) {
            return mem_str.as_str().parse::<u64>().ok();
        }
    }
    None
}

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
        &config.algorithm_name, &config.code_type, config.n, config.k, config.w
    );

    let file = File::create(&csv_path).expect("Failed to create CSV file");
    let mut writer = Writer::from_writer(file);

    writer
        .write_record(["Run", "Time (μs)", "Memory (KiB)", "Result"])
        .expect("Failed to write CSV headers");

    let txt_filename = format!(
        "./results/txt/{}_{}_n{}_k{}_w{}.txt",
        &config.algorithm_name, &config.code_type, config.n, config.k, config.w
    );

    (writer, txt_filename)
}

pub fn execute_benchmark_runs(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    let mut results = Vec::with_capacity(config.runs);
    for run in 1..=config.runs {
        match execute_single_run(config, run) {
            Some(result) => {
                println!(
                    "Run {}/{}: Time = {} μs, Memory = {} KiB, Result = {}",
                    run,
                    config.runs,
                    result.duration,
                    result.memory,
                    if result.success { "success" } else { "fail" }
                );

                results.push(result);
            }
            None => continue, // Skip failed runs
        }
    }

    results
}

pub fn execute_single_run(config: &BenchmarkConfig, run: usize) -> Option<BenchmarkResult> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut cmd = build_command(config);
    cmd.stdout(std::process::Stdio::piped());

    let child = cmd.spawn().expect("Failed to spawn process");
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Run {} failed: {}", run, e);
            return None;
        }
    };

    if !output.status.success() {
        eprintln!(
            "Run {} failed: {}",
            run,
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let success = stdout_str.contains("success");

    let duration = extract_time(&stdout_str).unwrap_or(0);
    let memory = extract_memory(&stdout_str).unwrap_or(0);

    Some(BenchmarkResult {
        duration,
        memory,
        success,
    })
}

pub fn build_command(config: &BenchmarkConfig) -> Command {
    let mut cmd = Command::new("./target/release/pqcat");

    cmd.arg(config.algorithm_name.as_str());

    // Add common parameters
    cmd.arg("--n")
        .arg(config.n.to_string())
        .arg("--k")
        .arg(config.k.to_string())
        .arg("--w")
        .arg(config.w.to_string());

    // Add code type parameter except for Patterson (which is always Goppa)
    if config.algorithm_name != "patterson" {
        cmd.arg("--code-type").arg(&config.code_type);
    }

    // Add MMT-specific parameters if needed
    if config.algorithm_name == "mmt" {
        if let Some(p) = config.p {
            cmd.arg("--p").arg(p.to_string());
        }
        if let Some(l1) = config.l1 {
            cmd.arg("--l1").arg(l1.to_string());
        }
        if let Some(l2) = config.l2 {
            cmd.arg("--l2").arg(l2.to_string());
        }
    }

    cmd
}

pub fn calculate_statistics(results: &[BenchmarkResult]) -> BenchmarkStats {
    let completed_runs = results.len();

    if completed_runs == 0 {
        return BenchmarkStats {
            avg_time: 0.0,
            avg_memory: 0.0,
            success_rate: 0.0,
            successful_runs: 0,
            completed_runs: 0,
        };
    }

    let total_time: u64 = results.iter().map(|r| r.duration).sum();
    let total_memory: u64 = results.iter().map(|r| r.memory).sum();
    let successful_runs = results.iter().filter(|r| r.success).count();

    BenchmarkStats {
        avg_time: total_time as f64 / completed_runs as f64,
        avg_memory: total_memory as f64 / completed_runs as f64,
        success_rate: (successful_runs as f64 / completed_runs as f64) * 100.0,
        successful_runs,
        completed_runs,
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

    writeln!(txt_file, "Algorithm: {}", config.algorithm_name).unwrap();
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
    writeln!(txt_file, "Avg Time: {:.2} μs", stats.avg_time).unwrap();
    writeln!(txt_file, "Avg Memory: {:.2} KiB", stats.avg_memory).unwrap();
    writeln!(
        txt_file,
        "Success Rate: {:.2}% ({} of {} runs)",
        stats.success_rate, stats.successful_runs, stats.completed_runs
    )
    .unwrap();
}

pub fn print_summary(config: &BenchmarkConfig, stats: &BenchmarkStats) {
    println!("\nBENCHMARK SUMMARY");
    println!("Algorithm: {}", config.algorithm_name);
    println!(
        "Code: {} (n={}, k={}, w={})",
        config.code_type, config.n, config.k, config.w
    );
    println!("Avg Time: {:.2} μs", stats.avg_time);
    println!("Avg Memory: {:.2} KiB", stats.avg_memory);
    println!(
        "Success Rate: {:.2}% ({}/{})\n\n",
        stats.success_rate, stats.successful_runs, stats.completed_runs
    );
}
