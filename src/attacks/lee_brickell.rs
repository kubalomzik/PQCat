use crate::attacks::attack_utils::{calculate_syndrome, generate_subsets};
use ndarray::Array2;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::time::Instant;

pub fn run_lee_brickell_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    n: usize,
    weight: usize,
) -> Option<Vec<u8>> {
    let start = Instant::now();

    let m = n / 2 + (n % 2);

    /*
    This algorithm, unlike Stern's or Prange's, generally requires the weight to be even.
    However, for maximum compatiblity with provided values this check is omitted.
    Consider always providing even weights to ensure proper use of this algorithm.

    if weight % 2 != 0 {
        println!("Weight must be even for Lee-Brickell algorithm.");
        return None;
    }
    */

    // Split indices into left and right halves
    let indices: Vec<usize> = (0..n).collect();
    let mut left_indices = indices[..m].to_vec();
    let mut right_indices = indices[m..].to_vec();
    left_indices.shuffle(&mut rand::thread_rng());
    right_indices.shuffle(&mut rand::thread_rng());

    // Compute the target syndrome
    let target_syndrome = calculate_syndrome(received_vector, h);

    // Create hash maps to store syndrome-to-subset mappings
    let mut left_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
    let mut right_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();

    // Generate subsets for both halves and store their syndromes
    // Left half subsets
    let left_weight = weight / 2;
    for subset in generate_subsets(&left_indices, left_weight) {
        let mut candidate_error = vec![0; n];
        for &i in &subset {
            candidate_error[i] = 1;
        }
        let syndrome = calculate_syndrome(&candidate_error, h);
        left_map.insert(syndrome.clone(), subset);
    }

    // Right half subsets
    let right_weight = weight - left_weight;
    for subset in generate_subsets(&right_indices, right_weight) {
        let mut candidate_error = vec![0; n];
        for &i in &subset {
            candidate_error[i] = 1;
        }
        let syndrome = calculate_syndrome(&candidate_error, h);
        right_map.insert(syndrome.clone(), subset);
    }

    // Iterate through the left map to find complementary syndromes in the right map
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
            let duration = start.elapsed().as_nanos();
            println!("Time: {} ns", duration);
            return Some(error_vector);
        }
    }

    let duration = start.elapsed().as_nanos();
    println!("Time: {} ns", duration);

    None
}
