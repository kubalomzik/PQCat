use crate::algorithms::algorithm_utils::calculate_syndrome;
use crate::algorithms::config::MAX_ITERATIONS;
use crate::algorithms::metrics::{start_memory_tracking, update_peak_memory, AlgorithmMetrics};
use ndarray::Array2;
use rand::seq::SliceRandom;
use std::time::Instant;

pub fn run_prange_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    weight: usize,
) -> (Option<Vec<u8>>, AlgorithmMetrics) {
    let start_time = Instant::now();
    let start_memory = start_memory_tracking();
    let mut peak_memory = 0;

    let target_syndrome = calculate_syndrome(received_vector, h);
    update_peak_memory(start_memory, &mut peak_memory);
    let n = h.shape()[1]; // Length of the error vector
    let mut indices: Vec<usize> = (0..n).collect(); // All possible indices

    let mut loop_count = 0;

    while loop_count < MAX_ITERATIONS {
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
        if candidate_syndrome == target_syndrome {
            update_peak_memory(start_memory, &mut peak_memory);

            let metrics = AlgorithmMetrics {
                time: start_time.elapsed().as_micros() as usize,
                peak_memory,
            };

            return (Some(candidate_error), metrics);
        }
        loop_count += 1;
    }

    update_peak_memory(start_memory, &mut peak_memory);

    let metrics = AlgorithmMetrics {
        time: start_time.elapsed().as_micros() as usize,
        peak_memory,
    };

    (None, metrics)
}
