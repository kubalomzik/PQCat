// FiniteField implementation for field element operations
#[derive(Clone)]
pub struct FiniteField {
    pub m: u8,     // Extension degree (field is GF(2^m))
    pub poly: u16, // Irreducible polynomial represented as a bit pattern
}

#[derive(Clone)]
pub struct GoppaParams {
    pub field: FiniteField,
    pub goppa_poly: Vec<u8>,
    pub support: Vec<u8>,
    pub t: usize,
}

#[derive(Clone)]
pub struct CodeParams {
    pub n: usize,
    pub k: usize,
    pub w: usize,
    pub code_type: String,
}

#[derive(Clone)]
pub struct PartitionParams {
    pub p: Option<usize>,
    pub l1: Option<usize>,
    pub l2: Option<usize>,
}

impl Default for PartitionParams {
    fn default() -> Self {
        Self {
            p: Some(2),
            l1: Some(1),
            l2: Some(1),
        }
    }
}
