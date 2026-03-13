use crate::algorithms::algorithm_utils::{
    apply_inverse_permutation, calculate_partial_syndrome, calculate_syndrome, generate_subsets,
    permute_columns, xor_assign,
};
use crate::algorithms::metrics::{AlgorithmMetrics, start_memory_tracking, update_peak_memory};
use ndarray::Array2;
use rand::rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::time::Instant;

const PARTITIONS: usize = 3;
const MAX_SYSTEMATIC_ATTEMPTS: usize = 64;

pub fn run_finiasz_sendrier_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    weight: usize,
) -> (Option<Vec<u8>>, AlgorithmMetrics) {
    let start_time = Instant::now();
    let start_memory = start_memory_tracking();
    let mut peak_memory = 0;

    let (rows, cols) = h.dim();

    if weight == 0 {
        let metrics = AlgorithmMetrics {
            time: start_time.elapsed().as_micros() as usize,
            peak_memory,
            best_syndrome_distance: 0,
        };
        return (Some(vec![0u8; cols]), metrics);
    }

    let mut attempt = 0usize;
    let mut rng_handle = rng();

    while attempt < MAX_SYSTEMATIC_ATTEMPTS {
        attempt += 1;

        let mut base_permutation: Vec<usize> = (0..cols).collect();
        base_permutation.shuffle(&mut rng_handle);

        let permuted_h = permute_columns(h, &base_permutation);
        let Some((systematic_h, column_order)) = parity_check_to_systematic(&permuted_h) else {
            continue;
        };

        let final_permutation: Vec<usize> = column_order
            .iter()
            .map(|&idx| base_permutation[idx])
            .collect();

        let mut permuted_received = vec![0u8; cols];
        for (new_idx, &orig_idx) in final_permutation.iter().enumerate() {
            permuted_received[new_idx] = received_vector[orig_idx];
        }

        let target_syndrome = calculate_syndrome(&permuted_received, &systematic_h);
        update_peak_memory(start_memory, &mut peak_memory);

        let free_cols = cols.saturating_sub(rows);
        if free_cols == 0 {
            continue;
        }

        let mut pivot_target = ((weight * rows + cols - 1) / cols).min(weight).min(rows);
        let mut free_weight_total = weight.saturating_sub(pivot_target);

        if free_weight_total > free_cols {
            free_weight_total = free_cols;
            pivot_target = weight.saturating_sub(free_weight_total).min(rows);
        }

        let pivot_rows = pivot_target.min(rows);
        let main_rows = rows.saturating_sub(pivot_rows);

        let mut free_indices: Vec<usize> = (0..free_cols).collect();
        free_indices.shuffle(&mut rng_handle);

        let mut part_lengths = vec![free_cols / PARTITIONS; PARTITIONS];
        for i in 0..(free_cols % PARTITIONS) {
            part_lengths[i] += 1;
        }

        let mut partitions: Vec<Vec<usize>> = Vec::with_capacity(PARTITIONS);
        let mut offset = 0;
        for len in part_lengths {
            let mut part = Vec::with_capacity(len);
            for idx in 0..len {
                if offset + idx < free_indices.len() {
                    part.push(free_indices[offset + idx]);
                }
            }
            offset += len;
            partitions.push(part);
        }

        let mut weights = vec![0usize; PARTITIONS];
        if free_weight_total > 0 {
            for w in &mut weights {
                *w = free_weight_total / PARTITIONS;
            }
            for i in 0..(free_weight_total % PARTITIONS) {
                weights[i] += 1;
            }
        }

        let mut feasible = true;
        for (part, &w_part) in partitions.iter().zip(weights.iter()) {
            if w_part > part.len() {
                feasible = false;
                break;
            }
        }
        if !feasible {
            continue;
        }

        let mut left_entries = Vec::new();
        for subset in generate_subsets(&partitions[0], weights[0]) {
            let syndrome = calculate_partial_syndrome(&systematic_h, &subset, rows);
            left_entries.push((syndrome, subset));
        }
        update_peak_memory(start_memory, &mut peak_memory);

        let mut middle_entries = Vec::new();
        for subset in generate_subsets(&partitions[1], weights[1]) {
            let syndrome = calculate_partial_syndrome(&systematic_h, &subset, rows);
            middle_entries.push((syndrome, subset));
        }
        update_peak_memory(start_memory, &mut peak_memory);

        let mut combined_map: HashMap<Vec<u8>, Vec<(Vec<u8>, Vec<usize>, Vec<usize>)>> =
            HashMap::new();
        for (syn_a, subset_a) in &left_entries {
            for (syn_b, subset_b) in &middle_entries {
                let combined_full = xor_vectors(syn_a, syn_b);
                let key = combined_full[..main_rows].to_vec();
                combined_map.entry(key).or_insert_with(Vec::new).push((
                    combined_full,
                    subset_a.clone(),
                    subset_b.clone(),
                ));
            }
        }
        update_peak_memory(start_memory, &mut peak_memory);

        let target_main = target_syndrome[..main_rows].to_vec();

        for subset_c in generate_subsets(&partitions[2], weights[2]) {
            let syn_c = calculate_partial_syndrome(&systematic_h, &subset_c, rows);
            let mut lookup_key = target_main.clone();
            xor_prefix(&mut lookup_key, &syn_c, main_rows);

            if let Some(candidates) = combined_map.get(&lookup_key) {
                for (combined_full, subset_a, subset_b) in candidates {
                    let mut free_syndrome = combined_full.clone();
                    xor_assign(&mut free_syndrome, &syn_c);

                    let mut pivot_vector = target_syndrome.clone();
                    xor_assign(&mut pivot_vector, &free_syndrome);

                    let pivot_weight = pivot_vector.iter().filter(|&&bit| bit == 1).count();
                    let free_weight = subset_a.len() + subset_b.len() + subset_c.len();
                    let total_weight = free_weight + pivot_weight;

                    if total_weight != weight || pivot_weight != pivot_target {
                        continue;
                    }

                    let mut candidate_permuted = vec![0u8; cols];
                    for &idx in subset_a {
                        candidate_permuted[idx] = 1;
                    }
                    for &idx in subset_b {
                        candidate_permuted[idx] = 1;
                    }
                    for &idx in &subset_c {
                        candidate_permuted[idx] = 1;
                    }
                    for (row_idx, &bit) in pivot_vector.iter().enumerate() {
                        if bit == 1 {
                            candidate_permuted[free_cols + row_idx] = 1;
                        }
                    }

                    let candidate_error =
                        apply_inverse_permutation(&candidate_permuted, &final_permutation);
                    update_peak_memory(start_memory, &mut peak_memory);

                    let metrics = AlgorithmMetrics {
                        time: start_time.elapsed().as_micros() as usize,
                        peak_memory,
                        best_syndrome_distance: 0,
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
        best_syndrome_distance: rows,
    };
    (None, metrics)
}

fn parity_check_to_systematic(h: &Array2<u8>) -> Option<(Array2<u8>, Vec<usize>)> {
    let (rows, cols) = h.dim();
    let mut working = h.clone();
    let mut pivot_columns = Vec::with_capacity(rows);
    let mut current_row = 0usize;

    for col in 0..cols {
        if current_row == rows {
            break;
        }

        if let Some(pivot_row) = (current_row..rows).find(|&r| working[(r, col)] == 1) {
            if pivot_row != current_row {
                for c in 0..cols {
                    let tmp = working[(current_row, c)];
                    working[(current_row, c)] = working[(pivot_row, c)];
                    working[(pivot_row, c)] = tmp;
                }
            }

            for r in 0..rows {
                if r != current_row && working[(r, col)] == 1 {
                    for c in 0..cols {
                        working[(r, c)] ^= working[(current_row, c)];
                    }
                }
            }

            pivot_columns.push(col);
            current_row += 1;
        }
    }

    if pivot_columns.len() != rows {
        return None;
    }

    let mut is_pivot = vec![false; cols];
    for &col in &pivot_columns {
        is_pivot[col] = true;
    }

    let mut column_order = Vec::with_capacity(cols);
    for col in 0..cols {
        if !is_pivot[col] {
            column_order.push(col);
        }
    }
    for &col in &pivot_columns {
        column_order.push(col);
    }

    let mut systematic = Array2::<u8>::zeros((rows, cols));
    for (new_idx, &old_idx) in column_order.iter().enumerate() {
        let source_col = working.column(old_idx);
        systematic.column_mut(new_idx).assign(&source_col);
    }

    Some((systematic, column_order))
}

fn xor_vectors(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    let mut out = Vec::with_capacity(a.len());
    for (&x, &y) in a.iter().zip(b.iter()) {
        out.push(x ^ y);
    }
    out
}

fn xor_prefix(target: &mut [u8], other: &[u8], len: usize) {
    for i in 0..len {
        target[i] ^= other[i];
    }
}
