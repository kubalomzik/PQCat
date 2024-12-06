use crate::codes::generate_code;
use crate::utils::{apply_errors, calculate_syndrome, generate_random_error_vector};
use ndarray::Array2;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::time::Instant;

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
    let start = Instant::now();

    let t = weight / 2;
    let max_subsets = 1000;
    let mut rng = thread_rng();
    let indices: Vec<usize> = (0..n).collect();
    let mut syndrome_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();

    // Generate random subsets of size t and store their syndromes
    for _ in 0..max_subsets {
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

    // Search for complement syndromes in the syndrome map
    for (stored_syndrome, subset_a) in &syndrome_map {
        let mut complement_syndrome = target_syndrome.clone();
        for (i, &val) in stored_syndrome.iter().enumerate() {
            complement_syndrome[i] ^= val;
        }

        // Check if complement_syndrome exists in the map
        if let Some(subset_b) = syndrome_map.get(&complement_syndrome) {
            // Combine subsets to form a candidate error vector
            let mut error_vector = vec![0; n];
            for &i in subset_a {
                error_vector[i] = 1;
            }
            for &i in subset_b {
                error_vector[i] ^= 1; // XOR to avoid duplicate indices
            }

            // Validate the error vector
            if error_vector.iter().filter(|&&bit| bit == 1).count() == weight {
                let syndrome = calculate_syndrome(&error_vector, h);
                if syndrome.iter().all(|&s| s == 0) {
                    let duration = start.elapsed().as_nanos();
                    println!("Time: {} ns", duration);
                    return Some(error_vector);
                }
            }
        }
    }

    let duration = start.elapsed().as_nanos();
    println!("Time: {} ns", duration);

    None
}
