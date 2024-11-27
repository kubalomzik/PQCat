use std::collections::HashSet;

use itertools::Itertools;
use ndarray::{Array1, Array2};
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

// Generate a random linear code (G, H matrices)
pub fn generate_random_code(n: usize, k: usize) -> (Array2<u8>, Array2<u8>) {
    assert!(k < n, "k must be less than n");
    let mut rng = rand::thread_rng();

    // Generate a random (k x (n-k)) P matrix
    let p = Array2::from_shape_fn((k, n - k), |_| rng.gen_range(0..=1));

    // Construct G = [I_k | P]
    let mut g = Array2::<u8>::zeros((k, n));
    for i in 0..k {
        g[(i, i)] = 1; // Identity part
    }
    for i in 0..k {
        for j in 0..(n - k) {
            g[(i, k + j)] = p[(i, j)];
        }
    }

    // Construct H = [-P^T | I_{n-k}]
    let mut h = Array2::<u8>::zeros((n - k, n));
    for i in 0..(n - k) {
        h[(i, k + i)] = 1; // Identity part
    }
    for i in 0..k {
        for j in 0..(n - k) {
            h[(j, i)] = p[(i, j)]; // -P^T part (no need for negative in GF(2))
        }
    }

    (g, h)
}

pub fn generate_random_error_vector(n: usize, weight: usize) -> Vec<u8> {
    assert!(
        weight <= n,
        "Weight must be less than or equal to the length of the vector"
    );

    let mut indices: Vec<usize> = (0..n).collect();
    indices.shuffle(&mut rand::thread_rng());
    let mut error_vector = vec![0; n];
    for &i in indices.iter().take(weight) {
        error_vector[i] = 1;
    }
    error_vector
}

/// Generate a random syndrome by multiplying a random error vector with the parity-check matrix
pub fn generate_random_syndrome(h: &Array2<u8>, weight: usize) -> Array1<u8> {
    let n = h.shape()[1]; // Number of columns in H (length of error vector)

    // Generate a random error vector of length n
    let error_vector = generate_random_error_vector(n, weight);

    // Calculate the syndrome: s = H * e^T
    let mut syndrome = h.dot(&Array1::from(error_vector));

    // Since we work over GF(2), take values mod 2
    syndrome.mapv_inplace(|x| x % 2);

    syndrome
}

// Apply errors to a codeword
pub fn apply_errors(codeword: &[u8], error_vector: &[u8]) -> Vec<u8> {
    codeword
        .iter()
        .zip(error_vector.iter())
        .map(|(c, e)| c ^ e)
        .collect()
}

// Calculate the syndrome of a received vector
pub fn calculate_syndrome(vector: &[u8], h: &Array2<u8>) -> Vec<u8> {
    assert_eq!(
        h.shape()[1],
        vector.len(),
        "Parity-check matrix H and vector dimensions must align"
    );
    let vec = Array1::from(vector.to_vec());
    let result = h.dot(&vec).mapv(|x| x % 2); // Element-wise modulo operation
    result.to_vec()
}

pub fn generate_subsets(indices: &[usize], subset_size: usize) -> Vec<Vec<usize>> {
    let mut results = vec![];
    let mut stack = vec![(0, vec![])];
    while let Some((start, mut current_subset)) = stack.pop() {
        if current_subset.len() == subset_size {
            results.push(current_subset);
        } else {
            for i in start..indices.len() {
                current_subset.push(indices[i]);
                stack.push((i + 1, current_subset.clone())); // clone the modified subset
                current_subset.pop(); // revert to previous state
            }
        }
    }
    results
}

pub fn compute_h_tilde(h: &Array2<u8>, p: usize) -> Array2<u8> {
    let (rows, cols) = h.dim();
    let mut rng = thread_rng();

    // Randomly select `p` rows from the matrix
    let selected_rows: Vec<usize> = (0..rows).choose_multiple(&mut rng, p);

    // Randomly permute columns
    let mut permuted_columns: Vec<usize> = (0..cols).collect();
    permuted_columns.shuffle(&mut rng);

    // Construct H-tilde
    let mut h_tilde_data = Vec::new();
    for &row in &selected_rows {
        let permuted_row: Vec<u8> = permuted_columns.iter().map(|&col| h[[row, col]]).collect();
        h_tilde_data.extend(permuted_row);
    }

    Array2::from_shape_vec((p, cols), h_tilde_data).unwrap()
}

pub fn column_match(
    h_tilde: &Array2<u8>,
    p: usize,
    l: usize,
    l1: usize,
    l2: usize,
) -> Vec<HashSet<usize>> {
    let cols = h_tilde.shape()[1];
    let mut solutions = Vec::new();

    for subset in (0..cols).combinations(l1 + l2) {
        let mut column_sum = Array1::zeros(p);

        for &col in &subset {
            column_sum = (&column_sum + &h_tilde.column(col).to_owned()) % 2;
        }

        if column_sum.iter().filter(|&&x: &&u8| x == 1).count() == l {
            solutions.push(subset.into_iter().collect::<HashSet<usize>>());
        }
    }

    solutions
}

pub fn extract_q_i(h_tilde: &Array2<u8>, indices: &HashSet<usize>) -> Array2<u8> {
    let rows = h_tilde.shape()[0];
    let mut submatrix_data = Vec::new();

    for &index in indices {
        submatrix_data.extend(h_tilde.column(index).iter());
    }

    Array2::from_shape_vec((rows, indices.len()), submatrix_data).unwrap()
}

pub fn calculate_weight(vec: &[u8]) -> usize {
    vec.iter().filter(|&&bit| bit == 1).count()
}

pub fn support_subset(q_i: &Array2<u8>) -> Vec<usize> {
    let mut support = Vec::new();
    let cols = q_i.shape()[1];

    for col in 0..cols {
        if q_i.column(col).iter().any(|&x| x == 1) {
            support.push(col);
        }
    }

    support
}

pub fn validate_syndrome(
    error_vector: &Vec<u8>,           // Error vector, length n
    syndrome: &Array1<u8>,            // Syndrome vector, length m
    parity_check_matrix: &Array2<u8>, // Parity-check matrix H, size m x n
) -> bool {
    // Convert error vector to an ndarray array
    let error_array = Array1::from(error_vector.to_owned());

    // Compute the syndrome: H * e^T (mod 2)
    let computed_syndrome = parity_check_matrix.dot(&error_array) % 2;

    // Compare with the target syndrome
    computed_syndrome == syndrome
}
