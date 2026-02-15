use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Signal {
    pub kind: SignalKind,
    pub severity: f64,       // 0.0-1.0, detector computes it
    pub message: String,     // human-readable one-liner
    pub commits: Vec<String>, // short hashes involved
    pub files: Vec<String>,  // file paths involved
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    FixAfterFeat,
    FixAfterRefactor,
}
