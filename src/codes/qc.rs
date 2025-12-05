use ndarray::Array2;
use rand::{rng, seq::SliceRandom};

const MAX_QC_ATTEMPTS: usize = 64;
const HQC_ROW_WEIGHTS: &[(usize, usize)] = &[(35338, 90), (71702, 114), (115274, 149)];

pub fn generate_hqc_qc_mdpc(n: usize, k: usize) -> Result<(Array2<u8>, Array2<u8>), String> {
    let r = n
        .checked_sub(k)
        .ok_or_else(|| "Invalid parameters: k must be <= n".to_string())?;

    if n != 2 * r || k != r {
        return Err(format!(
            "Invalid HQC QC-MDPC parameters: expected n=2r and k=r, got n={}, k={}, r={}",
            n, k, r
        ));
    }

    let row_weight = row_weight_for_length(n).unwrap_or_else(|| {
        let fallback = fallback_row_weight(r);
        eprintln!(
            "Warning: no official HQC row weight for n={}; using fallback weight {}.",
            n, fallback
        );
        fallback
    });

    let mut rng = rng();

    for _ in 0..MAX_QC_ATTEMPTS {
        let mut h_vector = vec![0u8; r];

        let mut positions: Vec<usize> = (0..r).collect();
        positions.shuffle(&mut rng);
        for &idx in positions.iter().take(row_weight) {
            h_vector[idx] = 1;
        }

        if !is_circulant_invertible(&h_vector) {
            continue;
        }

        let circulant = build_circulant(&h_vector);
        let h = assemble_parity_check(&circulant);
        let g = assemble_generator(&circulant);

        return Ok((g, h));
    }

    Err("Failed to generate invertible HQC QC-MDPC matrix after multiple attempts".to_string())
}

fn row_weight_for_length(n: usize) -> Option<usize> {
    HQC_ROW_WEIGHTS
        .iter()
        .find(|(length, _)| *length == n)
        .map(|(_, weight)| *weight)
}

fn fallback_row_weight(r: usize) -> usize {
    let candidate = std::cmp::max(1, r / 4);
    std::cmp::min(candidate, r)
}

fn build_circulant(first_row: &[u8]) -> Array2<u8> {
    let r = first_row.len();
    let mut circulant = Array2::<u8>::zeros((r, r));

    for row in 0..r {
        for col in 0..r {
            let idx = (col + row) % r;
            circulant[[row, col]] = first_row[idx];
        }
    }

    circulant
}

fn assemble_parity_check(circulant: &Array2<u8>) -> Array2<u8> {
    let r = circulant.shape()[0];
    let mut h = Array2::<u8>::zeros((r, 2 * r));

    for i in 0..r {
        h[(i, i)] = 1;
    }

    for row in 0..r {
        for col in 0..r {
            h[(row, r + col)] = circulant[[row, col]];
        }
    }

    h
}

fn assemble_generator(circulant: &Array2<u8>) -> Array2<u8> {
    let r = circulant.shape()[0];
    let mut g = Array2::<u8>::zeros((r, 2 * r));

    for row in 0..r {
        for col in 0..r {
            g[(row, col)] = circulant[[col, row]];
        }
    }

    for i in 0..r {
        g[(i, r + i)] = 1;
    }

    g
}

fn is_circulant_invertible(h_vector: &[u8]) -> bool {
    let r = h_vector.len();

    let mut modulus = vec![0u8; r + 1];
    modulus[0] = 1;
    modulus[r] = 1;

    let mut poly = h_vector.to_vec();
    trim_polynomial(&mut poly);
    if poly.is_empty() {
        return false;
    }

    let mut modulus_trimmed = modulus;
    trim_polynomial(&mut modulus_trimmed);

    let gcd = polynomial_gcd(poly, modulus_trimmed);
    gcd.len() == 1 && gcd[0] == 1
}

fn polynomial_gcd(mut a: Vec<u8>, mut b: Vec<u8>) -> Vec<u8> {
    trim_polynomial(&mut a);
    trim_polynomial(&mut b);

    while !b.is_empty() {
        let r = polynomial_mod(a, &b);
        a = b;
        b = r;
    }

    if a.is_empty() { vec![0] } else { a }
}

fn polynomial_mod(dividend: Vec<u8>, divisor: &[u8]) -> Vec<u8> {
    if divisor.is_empty() {
        return vec![];
    }

    let mut result = dividend;
    trim_polynomial(&mut result);

    let Some(divisor_deg) = polynomial_degree(divisor) else {
        return vec![];
    };

    while let Some(result_deg) = polynomial_degree(&result) {
        if result_deg < divisor_deg {
            break;
        }
        let shift = result_deg - divisor_deg;
        for i in 0..=divisor_deg {
            if let Some(coef) = divisor.get(i) {
                result[shift + i] ^= coef;
            }
        }
        trim_polynomial(&mut result);
    }

    result
}

fn polynomial_degree(poly: &[u8]) -> Option<usize> {
    for (idx, &coef) in poly.iter().enumerate().rev() {
        if coef & 1 == 1 {
            return Some(idx);
        }
    }
    None
}

fn trim_polynomial(poly: &mut Vec<u8>) {
    while let Some(&last) = poly.last() {
        if last == 0 {
            poly.pop();
        } else {
            break;
        }
    }
}
