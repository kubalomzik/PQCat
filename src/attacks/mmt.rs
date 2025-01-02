use ndarray::{Array1, Array2};

use crate::codes::generate_code;
use crate::utils::{
    calculate_weight, column_match, compute_h_tilde, extract_q_i, generate_random_syndrome,
    validate_syndrome,
};

pub fn run(n: usize, k: usize, w: usize, code_type: String, p: usize, ell1: usize, ell2: usize) {
    assert!(ell1 + ell2 == w, "Sum of l1 and l2 must be equal to weight");
    let (_g, h) = generate_code(n, k, w, code_type);

    /*
    This algorithm, unlike other available here, does not work directly with the corrupted codeword.
    Instead, it operates in syndrome space so there's no need to generate error vector or apply errors.
     */
    let s = generate_random_syndrome(&h, w);

    if let Some(decoded_error) = may_meurer_thomae_algorithm(&h, &s, n, w, p, ell1, ell2) {
        println!("Decoded Error Vector:  {:?}", decoded_error);
        println!("Result: success");
    } else {
        println!("Result: failure");
    }
}

pub fn may_meurer_thomae_algorithm(
    h: &Array2<u8>,
    s: &Array1<u8>,
    n: usize,
    w: usize,
    p: usize,
    ell1: usize,
    ell2: usize,
) -> Option<Vec<u8>> {
    let max_iterations = 100;

    for _ in 0..max_iterations {
        // Compute H-tilde
        let h_tilde = compute_h_tilde(h, p);

        // Find column subsets matching conditions
        let solutions = column_match(&h_tilde, p, w, ell1, ell2);

        for i in solutions {
            let q_i = extract_q_i(&h_tilde, &i);

            // Check weight condition
            if calculate_weight(q_i.as_slice().unwrap()) + i.len() == w {
                // Construct error vector
                let mut e_tilde = vec![0; n];
                for &index in &i {
                    e_tilde[index] = 1;
                }
                for j in q_i {
                    e_tilde[j as usize] = 1;
                }

                if validate_syndrome(&e_tilde, s, h) {
                    return Some(e_tilde);
                }
            }
        }
    }

    None
}
