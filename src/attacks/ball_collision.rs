use crate::attacks::attack_utils::calculate_syndrome;
use ndarray::Array2;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::time::Instant;

pub fn run_ball_collision_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    n: usize,
    weight: usize,
) -> Option<Vec<u8>> {
    let start = Instant::now();

    let max_iterations = 100;
    let list_size = 1000;

    let target_syndrome = calculate_syndrome(received_vector, h);
    let r = h.shape()[0];

    for iteration in 0..max_iterations {
        println!("Iteration {}/{}", iteration + 1, max_iterations);

        // Split indices into two parts
        let mut indices: Vec<usize> = (0..n).collect();
        indices.shuffle(&mut thread_rng());

        let half = n / 2;
        let part1: Vec<usize> = indices[0..half].to_vec();
        let part2: Vec<usize> = indices[half..n].to_vec();

        // Split weight between parts
        let p1 = weight / 2; // First half weight
        let p2 = weight - p1; // Second half weight

        // Generate first list
        let mut list1: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        for _ in 0..list_size {
            // Select random positions from part1
            let mut rng = thread_rng();
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
        for _ in 0..list_size {
            // Select random positions from part2
            let mut rng = thread_rng();
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
                let mut error_vector = vec![0; n];

                // Set bits from both lists
                for &i in indices1 {
                    error_vector[i] = 1;
                }

                for &i in &selected_indices {
                    error_vector[i] = 1;
                }

                let check_syndrome = calculate_syndrome(&error_vector, h);
                if check_syndrome == target_syndrome {
                    let duration = start.elapsed().as_micros();
                    println!("Time: {} μs", duration);
                    return Some(error_vector);
                }
            }
        }
    }

    let duration = start.elapsed().as_micros();
    println!("Time: {} μs", duration);
    None
}
