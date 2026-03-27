use ndarray::{Array2, Axis, s};

pub fn convert_to_systematic(h: Array2<u8>) -> (Array2<u8>, Array2<u8>) {
    let (m, n) = h.dim();
    let k = n - m;

    let mut working_h = h;
    let mut pivot_columns: Vec<usize> = Vec::with_capacity(m);
    let mut row = 0;

    for col in 0..n {
        if row == m {
            break;
        }

        // Locate a pivot in the current column at or below the active row
        if let Some(pivot_row) = (row..m).find(|&r| working_h[(r, col)] == 1) {
            if pivot_row != row {
                // Swap rows to bring the pivot into place
                for c in 0..n {
                    let tmp = working_h[(row, c)];
                    working_h[(row, c)] = working_h[(pivot_row, c)];
                    working_h[(pivot_row, c)] = tmp;
                }
            }

            // Clear all other 1s in this column (GF(2) elimination)
            for r in 0..m {
                if r != row && working_h[(r, col)] == 1 {
                    for c in 0..n {
                        working_h[(r, c)] ^= working_h[(row, c)];
                    }
                }
            }

            pivot_columns.push(col);
            row += 1;
        }
    }

    assert!(
        pivot_columns.len() == m,
        "Parity-check matrix is not full rank; cannot convert to systematic form"
    );

    let mut is_pivot = vec![false; n];
    for &col in &pivot_columns {
        is_pivot[col] = true;
    }

    // Reorder columns so non-pivot columns precede pivot columns, yielding [P^T | I]
    let mut column_order: Vec<usize> = Vec::with_capacity(n);
    for (col, &is_piv) in is_pivot.iter().enumerate().take(n) {
        if !is_piv {
            column_order.push(col);
        }
    }
    for &col in &pivot_columns {
        column_order.push(col);
    }

    let mut systematic_h = Array2::<u8>::zeros((m, n));
    for (new_idx, &old_idx) in column_order.iter().enumerate() {
        let source_col = working_h.column(old_idx);
        systematic_h.column_mut(new_idx).assign(&source_col);
    }

    // After permutation systematic_h = [P^T | I_m]
    let p_t = systematic_h.slice(s![.., ..k]).to_owned();

    let identity_k = Array2::<u8>::eye(k);
    let p = p_t.t().to_owned();
    let g = ndarray::concatenate(Axis(1), &[identity_k.view(), p.view()]).unwrap();

    (g, systematic_h)
}
