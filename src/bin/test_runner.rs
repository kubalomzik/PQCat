#[allow(unused_imports)]
use pqcat::benchmarks::benchmark_runner::{
    run_all_benchmarks, run_all_goppa_tests, run_all_hamming_tests, run_all_mmt_tests,
    run_all_qc_tests, run_all_tests_for_algorithm, run_benchmark, run_real_world_test,
};
#[allow(unused_imports)]
use pqcat::types::BenchmarkConfig;

fn main() {
    // Option 1: Run a single test case (e.g scaling size of the Hamming code)
    // let config = BenchmarkConfig::hamming_scaling_size(1).with_algorithm("prange").with_runs(100);
    // run_benchmark(config);

    // Option 2: Run all tests for one algorithm (scaling size, scaling weight for Hamming, Goppa and QC)
    // run_all_tests_for_algorithm("prange", 100);

    // Option 3: Run all code tests for all algorithms
    // let algorithms = ["prange", "stern", "lee_brickell", "ball_collision", "bjmm"];
    // for &alg in &algorithms {
    //     run_all_hamming_tests(alg, 100);
    // }

    // Option 4: Run everything
    // run_all_benchmarks(100);

    // Option 5: Run real-world parameters (more in-line with practical use cases) for specific algorithms
    // run_real_world_test("prange", 100); - or combine with e.g. option 3 to run for multiple chose algorithms

    let algorithms = ["bjmm"];
    for &alg in &algorithms {
        run_real_world_test(alg, 1);
    }
}
