use crate::utils::{
    apply_errors, calculate_syndrome, generate_random_code, generate_random_error_vector,
    subset_generator,
};
use ndarray::Array2;
use rand::seq::SliceRandom;
use std::collections::HashMap;

pub fn run(n: usize, k: usize, w: usize) {
    let (g, h) = generate_random_code(n, k); // Generator matrix G and parity-check matrix H

    let error_vector = generate_random_error_vector(n, w); // Generate a random error vector of weight w
    let received_vector = apply_errors(&g.row(0).to_vec(), &error_vector); // Apply errors to a valid codeword

    println!("Original Error Vector: {:?}", error_vector);
    println!("Received Vector: {:?}", received_vector);

    if let Some(decoded_error) = sterns_algorithm(&received_vector, &h, w) {
        println!("Decoded Error Vector: {:?}", decoded_error);
    } else {
        println!("Failed to decode!");
    }
}

pub fn sterns_algorithm(received_vector: &[u8], h: &Array2<u8>, weight: usize) -> Option<Vec<u8>> {
    let n = h.shape()[1];
    let m = n / 2;

    // Step 1: Split the indices into two sets
    let indices: Vec<usize> = (0..n).collect();
    let mut left_indices = indices[..m].to_vec();
    let mut right_indices = indices[m..].to_vec();

    // Shuffle to add randomness to bare closer resemblance to the probabilistic nature of Stern's algorithm
    left_indices.shuffle(&mut rand::thread_rng());
    right_indices.shuffle(&mut rand::thread_rng());

    // Step 2: Compute initial syndrome
    let target_syndrome = calculate_syndrome(received_vector, h);

    // Step 3: Create hash maps for subsets
    let mut left_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
    let mut right_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();

    // Populate the left map
    for subset in subset_generator(&left_indices, weight / 2) {
        let mut candidate_error = vec![0; n];
        for &i in &subset {
            candidate_error[i] = 1;
        }
        let syndrome = calculate_syndrome(&candidate_error, h);
        left_map.insert(syndrome.clone(), subset);
    }

    // Populate the right map
    for subset in subset_generator(&right_indices, weight / 2) {
        let mut candidate_error = vec![0; n];
        for &i in &subset {
            candidate_error[i] = 1;
        }
        let syndrome = calculate_syndrome(&candidate_error, h);
        right_map.insert(syndrome.clone(), subset);
    }

    // Step 4: Find matching syndromes in both maps
    for (left_syndrome, left_subset) in &left_map {
        let mut complement_syndrome = target_syndrome.clone();
        for (i, &val) in left_syndrome.iter().enumerate() {
            complement_syndrome[i] ^= val;
        }
        if let Some(right_subset) = right_map.get(&complement_syndrome) {
            // Combine the subsets to form the error vector
            let mut error_vector = vec![0; n];
            for &i in left_subset {
                error_vector[i] = 1;
            }
            for &i in right_subset {
                error_vector[i] = 1;
            }
            return Some(error_vector);
        }
    }

    // If no match found, return None
    None
}
