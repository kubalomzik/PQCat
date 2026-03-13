use itertools::Itertools;
use ndarray::Array2;
use rand::rng;
use rand::seq::SliceRandom;

pub fn generate_random_error_vector(n: usize, weight: usize) -> Vec<u8> {
    assert!(
        weight <= n,
        "Weight must be less than or equal to the length of the vector"
    );

    let mut indices: Vec<usize> = (0..n).collect();
    indices.shuffle(&mut rng());
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

pub fn syndrome_distance(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x ^ y) as usize)
        .sum()
}

/// Calculate syndrome contribution from a subset of columns
pub fn calculate_partial_syndrome(h: &Array2<u8>, indices: &[usize], r: usize) -> Vec<u8> {
    let mut syndrome = vec![0; r];

    for &idx in indices {
        for j in 0..r {
            syndrome[j] ^= h[[j, idx]];
        }
    }

    syndrome
}

pub fn permute_columns(matrix: &Array2<u8>, permutation: &[usize]) -> Array2<u8> {
    let (rows, cols) = matrix.dim();
    assert_eq!(cols, permutation.len());
    let mut permuted = Array2::<u8>::zeros((rows, cols));
    for (new_idx, &old_idx) in permutation.iter().enumerate() {
        let column = matrix.column(old_idx).to_owned();
        permuted.column_mut(new_idx).assign(&column);
    }
    permuted
}

pub fn xor_assign(target: &mut Vec<u8>, other: &[u8]) {
    assert_eq!(target.len(), other.len());
    for (t, &o) in target.iter_mut().zip(other.iter()) {
        *t ^= o;
    }
}

pub fn apply_inverse_permutation(vector: &[u8], permutation: &[usize]) -> Vec<u8> {
    assert_eq!(vector.len(), permutation.len());
    let mut original = vec![0u8; vector.len()];
    for (idx, &orig_pos) in permutation.iter().enumerate() {
        original[orig_pos] = vector[idx];
    }
    original
}
