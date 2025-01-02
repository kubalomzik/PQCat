use ndarray::s;
use ndarray::{Array2, Axis};
use rand::seq::SliceRandom;
use rand::Rng;
use std::process;

pub fn generate_code(n: usize, k: usize, w: usize, code_type: String) -> (Array2<u8>, Array2<u8>) {
    let (g, h);
    match code_type.as_str() {
        "random" => {
            (g, h) = generate_random_code(n, k);
        }
        "hamming" => {
            (g, h) = generate_hamming_code(n, k);
        }
        "goppa" => {
            (g, h) = generate_goppa_code(n, k, w);
        }
        _ => {
            println!("Error: Unsupported code type '{}'", code_type);
            process::exit(1);
        }
    }
    (g, h)
}

pub fn generate_random_code(n: usize, k: usize) -> (Array2<u8>, Array2<u8>) {
    assert!(k < n, "k must be less than n");
    let mut rng = rand::thread_rng();
    let m = n - k; // Number of parity bits

    // Generate a random (k x m) P matrix
    let p = Array2::from_shape_fn((k, m), |_| rng.gen_range(0..=1));

    // Construct G = [I_k | P]
    let mut g = Array2::<u8>::zeros((k, n));
    for i in 0..k {
        g[(i, i)] = 1; // Identity part
    }
    for i in 0..k {
        for j in 0..m {
            g[(i, k + j)] = p[(i, j)];
        }
    }

    // Construct H = [-P^T | I_m]
    let mut h = Array2::<u8>::zeros((m, n));
    for i in 0..(m) {
        h[(i, k + i)] = 1; // Identity part
    }
    for i in 0..k {
        for j in 0..m {
            h[(j, i)] = p[(i, j)]; // -P^T part (no need for negative in GF(2))
        }
    }

    (g, h)
}

pub fn generate_hamming_code(n: usize, k: usize) -> (Array2<u8>, Array2<u8>) {
    let m = n - k; // Number of parity bits

    // Create parity-check matrix H (m x n)
    let mut h = Array2::<u8>::zeros((m, n));
    for col in 0..n {
        let binary_repr = format!("{:0>width$b}", col + 1, width = m); // Columns are binary numbers (1 to n)
        for row in 0..m {
            h[[row, col]] = binary_repr.as_bytes()[row] - b'0';
        }
    }

    // Rearrange H into systematic form [P^T | I_m]
    let p_t = h.slice(s![.., ..k]).to_owned(); // Extract P^T (m x k)
    let identity = Array2::<u8>::eye(m); // Identity matrix (m x m)
    let mut systematic_h = Array2::<u8>::zeros((m, n));
    systematic_h.slice_mut(s![.., ..k]).assign(&p_t); // Place P^T in the left part
    systematic_h.slice_mut(s![.., k..]).assign(&identity); // Place I_m in the right part

    // Compute generator matrix G (k x n)
    // G = [I_k | P]
    let p = p_t.t().to_owned(); // Transpose of P^T (k x m)
    let identity_k = Array2::<u8>::eye(k); // Identity matrix (k x k)
    let g = ndarray::concatenate(Axis(1), &[identity_k.view(), p.view()]).unwrap(); // Concatenate I_k and P along columns

    (g, systematic_h)
}

pub fn generate_goppa_code(n: usize, _k: usize, w: usize) -> (Array2<u8>, Array2<u8>) {
    let t = w; // Weight corresponds to the degree of g(x) (Goppa polynomial)

    // Generate a random irreducible g(x)
    // Here, coefficients for g(x) are created randomly. In practice, you should verify irreducibility.
    let mut rng = rand::thread_rng();
    let goppa_poly: Vec<u8> = (0..=t).map(|_| rng.gen_range(0..2)).collect();

    // Generate a random support set L of size n
    // Support set must be unique elements in GF(2).
    let mut support: Vec<u8> = (0..n as u8).collect();
    support.shuffle(&mut rng);

    let h = generate_goppa_code_matrix(n, t, &goppa_poly, &support);
    let g = generate_generator_matrix(&h);

    (g, h)
}

fn generate_goppa_code_matrix(n: usize, t: usize, goppa_poly: &[u8], support: &[u8]) -> Array2<u8> {
    let mut h = Array2::<u8>::zeros((t, n));

    // Compute S(i, j) = 1 / g(L[j]) mod 2
    for (j, &l) in support.iter().enumerate() {
        let inv = modular_inverse(l, 2); // Compute modular inverse in GF(2)
        for i in 0..t {
            h[[i, j]] = (goppa_poly[i] * inv) % 2; // Compute H(i, j)
        }
    }

    h
}

fn modular_inverse(x: u8, mod_poly: u8) -> u8 {
    let mut a = x;
    let mut b = mod_poly;
    let mut u = 1u8;
    let mut v = 0u8;

    while a != 0 {
        let q = b / a;
        (b, a) = (a, b % a);
        (v, u) = (u, v ^ (q * u));
    }

    v
}

fn generate_generator_matrix(h: &Array2<u8>) -> Array2<u8> {
    let (m, n) = h.dim();
    let k = n - m; // Dimension of the code

    // Extract P and construct G = [I_k | P]
    let p = h.slice(s![..m, ..k]).to_owned(); // Extract P (m x k)
    let identity_k = Array2::<u8>::eye(k); // Identity matrix (k x k)
    ndarray::concatenate(Axis(1), &[identity_k.view(), p.view()]).unwrap()
}
