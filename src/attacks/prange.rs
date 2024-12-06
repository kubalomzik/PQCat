use crate::codes::generate_code;
use crate::utils::{apply_errors, calculate_syndrome, generate_random_error_vector};
use ndarray::Array2;
use rand::seq::SliceRandom;
use std::time::Instant;

pub fn run(n: usize, k: usize, w: usize, code_type: String) {
    let (g, h) = generate_code(n, k, w, code_type);

    let error_vector = generate_random_error_vector(n, w); // Generate a random error vector of weight w
    let received_vector = apply_errors(&g.row(0).to_vec(), &error_vector); // Apply errors to a valid codeword

    println!("Original Error Vector: {:?}", error_vector);
    println!("Received Vector:       {:?}", received_vector);

    if let Some(decoded_error) = prange_algorithm(&received_vector, &h, w) {
        println!("Decoded Error Vector:  {:?}", decoded_error);
        println!("Result: success");
    } else {
        println!("Result: failure");
    }
}

pub fn prange_algorithm(received_vector: &[u8], h: &Array2<u8>, weight: usize) -> Option<Vec<u8>> {
    let start = Instant::now();

    let n = h.shape()[1]; // Length of the error vector
    let mut indices: Vec<usize> = (0..n).collect(); // All possible indices
    let received_syndrome = calculate_syndrome(received_vector, h);

    let mut loop_count = 0;

    while loop_count < 100 {
        // Shuffle and take the first `weight` indices as candidate positions for the error vector
        indices.shuffle(&mut rand::thread_rng());
        let chosen_indices = &indices[..weight];

        // Create a candidate error vector
        let mut candidate_error = vec![0; n];
        for &i in chosen_indices {
            candidate_error[i] = 1;
        }

        // Calculate the candidate syndrome: S = H * E^T
        let candidate_syndrome = calculate_syndrome(&candidate_error, h);

        // If the syndrome matches (i.e., it is zero), we found a valid error vector
        if candidate_syndrome == received_syndrome {
            let duration = start.elapsed().as_nanos();
            println!("Time: {} ns", duration);
            return Some(candidate_error);
        }
        loop_count += 1;
    }

    let duration = start.elapsed().as_nanos();
    println!("Time: {} ns", duration);

    None
}
