use crate::algorithms::algorithm_utils::{
    apply_errors, calculate_syndrome, generate_random_error_vector,
};
use crate::algorithms::metrics::{AlgorithmMetrics, print_metrics};
use crate::algorithms::{
    ball_collision, bjmm, finiasz_sendrier, lee_brickell, mmt, patterson, prange, stern,
};
use crate::code_generator::generate_code;
use crate::types::{Algorithm, BenchmarkResult, CodeParams, GoppaParams, PartitionParams};
use ndarray::Array2;

pub fn run_single_benchmark(
    algorithm: Algorithm,
    code_params: &CodeParams,
    partition_params: &Option<PartitionParams>,
    g: &Array2<u8>,
    h: &Array2<u8>,
    goppa_params: &Option<GoppaParams>,
) -> BenchmarkResult {
    let original_error = generate_random_error_vector(code_params.n, code_params.w);

    let received_vector = if algorithm != Algorithm::Mmt {
        let codeword = g.row(0).to_vec();
        apply_errors(&codeword, &original_error)
    } else {
        Vec::new()
    };

    let (decoded_err, metrics) = dispatch_algorithm(
        algorithm,
        code_params,
        partition_params,
        h,
        goppa_params,
        &original_error,
        &received_vector,
    );

    let success = match decoded_err {
        Some(ref decoded_error) => {
            if algorithm == Algorithm::Mmt {
                let decoded_weight = decoded_error.iter().filter(|&&bit| bit == 1).count();
                let s_original = calculate_syndrome(&original_error, h);
                let s_decoded = calculate_syndrome(decoded_error, h);
                s_original == s_decoded && decoded_weight <= code_params.w
            } else {
                let corrected: Vec<u8> = received_vector
                    .iter()
                    .zip(decoded_error.iter())
                    .map(|(&r, &e)| r ^ e)
                    .collect();
                let corrected_syndrome = calculate_syndrome(&corrected, h);
                let decoded_weight = decoded_error.iter().filter(|&&bit| bit == 1).count();
                corrected_syndrome.iter().all(|&x| x == 0) && decoded_weight <= code_params.w
            }
        }
        None => false,
    };

    BenchmarkResult {
        duration: metrics.time as u64,
        memory: (metrics.peak_memory / 1024) as u64,
        success,
        best_syndrome_distance: metrics.best_syndrome_distance as u64,
    }
}

fn dispatch_algorithm(
    algorithm: Algorithm,
    code_params: &CodeParams,
    partition_params: &Option<PartitionParams>,
    h: &Array2<u8>,
    goppa_params: &Option<GoppaParams>,
    original_error: &[u8],
    received_vector: &[u8],
) -> (Option<Vec<u8>>, AlgorithmMetrics) {
    match algorithm {
        Algorithm::Mmt => {
            if let Some(params) = partition_params {
                let p = params.p.unwrap_or(2);
                let l1 = params.l1.unwrap_or(256);
                let l2 = params.l2.unwrap_or(256);
                let s_vec = calculate_syndrome(original_error, h);
                let s_array = ndarray::Array1::from_vec(s_vec);
                mmt::run_mmt_algorithm(h, &s_array, code_params.n, code_params.w, p, l1, l2)
            } else {
                (
                    None,
                    AlgorithmMetrics {
                        time: 0,
                        peak_memory: 0,
                        best_syndrome_distance: 0,
                    },
                )
            }
        }
        Algorithm::Prange => prange::run_prange_algorithm(received_vector, h, code_params.w),
        Algorithm::Stern => stern::run_stern_algorithm(received_vector, h, code_params.w),
        Algorithm::FiniaszSendrier => {
            finiasz_sendrier::run_finiasz_sendrier_algorithm(received_vector, h, code_params.w)
        }
        Algorithm::LeeBrickell => lee_brickell::run_lee_brickell_algorithm(
            received_vector,
            h,
            code_params.n,
            code_params.w,
        ),
        Algorithm::BallCollision => ball_collision::run_ball_collision_algorithm(
            received_vector,
            h,
            code_params.n,
            code_params.w,
        ),
        Algorithm::Bjmm => {
            bjmm::run_bjmm_algorithm(received_vector, h, code_params.n, code_params.w)
        }
        Algorithm::Patterson => {
            let gp = goppa_params.as_ref().unwrap();
            patterson::run_patterson_algorithm(received_vector, h, gp, code_params.w)
        }
    }
}

pub fn run_algorithm(
    algorithm: Algorithm,
    code_params: CodeParams,
    partition_params: Option<PartitionParams>,
) {
    let (g, h, goppa_params) = generate_code(
        code_params.n,
        code_params.k,
        code_params.w,
        code_params.code_type,
    );

    let original_error = generate_random_error_vector(code_params.n, code_params.w);
    println!("Original Error Vector: {:?}", original_error);

    let received_vector = if algorithm != Algorithm::Mmt {
        let codeword = g.row(0).to_vec();
        let received_vector = apply_errors(&codeword, &original_error);
        println!("Received Vector:       {:?}", received_vector);
        received_vector
    } else {
        Vec::new()
    };

    let (decoded_err, algorithm_metrics) = dispatch_algorithm(
        algorithm,
        &code_params,
        &partition_params,
        &h,
        &goppa_params,
        &original_error,
        &received_vector,
    );

    // Print algorithm metrics regardless of success/failure
    print_metrics(&algorithm_metrics);

    match decoded_err {
        Some(decoded_error) => {
            println!("Decoded Error Vector:  {:?}", decoded_error);

            // Check if applying this error corrects the received vector to a valid codeword
            let corrected = received_vector
                .iter()
                .zip(decoded_error.iter())
                .map(|(&r, &e)| r ^ e)
                .collect::<Vec<u8>>();

            let corrected_syndrome = calculate_syndrome(&corrected, &h);

            // Check weight constraint
            let decoded_weight = decoded_error.iter().filter(|&&bit| bit == 1).count();

            if corrected_syndrome.iter().all(|&x| x == 0) && decoded_weight <= code_params.w {
                println!("Result: success (valid error vector found)");
                if decoded_error == original_error {
                    println!("[Note: Found the exact original error vector]");
                } else {
                    println!("[Note: Found an alternative valid error vector]");
                }
            } else {
                println!("Result: failure (invalid error vector)");
            }
        }
        None => {
            println!("Result: failure (algorithm did not find an error vector)");
        }
    }
}
