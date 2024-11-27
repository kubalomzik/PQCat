use ndarray::{Array1, Array2};
use rand::seq::SliceRandom;
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
