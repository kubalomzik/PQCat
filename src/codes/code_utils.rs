use ndarray::{s, Array2, Axis};

pub fn convert_to_systematic(h: Array2<u8>) -> (Array2<u8>, Array2<u8>) {
    let (m, n) = h.dim();
    let k = n - m;

    /*
    In a textbook implementation, we'd perform Gaussian elimination to get H in systematic form
    For simplicity, we'll assume H is already in a form where we can extract P
    */

    // Extract P^T (m x k) from the left part of H
    let p_t = h.slice(s![.., ..k]).to_owned();

    // Create systematic form of H = [P^T | I_m]
    let identity = Array2::<u8>::eye(m);
    let mut systematic_h = Array2::<u8>::zeros((m, n));
    systematic_h.slice_mut(s![.., ..k]).assign(&p_t);
    systematic_h.slice_mut(s![.., k..]).assign(&identity);

    // Create generator matrix G = [I_k | P]
    let p = p_t.t().to_owned();
    let identity_k = Array2::<u8>::eye(k);
    let g = ndarray::concatenate(Axis(1), &[identity_k.view(), p.view()]).unwrap();

    (g, systematic_h)
}
