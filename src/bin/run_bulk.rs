use csv::Writer;
use regex::Regex;
use std::fs::File;
use std::process::Command;
use sysinfo::System;

fn extract_time(output: &str) -> Option<u64> {
    let re = Regex::new(r"Time:\s*(\d+)\s*μs").unwrap();

    if let Some(captures) = re.captures(output) {
        if let Some(time_str) = captures.get(1) {
            return time_str.as_str().parse::<u64>().ok();
        }
    }
    None
}

fn extract_memory(output: &str) -> Option<u64> {
    let re = Regex::new(r"Peak memory:\s*(\d+)\s*KiB").unwrap();

    if let Some(captures) = re.captures(output) {
        if let Some(mem_str) = captures.get(1) {
            return mem_str.as_str().parse::<u64>().ok();
        }
    }
    None
}

fn main() {
    let runs = 100;
    let algorithm_name = "lee-brickell";
    let n = 31;
    let k = 26;
    let w = 3;
    let code_type = "hamming";
    let _p: Option<usize> = None;
    let _l1: Option<usize> = None;
    let _l2: Option<usize> = None;

    let file = File::create(format!(
        "./results/{}_n{}_k{}_w{}_{}.csv",
        algorithm_name, n, k, w, code_type
    ))
    .expect("Failed to create CSV file");
    let mut writer = Writer::from_writer(file);

    writer
        .write_record(["Run", "Time (μs)", "Memory (KiB)", "Result"])
        .expect("Failed to write CSV headers");

    for run in 1..=runs {
        let mut sys = System::new_all();
        sys.refresh_all();

        let child = Command::new("./target/release/pqcat")
            .arg(algorithm_name)
            .arg("--n")
            .arg(n.to_string())
            .arg("--k")
            .arg(k.to_string())
            .arg("--w")
            .arg(w.to_string())
            .arg("--code-type")
            .arg(code_type)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let output = child
            .wait_with_output()
            .expect("Failed to wait for process");

        if !output.status.success() {
            eprintln!(
                "Run {} failed: {}",
                run,
                String::from_utf8_lossy(&output.stderr)
            );
            continue;
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let result = if stdout_str.contains("success") {
            "success"
        } else {
            "fail"
        };

        let duration = extract_time(&stdout_str).unwrap_or(0);
        let memory = extract_memory(&stdout_str).unwrap_or(0);

        println!(
            "Run {}: Time = {} μs, Memory = {} KiB, Result = {}\n",
            run, duration, memory, result
        );

        // Write to CSV
        writer
            .write_record(&[
                run.to_string(),
                duration.to_string(),
                memory.to_string(),
                result.to_string(),
            ])
            .expect("Failed to write to CSV");
    }

    writer.flush().expect("Failed to flush CSV writer");
    println!(
        "Results saved in ./results/{}_n{}_k{}_w{}_{}.csv",
        algorithm_name, n, k, w, code_type
    );
}
