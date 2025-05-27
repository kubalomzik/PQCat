use crate::algorithms::algorithm_utils::calculate_syndrome;
use crate::algorithms::metrics::{start_memory_tracking, update_peak_memory, AlgorithmMetrics};
use crate::codes::polynomial_utils::{
    evaluate_poly, polynomial_add, polynomial_divide, polynomial_mod, polynomial_multiply,
    trim_polynomial,
};
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

/// Compute T(z) = sqrt(S(z) + z) mod g(z)
fn compute_t_polynomial(syndrome: &[u8], goppa_poly: &[u8], field: &FiniteField) -> Vec<u8> {
    let t = goppa_poly.len() - 1;

    // Compute S(z) + z
    let mut s_plus_z = syndrome.to_vec();
    s_plus_z.push(0); // Add z term (coefficient of z^t is 1)
    s_plus_z[1] ^= 1; // XOR with z

    // Compute square root (in GF(2^m), sqrt(a) = a^(2^(m-1)))
    // In binary field, this is simpler - we take every second term
    let mut t_poly = vec![0; t];
    for i in 0..t {
        if 2 * i < s_plus_z.len() {
            t_poly[i] = s_plus_z[2 * i];
        }
    }

    // Reduce modulo g(z)
    polynomial_mod(&mut t_poly, goppa_poly, field);

    t_poly
}

/// Solve the key equation: sigma(z)·T(z) ≡ omega(z) mod g(z)
fn solve_key_equation(
    t_poly: &[u8],
    goppa_poly: &[u8],
    field: &FiniteField,
    t: usize,
) -> (Vec<u8>, Vec<u8>) {
    // Initialize the Extended Euclidean Algorithm
    let mut r0 = goppa_poly.to_vec();
    let mut r1 = t_poly.to_vec();

    // Make sure polynomials are properly trimmed
    trim_polynomial(&mut r0);
    trim_polynomial(&mut r1);

    let mut v0 = vec![0; t + 1];
    let mut v1 = vec![0; t + 1];
    v1[0] = 1; // v1(z) = 1

    // Run the Extended Euclidean Algorithm until deg(r1) < t/2
    let max_deg = t / 2;

    // Safety counter to prevent infinite loops
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100;

    while !r1.is_empty() && r1.len() > max_deg + 1 && iterations < MAX_ITERATIONS {
        // Check if r1 has become zero or nearly zero
        if r1.iter().all(|&x| x == 0) {
            break;
        }

        // Ensure r1's leading coefficient is non-zero
        trim_polynomial(&mut r1);

        let (q, r) = polynomial_divide(&r0, &r1, field);

        // Break if remainder becomes zero
        if r.is_empty() || r.iter().all(|&x| x == 0) {
            break;
        }

        // Update remainders
        r0 = r1;
        r1 = r;

        // Update auxiliary polynomials
        let mut new_v = polynomial_multiply(&q, &v1, field);
        polynomial_add(&mut new_v, &v0);
        v0 = v1;
        v1 = new_v;

        iterations += 1;
    }

    // Ensure v1 is properly formed
    trim_polynomial(&mut v1);

    // If v1 is empty, use a default polynomial
    if v1.is_empty() {
        v1 = vec![1]; // Default to 1
    }

    (v1, r1)
}

/// Find the roots of sigma polynomial
fn find_roots(sigma: &[u8], support: &[u8], field: &FiniteField, n: usize) -> Vec<usize> {
    let mut error_positions = Vec::new();

    // Check each support element using iterator with enumeration
    for (i, &x) in support.iter().take(n).enumerate() {
        let y = evaluate_poly(sigma, x, field);

        // If sigma(x) = 0, then x is a root
        if y == 0 {
            error_positions.push(i);
        }
    }

    error_positions
}

pub fn run_patterson_algorithm(
    received_vector: &[u8],
    h: &Array2<u8>,
    goppa_params: &GoppaParams,
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

        println!("No errors detected");
        return (Some(vec![0; n]), metrics);
    }

    // Compute T(z) = sqrt(S(z) + z) mod g(z)
    let t_poly = compute_t_polynomial(&syndrome, goppa_poly, field);

    // Solve the key equation to find sigma(z) and omega(z)
    // Using Extended Euclidean Algorithm to solve: sigma(z)·T(z) = omega(z) mod g(z)
    let (sigma, _omega) = solve_key_equation(&t_poly, goppa_poly, field, t);

    // Find roots of sigma(z) - these are the error locations
    let error_positions = find_roots(&sigma, support, field, n);

    // Construct error vector from positions
    let mut error_vector = vec![0; n];
    for &pos in &error_positions {
        error_vector[pos] = 1;
    }

    // Verify correctness by recalculating syndrome
    let check_syndrome = calculate_syndrome(&error_vector, h);
    let original_syndrome = calculate_syndrome(received_vector, h);

    if check_syndrome == original_syndrome {
        update_peak_memory(start_memory, &mut peak_memory);
        let metrics = AlgorithmMetrics {
            time: start_time.elapsed().as_micros() as usize,
            peak_memory,
        };
        return (Some(error_vector), metrics);
    }

    update_peak_memory(start_memory, &mut peak_memory);

    let metrics = AlgorithmMetrics {
        time: start_time.elapsed().as_micros() as usize,
        peak_memory,
    };

    (None, metrics)
}
