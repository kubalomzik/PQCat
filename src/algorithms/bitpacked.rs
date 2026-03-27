use ndarray::Array2;

const BITS: usize = 64;

/// Pack a slice of u8 bits (each 0 or 1) into a Vec<u64>
/// Each u64 holds up to 64 bits, MSB-first within each word
#[inline]
pub fn pack_bits(bits: &[u8]) -> Vec<u64> {
    let num_words = bits.len().div_ceil(BITS);
    let mut packed = vec![0u64; num_words];
    for (i, &b) in bits.iter().enumerate() {
        if b != 0 {
            packed[i / BITS] |= 1u64 << (i % BITS);
        }
    }
    packed
}

/// Pack each row of an Array2<u8> matrix into bitpacked form
/// Returns a Vec of packed rows, plus the number of columns (for length tracking)
pub fn pack_matrix_rows(matrix: &Array2<u8>) -> Vec<Vec<u64>> {
    matrix
        .outer_iter()
        .map(|row| pack_bits(row.as_slice().unwrap()))
        .collect()
}

/// Bitpacked syndrome calculation: h_rows × error_vector in GF(2)
/// Each row of H is AND-ed with the error vector, then popcount determines parity
#[inline]
pub fn syndrome_packed(packed_rows: &[Vec<u64>], packed_vec: &[u64]) -> Vec<u8> {
    packed_rows
        .iter()
        .map(|row| {
            let parity: u32 = row
                .iter()
                .zip(packed_vec.iter())
                .map(|(&r, &v)| (r & v).count_ones())
                .sum();
            (parity & 1) as u8
        })
        .collect()
}

/// Hamming distance between two u8 bit-vectors, computed via bitpacking
#[inline]
pub fn hamming_distance_packed(a: &[u8], b: &[u8]) -> usize {
    debug_assert_eq!(a.len(), b.len());
    let words_a = pack_bits(a);
    let words_b = pack_bits(b);
    words_a
        .iter()
        .zip(words_b.iter())
        .map(|(&wa, &wb)| (wa ^ wb).count_ones() as usize)
        .sum()
}

/// In-place XOR of two u8 bit-vectors, computed via bitpacking
/// Packs both, XORs words, then unpacks back into target
#[inline]
pub fn xor_assign_packed(target: &mut [u8], other: &[u8]) {
    debug_assert_eq!(target.len(), other.len());
    let len = target.len();
    let num_words = len.div_ceil(BITS);

    let mut packed_target = pack_bits(target);
    let packed_other = pack_bits(other);

    for i in 0..num_words {
        packed_target[i] ^= packed_other[i];
    }

    // Unpack back into target
    for i in 0..len {
        target[i] = ((packed_target[i / BITS] >> (i % BITS)) & 1) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    #[test]
    fn test_pack_unpack_roundtrip() {
        let bits: Vec<u8> = vec![1, 0, 1, 1, 0, 0, 1, 0, 1];
        let packed = pack_bits(&bits);
        // Verify each bit
        for (i, &b) in bits.iter().enumerate() {
            let unpacked = ((packed[i / BITS] >> (i % BITS)) & 1) as u8;
            assert_eq!(unpacked, b, "Mismatch at bit {i}");
        }
    }

    #[test]
    fn test_syndrome_packed_matches_naive() {
        // Small 3x5 parity check matrix
        let h = Array2::from_shape_vec((3, 5), vec![1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1])
            .unwrap();

        let error = vec![1u8, 0, 1, 0, 0];

        // Naive syndrome
        let naive: Vec<u8> = h
            .outer_iter()
            .map(|row| {
                row.iter()
                    .zip(error.iter())
                    .map(|(&r, &e)| r & e)
                    .fold(0u8, |acc, x| acc ^ x)
            })
            .collect();

        // Packed syndrome
        let packed_rows = pack_matrix_rows(&h);
        let packed_error = pack_bits(&error);
        let packed_result = syndrome_packed(&packed_rows, &packed_error);

        assert_eq!(naive, packed_result);
    }

    #[test]
    fn test_hamming_distance() {
        let a = vec![1u8, 0, 1, 1, 0];
        let b = vec![1u8, 1, 0, 1, 0];
        assert_eq!(hamming_distance_packed(&a, &b), 2);
    }

    #[test]
    fn test_xor_assign_packed() {
        let mut target = vec![1u8, 0, 1, 1, 0];
        let other = vec![0u8, 1, 1, 0, 1];
        xor_assign_packed(&mut target, &other);
        assert_eq!(target, vec![1, 1, 0, 1, 1]);
    }

    #[test]
    fn test_large_vector() {
        // Test with > 64 bits to exercise multi-word paths
        let n = 200;
        let mut bits = vec![0u8; n];
        for i in (0..n).step_by(3) {
            bits[i] = 1;
        }
        let packed = pack_bits(&bits);
        assert_eq!(packed.len(), n.div_ceil(64));
        for (i, &b) in bits.iter().enumerate() {
            let unpacked = ((packed[i / 64] >> (i % 64)) & 1) as u8;
            assert_eq!(unpacked, b, "Mismatch at bit {i}");
        }
    }
}
