use crate::attacks::attack_utils::{calculate_partial_syndrome, calculate_syndrome};
use ndarray::Array2;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::time::Instant;

pub fn run_bjmm_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    n: usize,
    weight: usize,
) -> Option<Vec<u8>> {
    let start = Instant::now();

    let max_iterations = 100;
    let list_size = 256;
    let r = h.shape()[0];

    let target_syndrome = calculate_syndrome(received_vector, h);

    let mut rng = thread_rng();

    for _iteration in 0..max_iterations {
        // Bring parity check matrix to systematic form (permute columns)
        let mut indices: Vec<usize> = (0..n).collect();
        indices.shuffle(&mut rng);

        // Partition the indices for a 4-way split as per BJMM
        let quarter = n / 4;
        let part1: Vec<usize> = indices[0..quarter].to_vec();
        let part2: Vec<usize> = indices[quarter..2 * quarter].to_vec();
        let part3: Vec<usize> = indices[2 * quarter..3 * quarter].to_vec();
        let part4: Vec<usize> = indices[3 * quarter..n].to_vec();

        // Split the weight roughly into 4 parts
        let w1 = weight / 4;
        let w2 = weight / 4;
        let w3 = weight / 4;
        let w4 = weight - w1 - w2 - w3;

        // Build intermediate representation lists (first level)

        let mut list_a: HashMap<Vec<u8>, Vec<Vec<usize>>> = HashMap::new();
        for _ in 0..list_size {
            let selected_indices = part1
                .choose_multiple(&mut rng, w1.min(part1.len()))
                .cloned()
                .collect::<Vec<usize>>();

            let representation = calculate_partial_syndrome(h, &selected_indices, r);

            list_a
                .entry(representation)
                .or_default()
                .push(selected_indices);
        }

        let mut list_b: HashMap<Vec<u8>, Vec<Vec<usize>>> = HashMap::new();
        for _ in 0..list_size {
            let selected_indices = part2
                .choose_multiple(&mut rng, w2.min(part2.len()))
                .cloned()
                .collect::<Vec<usize>>();

            let representation = calculate_partial_syndrome(h, &selected_indices, r);

            list_b
                .entry(representation)
                .or_default()
                .push(selected_indices);
        }

        // Build second-level representation lists by merging

        let mut list_c: HashMap<Vec<u8>, Vec<Vec<usize>>> = HashMap::new();
        for _ in 0..list_size {
            let selected_indices = part3
                .choose_multiple(&mut rng, w3.min(part3.len()))
                .cloned()
                .collect::<Vec<usize>>();

            let representation = calculate_partial_syndrome(h, &selected_indices, r);

            list_c
                .entry(representation)
                .or_default()
                .push(selected_indices);
        }

        let mut list_d: HashMap<Vec<u8>, Vec<Vec<usize>>> = HashMap::new();
        for _ in 0..list_size {
            let selected_indices = part4
                .choose_multiple(&mut rng, w4.min(part4.len()))
                .cloned()
                .collect::<Vec<usize>>();

            let representation = calculate_partial_syndrome(h, &selected_indices, r);

            list_d
                .entry(representation)
                .or_default()
                .push(selected_indices);
        }

        // Look for matches between combined representations

        for (rep_a, subsets_a) in &list_a {
            for (rep_b, subsets_b) in &list_b {
                // Calculate combined representation for AB
                let mut rep_ab = rep_a.clone();
                for i in 0..r {
                    rep_ab[i] ^= rep_b[i];
                }

                // Calculate what we need from C and D to match target
                let mut needed_rep_cd = target_syndrome.clone();
                for i in 0..r {
                    needed_rep_cd[i] ^= rep_ab[i];
                }

                for (rep_c, subsets_c) in &list_c {
                    // Calculate what we need from list_d
                    let mut needed_rep_d = needed_rep_cd.clone();
                    for i in 0..r {
                        needed_rep_d[i] ^= rep_c[i];
                    }

                    // Look for this representation in list_d
                    if let Some(subsets_d) = list_d.get(&needed_rep_d) {
                        // We found a potential match, try combining representations to form a complete error vector
                        for subset_a in subsets_a {
                            for subset_b in subsets_b {
                                for subset_c in subsets_c {
                                    for subset_d in subsets_d {
                                        // Create the combined error vector
                                        let mut error_vector = vec![0; n];
                                        for &idx in subset_a
                                            .iter()
                                            .chain(subset_b.iter())
                                            .chain(subset_c.iter())
                                            .chain(subset_d.iter())
                                        {
                                            error_vector[idx] = 1;
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
                        }
                    }
                }
            }
        }
    }

    let duration = start.elapsed().as_micros();
    println!("Time: {} μs", duration);
    None
}
