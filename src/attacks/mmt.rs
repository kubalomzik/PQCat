use std::collections::HashSet;

use itertools::Itertools;
use ndarray::{Array1, Array2};

use crate::attacks::attack_utils::{calculate_weight, validate_syndrome};

pub fn extract_q_i(h_tilde: &Array2<u8>, indices: &HashSet<usize>) -> Array2<u8> {
    let rows = h_tilde.shape()[0];
    let mut submatrix_data = Vec::new();

    for &index in indices {
        submatrix_data.extend(h_tilde.column(index).iter());
    }

    Array2::from_shape_vec((rows, indices.len()), submatrix_data).unwrap()
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
        if column_sum
            .iter()
            .map(|&x: &u8| x as usize)
            .filter(|&x| x == 1)
            .count()
            == l
        {
            solutions.push(subset.into_iter().collect::<HashSet<usize>>());
        }
    }

    solutions
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

pub fn run_mmt_algorithm(
    h: &Array2<u8>,
    s: &Array1<u8>,
    n: usize,
    w: usize,
    p: usize,
    l1: usize,
    l2: usize,
) -> Option<Vec<u8>> {
    let max_iterations = 100;

    for _ in 0..max_iterations {
        let h_tilde = compute_h_tilde(h, p);

        // Find column subsets matching conditions
        let solutions = column_match(&h_tilde, p, w, l1, l2);
        for i in solutions {
            let q_i = extract_q_i(&h_tilde, &i);

            // Check weight condition
            if calculate_weight(q_i.as_slice().unwrap()) + i.len() == w {
                // Construct error vector
                let mut e_tilde = vec![0; n];
                for &index in &i {
                    e_tilde[index] = 1;
                }
                for j in q_i {
                    e_tilde[j as usize] = 1;
                }

                if validate_syndrome(&e_tilde, s, h) {
                    return Some(e_tilde);
                }
            }
        }
    }

    None
}
