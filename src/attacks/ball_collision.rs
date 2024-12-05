use crate::codes::generate_code;
use crate::utils::{apply_errors, calculate_syndrome, generate_random_error_vector};
use ndarray::Array2;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::{HashMap, HashSet};

pub fn run(n: usize, k: usize, w: usize, code_type: String) {
    let (g, h) = generate_code(n, k, w, code_type);

    let error_vector = generate_random_error_vector(n, w); // Generate a random error vector of weight w
    let received_vector = apply_errors(&g.row(0).to_vec(), &error_vector); // Apply errors to a valid codeword

    println!("Original Error Vector: {:?}", error_vector);
    println!("Received Vector:       {:?}", received_vector);

    if let Some(decoded_error) = ball_collision_algorithm(&received_vector, &h, n, w) {
        println!("Decoded Error Vector:  {:?}", decoded_error);
    } else {
        println!("Failed to decode!");
    }
}

pub fn ball_collision_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    n: usize,
    weight: usize,
) -> Option<Vec<u8>> {
    let t = weight / 2;

    // Generate random subsets and their syndromes
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

    // Calculate the syndrome of the received vector
    let target_syndrome = calculate_syndrome(received_vector, h);

    // Search for collision in the syndrome map
    let mut matching_subsets = HashSet::new();

    for (stored_syndrome, subset) in &syndrome_map {
        let mut complement_syndrome = target_syndrome.clone();
        for (i, &val) in stored_syndrome.iter().enumerate() {
            complement_syndrome[i] ^= val;
        }

        // If the complement_syndrome is already in the map, it's a collision
        if let Some(_) = syndrome_map.get(&complement_syndrome) {
            matching_subsets.insert(subset.clone());
        }
    }

    // Combine subsets to form the error vector if a collision is found
    for subset in matching_subsets.iter() {
        let mut error_vector = vec![0; n];
        for &i in subset {
            error_vector[i] = 1;
        }

        // Attempt to combine with another subset from the matching subsets
        for matching_subset in matching_subsets.iter() {
            if subset != matching_subset {
                // Combine the subsets to form the final error vector
                for &i in matching_subset {
                    error_vector[i] = 1;
                }

                // Check if the combined error vector satisfies the parity-check equation
                let syndrome = calculate_syndrome(&error_vector, h);
                if syndrome.iter().all(|&s| s == 0) {
                    return Some(error_vector);
                }
            }
        }
    }

    None
}
