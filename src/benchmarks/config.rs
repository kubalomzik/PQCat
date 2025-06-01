use crate::types::BenchmarkConfig;

#[allow(dead_code)]
impl BenchmarkConfig {
    // ==================== HAMMING CODE CONFIGURATIONS ====================

    // Hamming code with scaling code size
    pub fn hamming_scaling_size(size_index: usize) -> Self {
        let sizes = [(7, 4), (15, 11), (31, 26), (63, 57)];
        let (n, k) = sizes[size_index];

        Self {
            n,
            k,
            w: 1,
            code_type: "hamming".to_string(),
            ..Self::default()
        }
    }

    // Hamming code with scaling error weight
    pub fn hamming_scaling_weight(weight_index: usize) -> Self {
        let weights = [1, 3, 5, 7];

        Self {
            n: 31,
            k: 26,
            w: weights[weight_index],
            code_type: "hamming".to_string(),
            ..Self::default()
        }
    }

    // ==================== GOPPA CODE CONFIGURATIONS ====================

    // Goppa code with scaling code size
    pub fn goppa_scaling_size(size_index: usize) -> Self {
        let params = [
            (15, 7, 2),    // n=15, m=4 => k ≤ 15-4*2 = 7
            (31, 21, 2),   // n=31, m=5 => k ≤ 31-5*2 = 21
            (63, 51, 2),   // n=63, m=6 => k ≤ 63-6*2 = 51
            (127, 113, 2), // n=127, m=7 => k ≤ 127-7*2 = 113
        ];
        let (n, k, w) = params[size_index];

        Self {
            n,
            k,
            w,
            code_type: "goppa".to_string(),
            ..Self::default()
        }
    }

    // Goppa code with scaling error correction capability
    pub fn goppa_scaling_weight(weight_index: usize) -> Self {
        let params = [
            (62, 56, 1), // For t=1, max support is 62, not 63
            (63, 51, 2), // n - m*t = 63 - 6*2 = 51, so k=51 is valid
            (63, 45, 3), // n - m*t = 63 - 6*3 = 45, so k=45 is valid
            (63, 39, 4), // n - m*t = 63 - 6*4 = 39, so k=39 is valid
        ];
        let (n, k, w) = params[weight_index];

        Self {
            n,
            k,
            w,
            code_type: "goppa".to_string(),
            ..Self::default()
        }
    }

    // ==================== QUASI-CYCLIC CODE CONFIGURATIONS ====================

    // Quasi-Cyclic code with scaling code size
    pub fn qc_scaling_size(size_index: usize) -> Self {
        let params = [
            (30, 20, 2), // (n, k, w)
            (60, 40, 2),
            (90, 60, 2),
            (120, 80, 2),
        ];
        let (n, k, w) = params[size_index];

        Self {
            n,
            k,
            w,
            code_type: "qc".to_string(),
            ..Self::default()
        }
    }

    // Quasi-Cyclic code with scaling error weight
    pub fn qc_scaling_weight(weight_index: usize) -> Self {
        let weights = [1, 2, 3, 4];
        let w = weights[weight_index];

        Self {
            n: 60,
            k: 40,
            w,
            code_type: "qc".to_string(),
            ..Self::default()
        }
    }

    // ==================== MMT CONFIGURATION ====================

    // MMT algorithm configuration
    pub fn mmt_config(n: usize, k: usize, w: usize, code_type: &str) -> Self {
        Self {
            runs: 100,
            algorithm_name: "mmt".to_string(),
            n,
            k,
            w,
            code_type: code_type.to_string(),
            p: Some(2),
            l1: Some(256),
            l2: Some(256),
        }
    }

    // ==================== REAL-WORLD CONFIGURATIONS ====================

    // Real-world Goppa parameters (adjusted for field size and constraints)
    pub fn real_world_goppa(security_level: usize) -> Self {
        let params = [
            (2047, 1695, 27),  // ~80-bit classical security (k reduced)
            (3487, 2719, 64),  // ~128-bit classical / NIST Level 1 (k reduced by 1)
            (4095, 3359, 96),  // ~192-bit classical / NIST Level 3 (k reduced)
            (6939, 5412, 119), // ~256-bit classical / NIST Level 5 (k reduced)
        ];
        let (n, k, w) = params[security_level];
        
        Self {
            n,
            k,
            w,
            code_type: "goppa".to_string(),
            ..Self::default()
        }
    }

    // Real-world QC-MDPC parameters (adjusted for field size and QC constraints)
    pub fn real_world_qc(security_level: usize) -> Self {
        let params = [
            (8190, 4095, 142),   // NIST Level 1 (n=2*4095, k=1*4095)
            (16382, 8191, 159),  // NIST Level 3 (n=2*8191, k=1*8191)
            (24573, 8191, 199),  // NIST Level 5 (n=3*8191, k=1*8191)
        ];
        let (n, k, w) = params[security_level];
        
        Self {
            n,
            k,
            w,
            code_type: "qc".to_string(),
            ..Self::default()
        }
    }

    // ==================== BUILDER METHODS ====================

    // Set algorithm
    pub fn with_algorithm(mut self, alg: &str) -> Self {
        self.algorithm_name = alg.to_string();
        self
    }

    // Set number of runs
    pub fn with_runs(mut self, runs: usize) -> Self {
        self.runs = runs;
        self
    }

    // Set MMT parameters
    pub fn with_mmt_params(mut self, p: usize, l1: usize, l2: usize) -> Self {
        self.p = Some(p);
        self.l1 = Some(l1);
        self.l2 = Some(l2);
        self
    }
}
