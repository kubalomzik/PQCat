use itertools::Itertools;
use ndarray::{Array1, Array2};
use rand::seq::SliceRandom;

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
            let row_vec: Vec<u8> = row.to_owned().to_vec();
            row_vec
                .iter()
                .zip(error_vector.iter())
                .map(|(r, e)| r & e) // Perform element-wise dot product (mod 2)
                .fold(0, |acc, x| acc ^ x) // XOR all elements to get the syndrome
        })
        .collect();
    syndrome
}

pub fn generate_subsets(indices: &[usize], size: usize) -> impl Iterator<Item = Vec<usize>> + '_ {
    indices.iter().cloned().combinations(size)
}

pub fn calculate_weight(vec: &[u8]) -> usize {
    vec.iter().filter(|&&bit| bit == 1).count()
}

pub fn validate_syndrome(e: &[u8], s: &Array1<u8>, h: &Array2<u8>) -> bool {
    let syndrome = h.dot(&Array1::from(e.to_vec()));
    syndrome.iter().zip(s.iter()).all(|(a, b)| a == b)
}
