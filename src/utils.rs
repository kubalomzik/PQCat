use std::collections::HashSet;

use itertools::Itertools;
use ndarray::{Array1, Array2};
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;

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

pub fn apply_errors(codeword: &[u8], error_vector: &[u8]) -> Vec<u8> {
    codeword
        .iter()
        .zip(error_vector.iter())
        .map(|(c, e)| c ^ e)
        .collect()
}

pub fn calculate_syndrome(error_vector: &[u8], h: &Array2<u8>) -> Vec<u8> {
    let syndrome: Vec<u8> = h
        .outer_iter()
        .map(|row| {
            let row_vec: Vec<u8> = row.to_owned().to_vec();  // Convert row to Vec<u8>
            row_vec.iter().zip(error_vector.iter())
                .map(|(r, e)| r & e) // Perform element-wise dot product (mod 2)
                .fold(0, |acc, x| acc ^ x)  // XOR all elements to get the syndrome
        })
        .collect();
    syndrome
}

pub fn generate_subsets<'a>(indices: &'a [usize], size: usize) -> impl Iterator<Item = Vec<usize>> + 'a {
    indices.iter().cloned().combinations(size)
}

pub fn compute_h_tilde(h: &Array2<u8>, p: usize) -> Array2<u8> {
    let rows = h.nrows();
    let cols = h.ncols();
    let mut h_tilde = Array2::<u8>::zeros((rows, cols / p));
    for i in 0..rows {
        for j in 0..(cols / p) {
            h_tilde[[i, j]] = h[[i, j * p]];
        }
    }
    h_tilde
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

        // Check if the weight of the sum matches `l`
        if column_sum.iter().map(|&x: &u8| x as usize).filter(|&x| x == 1).count() == l {
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

pub fn validate_syndrome(e: &[u8], s: &Array1<u8>, h: &Array2<u8>) -> bool {
    let syndrome = h.dot(&Array1::from(e.to_vec()));
    syndrome.iter().zip(s.iter()).all(|(a, b)| a == b)
}
