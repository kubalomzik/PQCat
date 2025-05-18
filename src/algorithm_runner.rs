use crate::algorithms::algorithm_utils::{
    apply_errors, calculate_syndrome, generate_random_error_vector,
};
use crate::algorithms::{ball_collision, bjmm, lee_brickell, mmt, patterson, prange, stern};
use crate::code_generator::generate_code;
use crate::types::{CodeParams, PartitionParams};

pub fn run_algorithm(
    algorithm_name: &str,
    code_params: CodeParams,
    partition_params: Option<PartitionParams>,
) {
    let (g, h, goppa_params) = generate_code(
        code_params.n,
        code_params.k,
        code_params.w,
        code_params.code_type.clone(),
    );

    let original_error = generate_random_error_vector(code_params.n, code_params.w); // Generate a random error vector of weight w

    let decoded_error = match algorithm_name {
        "mmt" => {
            /*
            This algorithm, unlike other available here, does not work directly with the corrupted codeword.
            Instead, it operates in syndrome space so there's no need to generate error vector or apply errors.
             */
            if let Some(params) = &partition_params {
                let p = params.p.unwrap_or(2);
                let l1 = params.l1.unwrap_or(256);
                let l2 = params.l2.unwrap_or(256);
                let s_vec = calculate_syndrome(&original_error, &h);
                let s_array = ndarray::Array1::from_vec(s_vec);
                mmt::run_mmt_algorithm(&h, &s_array, code_params.n, code_params.w, p, l1, l2)
            } else {
                eprintln!("MMT algorithm requires partition parameters");
                None
            }
        }
        _ => {
            let received_vector = apply_errors(&g.row(0).to_vec(), &original_error); // Apply errors to a valid codeword

            println!("Received Vector:       {:?}", received_vector);

            match algorithm_name {
                "prange" => prange::run_prange_algorithm(&received_vector, &h, code_params.w),
                "stern" => stern::run_stern_algorithm(&received_vector, &h, code_params.w),
                "lee_brickell" => lee_brickell::run_lee_brickell_algorithm(
                    &received_vector,
                    &h,
                    code_params.n,
                    code_params.w,
                ),
                "ball_collision" => ball_collision::run_ball_collision_algorithm(
                    &received_vector,
                    &h,
                    code_params.n,
                    code_params.w,
                ),
                "bjmm" => {
                    bjmm::run_bjmm_algorithm(&received_vector, &h, code_params.n, code_params.w)
                }
                "patterson" => {
                    let goppa_params = goppa_params.unwrap();
                    patterson::run_patterson_algorithm(&received_vector, &h, &goppa_params)
                }
                _ => None,
            }
        }
    };

    if let Some(decoded) = decoded_error {
        println!("Decoded Error Vector:  {:?}", decoded);

        // Check syndrome correctness
        let decoded_syndrome = calculate_syndrome(&decoded, &h);
        let original_syndrome = calculate_syndrome(&original_error, &h);

        // Check weight constraint
        let decoded_weight = decoded.iter().filter(|&&bit| bit == 1).count();

        if decoded_syndrome == original_syndrome && decoded_weight <= code_params.w {
            println!("Result: success (valid error vector found)");
            if decoded == original_error {
                println!("[Note: Found the exact original error vector]");
            } else {
                println!("[Note: Found an alternative valid error vector]");
            }
        } else {
            println!("Result: failure (invalid error vector)");
        }
    }
}
