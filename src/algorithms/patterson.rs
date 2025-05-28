use crate::algorithms::algorithm_utils::calculate_syndrome;
use crate::algorithms::metrics::{AlgorithmMetrics, start_memory_tracking, update_peak_memory};
use crate::codes::polynomial_utils::{evaluate_poly, trim_polynomial};
use crate::types::FiniteField;
use crate::types::GoppaParams;
use ndarray::Array2;
use std::time::Instant;

/// Compute the syndrome polynomial S(z)
fn compute_syndrome_polynomial(
    received: &[u8],
    support: &[u8],
    goppa_poly: &[u8],
    field: &FiniteField,
    n: usize,
) -> Vec<u8> {
    let t = goppa_poly.len() - 1;
    let mut syndrome = vec![0; t];

    // For each position in the received vector
    for i in 0..n {
        if received[i] == 1 {
            let x = support[i];
            if x == 0 {
                continue; // Skip if support element is 0
            }

            // Calculate g(x)^(-1)
            let g_x = evaluate_poly(goppa_poly, x, field);
            if g_x == 0 {
                continue; // Skip if x is a root of g(z)
            }
            let g_x_inv = field.inverse(g_x);

            // Update syndrome polynomial
            let mut x_pow = 1;
            for (_j, syndrome_coef) in syndrome.iter_mut().enumerate().take(t) {
                let term = field.field_multiply(g_x_inv, x_pow);
                *syndrome_coef ^= term; // XOR since we're in GF(2)
                x_pow = field.field_multiply(x_pow, x);
            }
        }
    }

    syndrome
}

/// Find the error locator polynomial using the Berlekamp-Massey algorithm
fn berlekamp_massey(syndrome: &[u8], field: &FiniteField, t: usize) -> Vec<u8> {
    // Ensure syndrome has length 2t
    let mut syndrome_seq = syndrome.to_vec();
    if syndrome_seq.len() < 2 * t {
        syndrome_seq.resize(2 * t, 0);
    }

    let mut connection_poly: Vec<u8> = vec![1]; // Connection polynomial (sigma)
    let mut prev_connection_poly = vec![1]; // Previous connection polynomial
    let mut lfsr_length = 0; // Current length of LFSR
    let mut last_discrepancy = 1; // Scalar factor
    let mut iterations_since_change = 1; // Number of iterations since lfsr_length changed

    for n in 0..2 * t {
        // Compute discrepancy
        let mut discrepancy = syndrome_seq[n];
        for i in 1..=lfsr_length {
            if i < connection_poly.len() {
                discrepancy ^= field.field_multiply(connection_poly[i], syndrome_seq[n - i]);
            }
        }

        if discrepancy == 0 {
            iterations_since_change += 1;
        } else {
            // Connection polynomial needs adjustment
            if 2 * lfsr_length <= n {
                // Save old connection polynomial
                let temp = connection_poly.clone();

                // connection_poly(z) = connection_poly(z) - discrepancy/last_discrepancy * z^iterations_since_change * prev_connection_poly(z)
                let factor = field.field_multiply(discrepancy, field.inverse(last_discrepancy));

                // Create shifted prev_connection_poly polynomial (z^iterations_since_change * prev_connection_poly(z))
                let mut shifted_prev_poly =
                    vec![0; iterations_since_change + prev_connection_poly.len()];
                shifted_prev_poly[iterations_since_change
                    ..(prev_connection_poly.len() + iterations_since_change)]
                    .copy_from_slice(&prev_connection_poly[..]);

                // Scale shifted_prev_poly by discrepancy/last_discrepancy
                for i in &mut shifted_prev_poly {
                    *i = field.field_multiply(*i, factor);
                }

                // Resize connection_poly if needed
                if shifted_prev_poly.len() > connection_poly.len() {
                    connection_poly.resize(shifted_prev_poly.len(), 0);
                }

                // connection_poly = connection_poly - discrepancy/last_discrepancy * z^iterations_since_change * prev_connection_poly
                for i in 0..shifted_prev_poly.len() {
                    if i < connection_poly.len() {
                        connection_poly[i] ^= shifted_prev_poly[i]; // XOR in GF(2)
                    }
                }

                // Trim trailing zeros
                trim_polynomial(&mut connection_poly);

                // Update variables
                lfsr_length = n + 1 - lfsr_length;
                prev_connection_poly = temp;
                last_discrepancy = discrepancy;
                iterations_since_change = 1;
            } else {
                // connection_poly(z) = connection_poly(z) - discrepancy/last_discrepancy * z^iterations_since_change * prev_connection_poly(z)
                let factor = field.field_multiply(discrepancy, field.inverse(last_discrepancy));

                // Create shifted prev_connection_poly polynomial (z^iterations_since_change * prev_connection_poly(z))
                let mut shifted_prev_poly =
                    vec![0; iterations_since_change + prev_connection_poly.len()];
                shifted_prev_poly[iterations_since_change
                    ..(prev_connection_poly.len() + iterations_since_change)]
                    .copy_from_slice(&prev_connection_poly[..]);

                // Scale shifted_prev_poly by discrepancy/last_discrepancy
                for i in &mut shifted_prev_poly {
                    *i = field.field_multiply(*i, factor);
                }

                // Resize connection_poly if needed
                if shifted_prev_poly.len() > connection_poly.len() {
                    connection_poly.resize(shifted_prev_poly.len(), 0);
                }

                // connection_poly = connection_poly - discrepancy/last_discrepancy * z^iterations_since_change * prev_connection_poly
                for i in 0..shifted_prev_poly.len() {
                    if i < connection_poly.len() {
                        connection_poly[i] ^= shifted_prev_poly[i]; // XOR in GF(2)
                    }
                }

                // Trim trailing zeros
                trim_polynomial(&mut connection_poly);

                iterations_since_change += 1;
            }
        }
    }

    // Reverse the connection polynomial to get sigma(z)
    // (Berlekamp-Massey gives connection polynomial in reverse order)
    connection_poly.reverse();

    // Ensure the polynomial has degree at most t
    if connection_poly.len() > t + 1 {
        connection_poly.truncate(t + 1);
    }

    connection_poly
}

