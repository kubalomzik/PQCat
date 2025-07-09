// FiniteField implementation for field element operations
#[derive(Clone)]
pub struct FiniteField {
    pub m: u8,     // Extension degree (field is GF(2^m))
    pub poly: u32, // Irreducible polynomial represented as a bit pattern
}

#[derive(Clone)]
pub struct GoppaParams {
    pub field: FiniteField,
    pub goppa_poly: Vec<u32>,
    pub support: Vec<u32>,
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

pub struct BenchmarkConfig {
    pub runs: usize,
    pub algorithm_name: String,
    pub n: usize,
    pub k: usize,
    pub w: usize,
    pub code_type: String,
    // Optional parameters only for MMT
    pub p: Option<usize>,
    pub l1: Option<usize>,
    pub l2: Option<usize>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            runs: 100,
            algorithm_name: "prange".to_string(),
            n: 15,
            k: 11,
            w: 1,
            code_type: "hamming".to_string(),
            p: None,
            l1: None,
            l2: None,
        }
    }
}

pub struct BenchmarkResult {
    pub duration: u64,
    pub memory: u64,
    pub success: bool,
}

pub struct BenchmarkStats {
    pub median_time: f64,
    pub median_memory: f64,
    pub success_rate: f64,
    pub successful_runs: usize,
    pub completed_runs: usize,
    pub time_ci_lower: f64,
    pub time_ci_upper: f64,
    pub memory_ci_lower: f64,
    pub memory_ci_upper: f64,
}
