use anyhow::{Context, Result};
use ort::session::Session;
use ort::value::TensorRef;
use std::collections::HashMap;
use std::path::Path;
use tokenizers::Tokenizer;

/// Default confidence threshold — ML predictions below this are discarded.
const DEFAULT_THRESHOLD: f32 = 0.5;

/// Maximum sequence length for the tokenizer (must match model training config).
const MAX_LENGTH: usize = 128;

/// ML-based commit classifier using an ONNX model.
pub struct MlClassifier {
    session: Session,
    tokenizer: Tokenizer,
    label_map: HashMap<usize, String>,
    threshold: f32,
}

impl MlClassifier {
    /// Load an ONNX model, tokenizer, and label mapping from `model_dir`.
    ///
    /// Expected files:
    /// - `model.onnx` (+ optional `model.onnx.data` for external weights)
    /// - `tokenizer.json`
    /// - `label_mapping.json` — `{"0": "fix", "1": "feat", ...}`
    pub fn load(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let label_path = model_dir.join("label_mapping.json");

        // Load ONNX session
        let session = Session::builder()
            .context("Failed to create ONNX session builder")?
            .commit_from_file(&model_path)
            .with_context(|| format!("Failed to load ONNX model from {:?}", model_path))?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer from {:?}: {}", tokenizer_path, e))?;

        // Load label mapping
        let label_json = std::fs::read_to_string(&label_path)
            .with_context(|| format!("Failed to read label mapping from {:?}", label_path))?;
        let raw_map: HashMap<String, String> = serde_json::from_str(&label_json)
            .with_context(|| "Failed to parse label_mapping.json")?;
        let label_map: HashMap<usize, String> = raw_map
            .into_iter()
            .map(|(k, v)| {
                let idx: usize = k.parse().expect("label_mapping keys must be numeric");
                (idx, v)
            })
            .collect();

        Ok(Self {
            session,
            tokenizer,
            label_map,
            threshold: DEFAULT_THRESHOLD,
        })
    }

    /// Classify a commit message. Returns `Some((label, confidence))` if the
    /// top prediction exceeds the confidence threshold, `None` otherwise.
    pub fn classify(&mut self, message: &str) -> Option<(String, f32)> {
        let encoding = self.tokenizer.encode(message, true).ok()?;

        let ids = encoding.get_ids();
        let attention = encoding.get_attention_mask();

        // Pad or truncate to MAX_LENGTH
        let input_ids = pad_or_truncate(ids, MAX_LENGTH);
        let attention_mask = pad_or_truncate(attention, MAX_LENGTH);

        // Convert to i64 tensors with shape [1, MAX_LENGTH]
        let input_ids_i64: Vec<i64> = input_ids.iter().map(|&x| x as i64).collect();
        let attention_mask_i64: Vec<i64> = attention_mask.iter().map(|&x| x as i64).collect();

        let input_ids_ref = TensorRef::from_array_view(
            ([1usize, MAX_LENGTH], &*input_ids_i64),
        ).ok()?;
        let attention_mask_ref = TensorRef::from_array_view(
            ([1usize, MAX_LENGTH], &*attention_mask_i64),
        ).ok()?;

        let outputs = self.session.run(
            ort::inputs![input_ids_ref, attention_mask_ref],
        ).ok()?;

        // Extract logits from first output — returns (&Shape, &[f32])
        let (_shape, logits_data) = outputs[0].try_extract_tensor::<f32>().ok()?;
        let logits: Vec<f32> = logits_data.to_vec();

        // Apply softmax
        let probs = softmax(&logits);

        // Find argmax
        let (max_idx, max_prob) = probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())?;

        if *max_prob < self.threshold {
            return None;
        }

        let label = self.label_map.get(&max_idx)?;
        Some((label.clone(), *max_prob))
    }
}

/// Pad a slice to `len` with zeros, or truncate if longer.
fn pad_or_truncate(data: &[u32], len: usize) -> Vec<u32> {
    let mut result = Vec::with_capacity(len);
    for &val in data.iter().take(len) {
        result.push(val);
    }
    result.resize(len, 0);
    result
}

/// Compute softmax over a slice of logits.
pub fn softmax(logits: &[f32]) -> Vec<f32> {
    if logits.is_empty() {
        return Vec::new();
    }
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

/// Load a label mapping from a JSON file. Exposed for testing.
pub fn load_label_mapping(path: &Path) -> Result<HashMap<usize, String>> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    let raw: HashMap<String, String> = serde_json::from_str(&json)?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| (k.parse::<usize>().expect("numeric key"), v))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn softmax_basic() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax(&logits);
        assert_eq!(probs.len(), 3);
        // Sum should be ~1.0
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5, "softmax sum = {}", sum);
        // Largest logit should have largest probability
        assert!(probs[2] > probs[1]);
        assert!(probs[1] > probs[0]);
    }

    #[test]
    fn softmax_equal_logits() {
        let logits = vec![1.0, 1.0, 1.0];
        let probs = softmax(&logits);
        for &p in &probs {
            assert!((p - 1.0 / 3.0).abs() < 1e-5);
        }
    }

    #[test]
    fn softmax_empty() {
        let probs = softmax(&[]);
        assert!(probs.is_empty());
    }

    #[test]
    fn softmax_single() {
        let probs = softmax(&[5.0]);
        assert_eq!(probs.len(), 1);
        assert!((probs[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn softmax_large_values_no_overflow() {
        // Large logits should not cause overflow thanks to max subtraction
        let logits = vec![1000.0, 1001.0, 1002.0];
        let probs = softmax(&logits);
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!(probs[2] > probs[1]);
    }

    #[test]
    fn pad_or_truncate_shorter() {
        let data = vec![1, 2, 3];
        let result = pad_or_truncate(&data, 5);
        assert_eq!(result, vec![1, 2, 3, 0, 0]);
    }

    #[test]
    fn pad_or_truncate_exact() {
        let data = vec![1, 2, 3];
        let result = pad_or_truncate(&data, 3);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn pad_or_truncate_longer() {
        let data = vec![1, 2, 3, 4, 5];
        let result = pad_or_truncate(&data, 3);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn label_mapping_loads() {
        // Write a temp label_mapping.json and verify it loads
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("label_mapping.json");
        std::fs::write(&path, r#"{"0": "build", "1": "feat", "2": "fix"}"#).unwrap();

        let map = load_label_mapping(&path).unwrap();
        assert_eq!(map.len(), 3);
        assert_eq!(map[&0], "build");
        assert_eq!(map[&1], "feat");
        assert_eq!(map[&2], "fix");
    }
}
