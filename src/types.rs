use clap::ValueEnum;
use std::fmt;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Algorithm {
    Prange,
    Stern,
    LeeBrickell,
    BallCollision,
    Bjmm,
    Patterson,
    Mmt,
    FiniaszSendrier,
}

impl Algorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Algorithm::Prange => "prange",
            Algorithm::Stern => "stern",
            Algorithm::LeeBrickell => "lee-brickell",
            Algorithm::BallCollision => "ball-collision",
            Algorithm::Bjmm => "bjmm",
            Algorithm::Patterson => "patterson",
            Algorithm::Mmt => "mmt",
            Algorithm::FiniaszSendrier => "finiasz-sendrier",
        }
    }

    pub fn as_cli_subcommand(&self) -> &'static str {
        // Clap uses the kebab-case representation for subcommand names
        self.as_str()
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum CodeType {
    Random,
    Hamming,
    Goppa,
    Qc,
}

impl CodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeType::Random => "random",
            CodeType::Hamming => "hamming",
            CodeType::Goppa => "goppa",
            CodeType::Qc => "qc",
        }
    }
}

impl fmt::Display for CodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum CodePreset {
    ClassicMceliece348864,
    ClassicMceliece460896,
    ClassicMceliece6688128,
    Hqc128,
    Hqc192,
    Hqc256,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CodePresetParams {
    pub code_type: CodeType,
    pub n: usize,
    pub k: usize,
    pub w: usize,
}

impl CodePreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodePreset::ClassicMceliece348864 => "classic-mceliece-348864",
            CodePreset::ClassicMceliece460896 => "classic-mceliece-460896",
            CodePreset::ClassicMceliece6688128 => "classic-mceliece-6688128",
            CodePreset::Hqc128 => "hqc-128",
            CodePreset::Hqc192 => "hqc-192",
            CodePreset::Hqc256 => "hqc-256",
        }
    }

    pub fn params(&self) -> CodePresetParams {
        match self {
            CodePreset::ClassicMceliece348864 => CodePresetParams {
                code_type: CodeType::Goppa,
                n: 3488,
                k: 2720,
                w: 64,
            },
            CodePreset::ClassicMceliece460896 => CodePresetParams {
                code_type: CodeType::Goppa,
                n: 4608,
                k: 3360,
                w: 96,
            },
            CodePreset::ClassicMceliece6688128 => CodePresetParams {
                code_type: CodeType::Goppa,
                n: 6688,
                k: 5024,
                w: 128,
            },
            CodePreset::Hqc128 => CodePresetParams {
                code_type: CodeType::Qc,
                n: 35338,
                k: 17669,
                w: 128,
            },
            CodePreset::Hqc192 => CodePresetParams {
                code_type: CodeType::Qc,
                n: 71702,
                k: 35851,
                w: 192,
            },
            CodePreset::Hqc256 => CodePresetParams {
                code_type: CodeType::Qc,
                n: 115274,
                k: 57637,
                w: 256,
            },
        }
    }
}

impl fmt::Display for CodePreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone)]
pub struct CodeParams {
    pub n: usize,
    pub k: usize,
    pub w: usize,
    pub code_type: CodeType,
}

impl CodeParams {
    pub fn apply_preset(&mut self, preset: CodePreset) -> CodePresetParams {
        let params = preset.params();
        self.n = params.n;
        self.k = params.k;
        self.w = params.w;
        self.code_type = params.code_type;
        params
    }
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

#[derive(Clone)]
pub struct BenchmarkConfig {
    pub runs: usize,
    pub algorithm: Algorithm,
    pub n: usize,
    pub k: usize,
    pub w: usize,
    pub code_type: CodeType,
    // Optional parameters only for MMT
    pub p: Option<usize>,
    pub l1: Option<usize>,
    pub l2: Option<usize>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            runs: 100,
            algorithm: Algorithm::Prange,
            n: 15,
            k: 11,
            w: 1,
            code_type: CodeType::Hamming,
            p: None,
            l1: None,
            l2: None,
        }
    }
}

impl BenchmarkConfig {
    pub fn apply_preset(&mut self, preset: CodePreset) -> CodePresetParams {
        let params = preset.params();
        self.n = params.n;
        self.k = params.k;
        self.w = params.w;
        self.code_type = params.code_type;
        params
    }

    pub fn with_preset(mut self, preset: CodePreset) -> Self {
        self.apply_preset(preset);
        self
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
