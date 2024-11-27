use ndarray::{Array1, Array2};

use crate::utils::{
    calculate_weight, column_match, compute_h_tilde, extract_q_i, generate_random_code,
    generate_random_syndrome, support_subset, validate_syndrome,
};

pub fn run(n: usize, k: usize, w: usize, p: usize, ell1: usize, ell2: usize) {
    let l = ell1 + ell2; // Total number of errors

    // Generate random code matrices (G and H)
    let (_g, h) = generate_random_code(n, k);

    // Generate a random syndrome of appropriate length
    let s = generate_random_syndrome(&h, w);

    let result = may_meurer_thomae_algorithm(&h, &s, w, p, l, ell1, ell2);

    match result {
        Some(error_vector) => {
            println!("Decoded Error Vector: {:?}", error_vector);
        }
        None => {
            println!("Failed to decode!");
        }
    }
}

pub fn may_meurer_thomae_algorithm(
    h: &Array2<u8>,
    s: &Array1<u8>,
    weight: usize,
    p: usize,
    l: usize,
    ell1: usize,
    ell2: usize,
) -> Option<Vec<u8>> {
    assert_eq!(l, ell1 + ell2);
    let n = h.shape()[1];
    let max_iterations = 100; // Limit iterations

    for _ in 0..max_iterations {
        // Step 1: Compute H-tilda
        let h_tilde = compute_h_tilde(h, p);

        // Step 2: Find column subsets matching conditions
        let solutions = column_match(&h_tilde, p, l, ell1, ell2);

        for i in solutions {
            let q_i = extract_q_i(&h_tilde, &i);

            // Step 3: Check weight condition
            if calculate_weight(&q_i.iter().cloned().collect::<Vec<u8>>()) + i.len() == weight {
                // Construct error vector
                let mut e_tilde = vec![0; n];
                for &index in &i {
                    e_tilde[index] = 1;
                }
                for j in support_subset(&q_i) {
                    e_tilde[j] = 1;
                }

                // Validate syndrome
                if validate_syndrome(&e_tilde, s, h) {
                    return Some(e_tilde);
                }
            }
        }
    }

    None // Decoding failed
}
