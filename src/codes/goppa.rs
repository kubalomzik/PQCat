use crate::codes::polynomial_utils::{evaluate_poly, random_irreducible_poly};
use crate::types::FiniteField;
use ndarray::Array2;

/// Helper function for creating valid Goppa code parameters
pub fn generate_valid_goppa_params(n: usize, t: usize) -> (Vec<u8>, Vec<u8>, FiniteField) {
    let m = (n as f64).log2().ceil() as u8;
    let field = FiniteField::new(m);

    // Generate Goppa polynomial and support
    let mut goppa_poly = random_irreducible_poly(t, &field);
    let mut support = field.random_support(n);

    // Validate: ensure no support element is a root of the Goppa polynomial
    let mut retry_count = 0;
    const MAX_RETRIES: usize = 10;

    while retry_count < MAX_RETRIES {
        let mut has_root_in_support = false;

        for &x in &support {
            let g_x = evaluate_poly(&goppa_poly, x, &field);
            if g_x == 0 {
                has_root_in_support = true;
                break;
            }
        }

        if !has_root_in_support {
            // Valid combination found
            return (goppa_poly, support, field);
        }

        // Try again with a new polynomial
        goppa_poly = random_irreducible_poly(t, &field);
        retry_count += 1;
    }

    // If we're still having issues after several retries, regenerate both
    goppa_poly = random_irreducible_poly(t, &field);
    support = field.random_support(n);

    // Final validation - in extreme cases, we just ensure individual elements work
    let mut valid_support = Vec::with_capacity(n);
    for &x in &support {
        let g_x = evaluate_poly(&goppa_poly, x, &field);
        if g_x != 0 {
            valid_support.push(x);
            if valid_support.len() >= n {
                break;
            }
        }
    }

    // If we couldn't find enough valid support elements
    if valid_support.len() < n {
        panic!("Couldn't generate valid Goppa code parameters after multiple attempts");
    }

    (goppa_poly, valid_support, field)
}

pub fn generate_goppa_parity_matrix(
    n: usize,
    t: usize,
    goppa_poly: &[u8],
    support: &[u8],
    field: &FiniteField,
) -> Array2<u8> {
    // The parity check matrix for a binary Goppa code has t*m rows
    let m = field.get_m() as usize;
    let mut h = Array2::<u8>::zeros((t * m, n));

    for j in 0..n {
        // For each support element L[j]
        let l_j = support[j];

        // Calculate g(L[j])
        let g_l_j = evaluate_poly(goppa_poly, l_j, field);

        // Calculate 1/g(L[j])
        let inv_g_l_j = field.inverse(g_l_j);

        // Generate the column with format [L[j]^(t-1)/g(L[j]), ..., L[j]/g(L[j]), 1/g(L[j])]
        let mut power = 1u8; // Start with L[j]^0 = 1

        for i in 0..t {
            let col_val = field.field_multiply(power, inv_g_l_j);

            // Convert to binary and place in the appropriate rows
            for bit in 0..m {
                h[[i * m + bit, j]] = (col_val >> bit) & 1;
            }

            // Calculate next power
            power = field.field_multiply(power, l_j);
        }
    }

    h
}
