use crate::codes::code_utils::convert_to_systematic;
use crate::codes::goppa::{generate_goppa_parity_matrix, FiniteField};
use ndarray::s;
use ndarray::{Array2, Axis};
use rand::seq::SliceRandom;
use rand::Rng;
use std::process;

fn handle_code_result<T>(result: Result<T, String>, code_type: &str) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error generating {} code: {}", code_type, e);
            process::exit(1);
        }
    }
}

pub fn generate_code(n: usize, k: usize, w: usize, code_type: String) -> (Array2<u8>, Array2<u8>) {
    match code_type.as_str() {
        "random" => handle_code_result(generate_random_code(n, k), "random"),
        "hamming" => handle_code_result(generate_hamming_code(n, k), "hamming"),
        "goppa" => handle_code_result(generate_goppa_code(n, k, w), "goppa"),
        "qc" => handle_code_result(generate_qc_code(n, k), "qc"),
        _ => {
            eprintln!("Error: Unsupported code type '{}'", code_type);
            process::exit(1);
        }
    }
}

pub fn generate_random_code(n: usize, k: usize) -> Result<(Array2<u8>, Array2<u8>), String> {
    assert!(k < n, "k must be less than n");
    let mut rng = rand::thread_rng();
    let m = n - k; // Number of parity bits

    let p = Array2::from_shape_fn((k, m), |_| rng.gen_range(0..=1)); // Generate a random (k x m) P matrix

    let mut g = Array2::<u8>::zeros((k, n)); // Construct G = [I_k | P]
    for i in 0..k {
        g[(i, i)] = 1; // Identity part
    }
    for i in 0..k {
        for j in 0..m {
            g[(i, k + j)] = p[(i, j)];
        }
    }

    let mut h = Array2::<u8>::zeros((m, n)); // Construct H = [-P^T | I_m]
    for i in 0..(m) {
        h[(i, k + i)] = 1; // Identity part
    }
    for i in 0..k {
        for j in 0..m {
            h[(j, i)] = p[(i, j)]; // -P^T part (no need for negative in GF(2))
        }
    }

    Ok((g, h))
}

pub fn generate_hamming_code(n: usize, k: usize) -> Result<(Array2<u8>, Array2<u8>), String> {
    let m = n - k; // Number of parity bits

    let mut h = Array2::<u8>::zeros((m, n)); // Create parity-check matrix H (m x n)
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

    // Compute generator matrix G (k x n) = [I_k | P]
    let p = p_t.t().to_owned(); // Transpose of P^T (k x m)
    let identity_k = Array2::<u8>::eye(k); // Identity matrix (k x k)
    let g = ndarray::concatenate(Axis(1), &[identity_k.view(), p.view()]).unwrap(); // Concatenate I_k and P along columns

    Ok((g, systematic_h))
}

pub fn generate_goppa_code(
    n: usize,
    k: usize,
    t: usize,
) -> Result<(Array2<u8>, Array2<u8>), String> {
    let m = (n as f64).log2().ceil() as u8; // Determine the field size m such that 2^m > n

    if k > n - (m as usize) * t {
        return Err(format!(
            "Invalid Goppa code parameters: k ({}) must be ≤ n - m*t ({} - {}*{} = {})",
            k,
            n,
            m,
            t,
            n - (m as usize) * t
        ));
    }

    let field = FiniteField::new(m); // Create a finite field GF(2^m)

    let goppa_poly = field.random_irreducible_poly(t); // Generate a random irreducible polynomial of degree t

    let mut support = field.random_support(n); // Generate a random support set L of size n (distinct elements from GF(2^m))

    // Validate that the Goppa polynomial has no roots in the support set
    let mut valid = false;
    while !valid {
        valid = true;
        for j in 0..support.len() {
            let l_j = support[j];
            let g_l_j = field.evaluate_poly(&goppa_poly, l_j);
            if g_l_j == 0 {
                // Found a root of g(z) in the support set
                valid = false;

                let field_size = 1 << field.get_m();
                let mut rng = rand::thread_rng();
                let mut new_element;
                loop {
                    new_element = rng.gen_range(1..field_size) as u8;
                    if !support.contains(&new_element) {
                        break;
                    }
                }
                support[j] = new_element;

                break;
            }
        }
    }

    let h = generate_goppa_parity_matrix(n, t, &goppa_poly, &support, &field);

    let (g, h_systematic) = convert_to_systematic(h); // Convert H to systematic form and derive the generator matrix

    Ok((g, h_systematic))
}

pub fn generate_qc_code(n: usize, k: usize) -> Result<(Array2<u8>, Array2<u8>), String> {
    let r = n - k; // Number of parity bits

    if n % r != 0 || k % r != 0 {
        return Err(format!(
            "Invalid QC code parameters: both n ({}) and k ({}) should be multiples of r ({})",
            n, k, r
        ));
    }

    let p = r; // Block size for circulant matrices (using r for simplicity)
    let num_blocks_cols = n / p;
    let num_blocks_rows = r / p;

    if num_blocks_rows != 1 {
        return Err(format!(
            "For simplicity, this implementation requires r=p, got r={}, p={}",
            r, p
        ));
    }

    // Create the parity-check matrix composed of circulant blocks
    let mut h = Array2::<u8>::zeros((r, n));

    for block_col in 0..num_blocks_cols {
        // For each block column, generate a random first row
        let mut first_row = vec![0; p];

        // Make it sparse for better error correction (typically 2-3 1s per row)
        let ones_per_row = 2.min(p / 2);
        let mut indices: Vec<usize> = (0..p).collect();
        indices.shuffle(&mut rand::thread_rng());

        for &idx in indices.iter().take(ones_per_row) {
            first_row[idx] = 1;
        }

        // Fill the block with cyclic shifts of the first row
        for row in 0..p {
            for col in 0..p {
                let shifted_col = (col + row) % p;
                h[[row, block_col * p + col]] = first_row[shifted_col];
            }
        }
    }

    // Ensure the last block is invertible by making it the identity matrix
    for i in 0..p {
        for j in 0..p {
            h[[i, (num_blocks_cols - 1) * p + j]] = if i == j { 1 } else { 0 };
        }
    }

    // Convert to systematic form and derive generator matrix
    let (g, h_systematic) = convert_to_systematic(h);

    Ok((g, h_systematic))
}
