use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ReceiptError {
    #[error("io error: {0}")]
    Io(String),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: Uuid,
    pub timestamp: String,
    pub action_type: String,
    pub policy_decision: String,
    pub capability_id: Option<Uuid>,
    pub input_hash: String,
    pub output_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_route: Option<LlmRouteReceipt>,
    pub prev_hash: Option<String>,
    pub chain_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmRouteReceipt {
    pub selected_route: String,
    pub local_rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub council_rationale: Option<String>,
    pub task_difficulty: String,
    pub knowledge_locality: String,
    pub sensitivity: String,
    pub compute_budget: String,
    pub risk: String,
    pub models_considered: Vec<String>,
    pub models_used: Vec<String>,
    pub redaction_required: bool,
    pub privacy_class_sent: String,
    pub cost_estimate: String,
    pub latency_estimate: String,
    pub quality_confidence: String,
    pub policy_decision: String,
}

pub struct ReceiptLog {
    path: PathBuf,
    last_hash: Mutex<Option<String>>,
}

impl ReceiptLog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ReceiptError> {
        let path = path.as_ref().to_path_buf();
        let last_hash = read_last_hash(&path)?;
        Ok(Self {
            path,
            last_hash: Mutex::new(last_hash),
        })
    }

    pub fn append(&self, mut receipt: Receipt) -> Result<(), ReceiptError> {
        let mut last = self.last_hash.lock().map_err(|_| ReceiptError::Io("lock".into()))?;
        receipt.prev_hash = last.clone();
        receipt.chain_hash = hash_receipt(&receipt);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| ReceiptError::Io(e.to_string()))?;
        let line = serde_json::to_string(&receipt).map_err(|e| ReceiptError::Parse(e.to_string()))?;
        writeln!(file, "{line}").map_err(|e| ReceiptError::Io(e.to_string()))?;
        *last = Some(receipt.chain_hash.clone());
        Ok(())
    }
}

pub fn verify_log(path: impl AsRef<Path>) -> Result<(), ReceiptError> {
    let file = File::open(path.as_ref()).map_err(|e| ReceiptError::Io(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut prev: Option<String> = None;
    for line in reader.lines() {
        let line = line.map_err(|e| ReceiptError::Io(e.to_string()))?;
        let receipt: Receipt = serde_json::from_str(&line).map_err(|e| ReceiptError::Parse(e.to_string()))?;
        if receipt.prev_hash != prev {
            return Err(ReceiptError::Parse("hash chain mismatch".into()));
        }
        let expected = hash_receipt(&receipt);
        if receipt.chain_hash != expected {
            return Err(ReceiptError::Parse("chain hash invalid".into()));
        }
        prev = Some(receipt.chain_hash);
    }
    Ok(())
}

fn hash_receipt(receipt: &Receipt) -> String {
    let mut h = Sha256::new();
    h.update(receipt.receipt_id.to_string());
    h.update(&receipt.timestamp);
    h.update(&receipt.action_type);
    h.update(&receipt.policy_decision);
    if let Some(id) = receipt.capability_id {
        h.update(id.to_string());
    }
    h.update(&receipt.input_hash);
    h.update(&receipt.output_hash);
    if let Some(route) = &receipt.llm_route {
        if let Ok(route_json) = serde_json::to_string(route) {
            h.update(route_json);
        }
    }
    if let Some(prev) = &receipt.prev_hash {
        h.update(prev);
    }
    format!("{:x}", h.finalize())
}

fn read_last_hash(path: &Path) -> Result<Option<String>, ReceiptError> {
    if !path.exists() {
        return Ok(None);
    }
    let file = File::open(path).map_err(|e| ReceiptError::Io(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut last: Option<String> = None;
    for line in reader.lines() {
        let line = line.map_err(|e| ReceiptError::Io(e.to_string()))?;
        let receipt: Receipt = serde_json::from_str(&line).map_err(|e| ReceiptError::Parse(e.to_string()))?;
        last = Some(receipt.chain_hash);
    }
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn receipt_chain_verifies() {
        let path = "target/test_receipts.jsonl";
        let _ = fs::remove_file(path);
        let log = ReceiptLog::open(path).unwrap();
        let r1 = Receipt {
            receipt_id: Uuid::new_v4(),
            timestamp: "t1".into(),
            action_type: "demo".into(),
            policy_decision: "allow".into(),
            capability_id: None,
            input_hash: "a".into(),
            output_hash: "b".into(),
            llm_route: None,
            prev_hash: None,
            chain_hash: "".into(),
        };
        log.append(r1).unwrap();
        verify_log(path).unwrap();
    }

    #[test]
    fn route_metadata_round_trips_and_affects_hash() {
        let route = LlmRouteReceipt {
            selected_route: "local_small".into(),
            local_rationale: "routine local route".into(),
            council_rationale: None,
            task_difficulty: "routine".into(),
            knowledge_locality: "answerable_from_memory".into(),
            sensitivity: "public".into(),
            compute_budget: "can_run_locally_now".into(),
            risk: "draft_only".into(),
            models_considered: vec!["local:small".into()],
            models_used: vec!["local:small".into()],
            redaction_required: false,
            privacy_class_sent: "none".into(),
            cost_estimate: "low".into(),
            latency_estimate: "low".into(),
            quality_confidence: "medium".into(),
            policy_decision: "allowed".into(),
        };
        let receipt = Receipt {
            receipt_id: Uuid::new_v4(),
            timestamp: "t1".into(),
            action_type: "model.route".into(),
            policy_decision: "allow".into(),
            capability_id: None,
            input_hash: "a".into(),
            output_hash: "b".into(),
            llm_route: Some(route.clone()),
            prev_hash: None,
            chain_hash: "".into(),
        };

        let encoded = serde_json::to_string(&receipt).unwrap();
        let decoded: Receipt = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.llm_route, Some(route));

        let without_route = Receipt {
            llm_route: None,
            ..receipt.clone()
        };
        assert_ne!(hash_receipt(&receipt), hash_receipt(&without_route));
    }
}