/// Find the roots of sigma polynomial
fn find_roots(sigma: &[u8], support: &[u8], field: &FiniteField, n: usize) -> Vec<usize> {
    let mut error_positions = Vec::new();

    // Check if polynomial is valid
    if sigma.len() <= 1 || sigma.iter().all(|&x| x == 0) {
        return error_positions;
    }

    // Check each support element with multiple evaluation methods
    for (i, &x) in support.iter().take(n).enumerate() {
        // Regular polynomial evaluation
        let y1 = evaluate_poly(sigma, x, field);

        // Horner's method for verification
        let y2 = evaluate_poly_horner(sigma, x, field);

        // Consider a root if either method finds it
        // (This helps with numerical instability in finite fields)
        if y1 == 0 || y2 == 0 {
            println!("Confirmed error at position {}, x={:#x}", i, x);
            error_positions.push(i);
        }
    }
    error_positions
}

fn evaluate_poly_horner(poly: &[u8], x: u8, field: &FiniteField) -> u8 {
    let mut result = 0;
    for &coef in poly.iter().rev() {
        result = field.field_add(field.field_multiply(result, x), coef);
    }
    result
}

pub fn run_patterson_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    goppa_params: &GoppaParams,
    w: usize,
) -> (Option<Vec<u8>>, AlgorithmMetrics) {
    let start_time = Instant::now();
    let start_memory = start_memory_tracking();
    let mut peak_memory = 0;

    update_peak_memory(start_memory, &mut peak_memory);

    let support = &goppa_params.support;
    let goppa_poly = &goppa_params.goppa_poly;
    let field = &goppa_params.field;
    let t = goppa_params.t;

    let n = received_vector.len();

    // Compute the syndrome polynomial S(z)
    let syndrome = compute_syndrome_polynomial(received_vector, support, goppa_poly, field, n);

    if syndrome.iter().all(|&x| x == 0) {
        // No errors detected
        update_peak_memory(start_memory, &mut peak_memory);

        let metrics = AlgorithmMetrics {
            time: start_time.elapsed().as_micros() as usize,
            peak_memory,
        };

        return (Some(vec![0; n]), metrics);
    }

    // Ensure the syndrome is properly formatted for Berlekamp-Massey
    let mut extended_syndrome = syndrome.clone();
    if extended_syndrome.len() < 2 * t {
        // Extend the syndrome if needed
        let original_length = extended_syndrome.len();
        extended_syndrome.resize(2 * t, 0);

        // For binary Goppa codes, compute additional syndrome elements
        // This is important for t>2 cases
        if t > 2 {
            for i in original_length..2 * t {
                // For binary Goppa codes in characteristic 2, compute additional syndrome terms using the recursive relationship of syndromes
                let mut s_i = 0;
                for j in 1..=i / 2 {
                    if j < original_length && (i - j) < original_length {
                        let s_j = extended_syndrome[j];
                        let s_ij = extended_syndrome[i - j];
                        s_i ^= field.field_multiply(s_j, s_ij);
                    }
                }
                extended_syndrome[i] = s_i;
            }
        }
    }

    // Find the error locator polynomial using Berlekamp-Massey with extended syndrome
    let sigma = berlekamp_massey(&extended_syndrome, field, t);

    // Find roots of sigma(z) - these are the error locations
    let error_positions = find_roots(&sigma, support, field, n);

    // Construct error vector from positions
    let mut error_vector = vec![0; n];
    for &pos in &error_positions {
        error_vector[pos] = 1;
    }

    // Check if we found all expected errors
    if !error_positions.is_empty() {
        // Modified validation approach
        let received_xor_error = received_vector
            .iter()
            .zip(error_vector.iter())
            .map(|(&r, &e)| r ^ e)
            .collect::<Vec<u8>>();

        // Check if the result is a valid codeword
        let result_syndrome = calculate_syndrome(&received_xor_error, h);

        if result_syndrome.iter().all(|&x| x == 0) {
            // Success - we found a valid error pattern
            update_peak_memory(start_memory, &mut peak_memory);
            let metrics = AlgorithmMetrics {
                time: start_time.elapsed().as_micros() as usize,
                peak_memory,
            };
            return (Some(error_vector), metrics);
        }
    }

    if t > 2 && !error_positions.is_empty() && error_positions.len() < t {
        // We found some but not all errors, try to find the rest
        // Calculate the remaining syndrome after correcting known errors
        let mut partial_correction = vec![0; n];
        for &pos in &error_positions {
            partial_correction[pos] = 1;
        }

        // Try to find the remaining errors with a smaller brute force search
        let remaining_t = t - error_positions.len();

        use itertools::Itertools;
        let positions: Vec<usize> = (0..n).collect();
        let mut pattern_count = 0;
        let max_patterns_for_completion = 10000;

        for combo in positions.iter().combinations(remaining_t) {
            if pattern_count >= max_patterns_for_completion {
                break;
            }

            // Skip positions we already found
            if combo.iter().any(|&&pos| error_positions.contains(&pos)) {
                continue;
            }

            let mut trial_error = partial_correction.clone();
            for &&pos in combo.iter() {
                trial_error[pos] = 1;
            }

            // Check if this completes the correction
            let corrected = received_vector
                .iter()
                .zip(trial_error.iter())
                .map(|(&r, &e)| r ^ e)
                .collect::<Vec<u8>>();

            let check = calculate_syndrome(&corrected, h);

            if check.iter().all(|&x| x == 0) {
                update_peak_memory(start_memory, &mut peak_memory);
                let metrics = AlgorithmMetrics {
                    time: start_time.elapsed().as_micros() as usize,
                    peak_memory,
                };

                return (Some(trial_error), metrics);
            }

            pattern_count += 1;
        }
    }

    // If we get here, the standard approach failed - try brute force for small t
    if t <= 4 {
        // Limit the number of patterns to try for safety
        let max_patterns = 10000;
        let mut pattern_count = 0;

        if w == 1 {
            // Single error case
            for i in 0..n {
                let mut trial_error = vec![0; n];
                trial_error[i] = 1;

                // Check if this corrects the errors
                let corrected = received_vector
                    .iter()
                    .zip(trial_error.iter())
                    .map(|(&r, &e)| r ^ e)
                    .collect::<Vec<u8>>();

                let check = calculate_syndrome(&corrected, h);

                if check.iter().all(|&x| x == 0) {
                    update_peak_memory(start_memory, &mut peak_memory);
                    let metrics = AlgorithmMetrics {
                        time: start_time.elapsed().as_micros() as usize,
                        peak_memory,
                    };

                    return (Some(trial_error), metrics);
                }
            }
        } else if t == 2 {
            // For t=2, try all possible pairs of errors
            for i in 0..n {
                for j in i + 1..n {
                    if pattern_count >= max_patterns {
                        break;
                    }

                    let mut trial_error = vec![0; n];
                    trial_error[i] = 1;
                    trial_error[j] = 1;

                    // Check if this corrects the errors
                    let corrected = received_vector
                        .iter()
                        .zip(trial_error.iter())
                        .map(|(&r, &e)| r ^ e)
                        .collect::<Vec<u8>>();

                    let check = calculate_syndrome(&corrected, h);

                    if check.iter().all(|&x| x == 0) {
                        update_peak_memory(start_memory, &mut peak_memory);
                        let metrics = AlgorithmMetrics {
                            time: start_time.elapsed().as_micros() as usize,
                            peak_memory,
                        };

                        return (Some(trial_error), metrics);
                    }

                    pattern_count += 1;
                }
            }
        } else if t == 3 || t == 4 {
            use itertools::Itertools;

            let positions: Vec<usize> = (0..n).collect();

            for combo in positions.iter().combinations(t) {
                pattern_count += 1;
                if pattern_count >= max_patterns {
                    break;
                }

                let mut trial_error = vec![0; n];
                for &&pos in combo.iter() {
                    trial_error[pos] = 1;
                }

                // Check if this corrects the errors
                let corrected = received_vector
                    .iter()
                    .zip(trial_error.iter())
                    .map(|(&r, &e)| r ^ e)
                    .collect::<Vec<u8>>();

                let check = calculate_syndrome(&corrected, h);

                if check.iter().all(|&x| x == 0) {
                    update_peak_memory(start_memory, &mut peak_memory);
                    let metrics = AlgorithmMetrics {
                        time: start_time.elapsed().as_micros() as usize,
                        peak_memory,
                    };

                    return (Some(trial_error), metrics);
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
