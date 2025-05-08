use crate::attacks::attack_utils::calculate_syndrome;
use ndarray::Array2;
use rand::seq::SliceRandom;
use std::time::Instant;

pub fn run_prange_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    weight: usize,
) -> Option<Vec<u8>> {
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
            let duration = start.elapsed().as_micros();
            println!("Time: {} μs", duration);
            return Some(candidate_error);
        }
        loop_count += 1;
    }

    let duration = start.elapsed().as_micros();
    println!("Time: {} μs", duration);

    None
}
