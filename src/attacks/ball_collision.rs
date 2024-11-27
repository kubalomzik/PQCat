use crate::utils::{
    apply_errors, calculate_syndrome, generate_random_code, generate_random_error_vector,
};
use ndarray::Array2;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::{HashMap, HashSet};

pub fn run(n: usize, k: usize, w: usize) {
    let (g, h) = generate_random_code(n, k); // Generator matrix G and parity-check matrix H

    let error_vector = generate_random_error_vector(n, w); // Generate a random error vector of weight w
    let received_vector = apply_errors(&g.row(0).to_vec(), &error_vector); // Apply errors to a valid codeword

    println!("Original Error Vector: {:?}", error_vector);
    println!("Received Vector: {:?}", received_vector);

    if let Some(decoded_error) = ball_collision_algorithm(&received_vector, &h, w) {
        println!("Decoded Error Vector: {:?}", decoded_error);
    } else {
        println!("Failed to decode!");
    }
}

pub fn ball_collision_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    weight: usize,
) -> Option<Vec<u8>> {
    let n = h.shape()[1]; // Length of the codeword
    let t = weight / 2; // Ensure t handles both odd and even weights

    // Step 1: Generate random subsets and their syndromes
    let mut rng = thread_rng();
    let indices: Vec<usize> = (0..n).collect();
    let mut syndrome_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();

    for _ in 0..(1 << t) {
        let subset: Vec<usize> = indices.choose_multiple(&mut rng, t).cloned().collect();

        let mut candidate_error = vec![0; n];
        for &i in &subset {
            candidate_error[i] = 1;
        }

        let syndrome = calculate_syndrome(&candidate_error, h);
        syndrome_map.insert(syndrome.clone(), subset);
    }

    // Step 2: Calculate the syndrome of the received vector
    let target_syndrome = calculate_syndrome(received_vector, h);

    // Step 3: Search for collision in the syndrome map
    // Use HashSet for faster collision detection
    let mut matching_subsets = HashSet::new();

    for (stored_syndrome, subset) in &syndrome_map {
        let mut complement_syndrome = target_syndrome.clone();
        for (i, &val) in stored_syndrome.iter().enumerate() {
            complement_syndrome[i] ^= val;
        }

        // If the complement_syndrome is already in the map, it's a collision
        if syndrome_map.contains_key(&complement_syndrome) {
            matching_subsets.insert(subset.clone());
        }
    }

    // Step 4: Combine subsets to form the error vector if a collision is found
    for subset in matching_subsets.iter() {
        // Borrow the subsets with `.iter()`
        let mut error_vector = vec![0; n];
        for &i in subset {
            error_vector[i] = 1;
        }

        // Attempt to find a matching subset that forms the full error vector
        for matching_subset in matching_subsets.iter() {
            if subset != matching_subset {
                // Combine the subsets to form the final error vector
                for &i in matching_subset {
                    error_vector[i] = 1;
                }
                return Some(error_vector); // Return the combined error vector
            }
        }
    }

    None // Return None if no collision is found
}
