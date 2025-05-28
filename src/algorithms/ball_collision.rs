use crate::algorithms::algorithm_utils::calculate_syndrome;
use crate::algorithms::config::{LIST_SIZE, MAX_ITERATIONS};
use crate::algorithms::metrics::{AlgorithmMetrics, start_memory_tracking, update_peak_memory};
use ndarray::Array2;
use rand::prelude::IndexedRandom;
use rand::{rng, seq::SliceRandom};
use std::collections::HashMap;
use std::time::Instant;

pub fn run_ball_collision_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    n: usize,
    weight: usize,
) -> (Option<Vec<u8>>, AlgorithmMetrics) {
    let start_time = Instant::now();
    let start_memory = start_memory_tracking();
    let mut peak_memory = 0;

    let target_syndrome = calculate_syndrome(received_vector, h);
    update_peak_memory(start_memory, &mut peak_memory);
    let r = h.shape()[0];

    for _iteration in 0..MAX_ITERATIONS {
        // Split indices into two parts
        let mut indices: Vec<usize> = (0..n).collect();
        indices.shuffle(&mut rng());

        let half = n / 2;
        let part1: Vec<usize> = indices[0..half].to_vec();
        let part2: Vec<usize> = indices[half..n].to_vec();

        // Split weight between parts
        let p1 = weight / 2; // First half weight
        let p2 = weight - p1; // Second half weight

        // Generate first list
        let mut list1: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        for _ in 0..LIST_SIZE {
            // Select random positions from part1
            let mut rng = rng();
            let selected_indices = part1
                .choose_multiple(&mut rng, p1.min(part1.len()))
                .cloned()
                .collect::<Vec<usize>>();

            if selected_indices.is_empty() {
                continue;
            }

            // Calculate partial syndrome
            let mut partial_syndrome = vec![0; r];
            for &idx in &selected_indices {
                for j in 0..r {
                    partial_syndrome[j] ^= h[[j, idx]];
                }
            }

            // Store indices for this syndrome
            list1.insert(partial_syndrome, selected_indices);
        }

        // Generate second list and check for collisions
        for _ in 0..LIST_SIZE {
            // Select random positions from part2
            let mut rng = rng();
            let selected_indices = part2
                .choose_multiple(&mut rng, p2.min(part2.len()))
                .cloned()
                .collect::<Vec<usize>>();

            if selected_indices.is_empty() {
                continue;
            }

            // Calculate partial syndrome
            let mut partial_syndrome = vec![0; r];
            for &idx in &selected_indices {
                for j in 0..r {
                    partial_syndrome[j] ^= h[[j, idx]];
                }
            }

            // Calculate what we need from list1 to match target
            let mut needed_syndrome = vec![0; r];
            for i in 0..r {
                needed_syndrome[i] = target_syndrome[i] ^ partial_syndrome[i];
            }

            // Look for matching syndrome in list1
            if let Some(indices1) = list1.get(&needed_syndrome) {
                // Found a potential match, create error vector
                let mut candidate_error = vec![0; n];

                // Set bits from both lists
                for &i in indices1 {
                    candidate_error[i] = 1;
                }

                for &i in &selected_indices {
                    candidate_error[i] = 1;
                }

                let check_syndrome = calculate_syndrome(&candidate_error, h);
                if check_syndrome == target_syndrome {
                    update_peak_memory(start_memory, &mut peak_memory);

                    let metrics = AlgorithmMetrics {
                        time: start_time.elapsed().as_micros() as usize,
                        peak_memory,
                    };

                    return (Some(candidate_error), metrics);
                }
            }
        }
    }

    update_peak_memory(start_memory, &mut peak_memory);

    let metrics = AlgorithmMetrics {
        time: start_time.elapsed().as_micros() as usize,
        peak_memory,
    };

    (None, metrics)
}
