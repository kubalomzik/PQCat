use crate::attacks::attack_utils::{
    apply_errors, calculate_syndrome, generate_random_error_vector,
};
use crate::attacks::{ball_collision, lee_brickell, mmt, prange, stern};
use crate::code_generator::generate_code;
use crate::types::{CodeParams, PartitionParams};

pub fn run_algorithm(
    algorithm_name: &str,
    code_params: CodeParams,
    partition_params: Option<PartitionParams>,
) {
    let (g, h) = generate_code(
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
                _ => None,
            }
        }
    };

    println!("Original Error Vector: {:?}", original_error);
    if let Some(decoded) = decoded_error {
        println!("Decoded Error Vector:  {:?}", decoded);
        if decoded == original_error {
            println!("Result: success (correct error vector)");
        } else {
            println!("Result: failure (wrong error vector)");
        }
    } else {
        println!("Result: failure (no solution found)");
    }
}
