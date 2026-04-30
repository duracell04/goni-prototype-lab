use async_trait::async_trait;
use goni_types::{ContextSelection, ModelTier};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct RoutingDecision {
    pub chosen_tier: ModelTier,
    pub reason: String,
    pub selected_route: String,
    pub local_rationale: String,
    pub council_rationale: Option<String>,
    pub classification: RoutingClassification,
    pub models_considered: Vec<String>,
    pub models_used: Vec<String>,
    pub redaction_required: bool,
    pub privacy_class_sent: String,
    pub cost_estimate: String,
    pub latency_estimate: String,
    pub quality_confidence: String,
    pub policy_decision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingClassification {
    pub task_difficulty: String,
    pub knowledge_locality: String,
    pub sensitivity: String,
    pub compute_budget: String,
    pub risk: String,
}

/// Optional escalation policy (e.g., for SPRT-style mid-generation escalation).
#[derive(Clone, Debug)]
pub enum EscalationPolicy {
    None,
    SprtThreshold(f32),
}

/// Router decisions should align with the escalation triggers in `docs/llm-council.md`
/// and the reference defaults in `config/council.yaml` (explicit user flag, high
/// difficulty/safety-critical classification, long-context needs).
#[async_trait]
pub trait Router: Send + Sync {
    async fn decide(
        &self,
        prompt: &str,
        context: &ContextSelection,
    ) -> (RoutingDecision, EscalationPolicy);
}

/// Dummy implementation: always use LocalSmall.
pub struct NullRouter;

#[async_trait]
impl Router for NullRouter {
    async fn decide(
        &self,
        _prompt: &str,
        _context: &ContextSelection,
    ) -> (RoutingDecision, EscalationPolicy) {
        (
            RoutingDecision {
                chosen_tier: ModelTier::LocalSmall,
                reason: "NullRouter".into(),
                selected_route: "local_small".into(),
                local_rationale: "null router defaulted to local small".into(),
                council_rationale: None,
                classification: RoutingClassification {
                    task_difficulty: "routine".into(),
                    knowledge_locality: "answerable_from_memory".into(),
                    sensitivity: "public".into(),
                    compute_budget: "can_run_locally_now".into(),
                    risk: "draft_only".into(),
                },
                models_considered: vec!["local:small".into()],
                models_used: vec!["local:small".into()],
                redaction_required: false,
                privacy_class_sent: "none".into(),
                cost_estimate: "low".into(),
                latency_estimate: "low".into(),
                quality_confidence: "unknown".into(),
                policy_decision: "allowed".into(),
            },
            EscalationPolicy::None,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ConfigRouter {
    cfg: RouterConfig,
}

#[derive(Clone, Debug, Deserialize)]
struct RouterConfig {
    models: HashMap<String, ModelConfig>,
    routing: Option<RoutingConfig>,
}

#[derive(Clone, Debug, Deserialize)]
struct ModelConfig {
    id: String,
    max_tokens: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
struct RoutingConfig {
    local_first: Option<LocalFirstConfig>,
    frugal_sovereign: Option<FrugalSovereignConfig>,
}

#[derive(Clone, Debug, Deserialize)]
struct LocalFirstConfig {
    target_local_token_ratio: Option<f32>,
}

#[derive(Clone, Debug, Deserialize)]
struct FrugalSovereignConfig {
    enabled: Option<bool>,
    block_private_raw_cloud: Option<bool>,
    remote_allowed_privacy_classes: Option<Vec<String>>,
}

impl ConfigRouter {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let cfg: RouterConfig = serde_yaml::from_str(&text)?;
        Ok(Self { cfg })
    }

    fn model_id(&self, key: &str, fallback: &str) -> String {
        self.cfg
            .models
            .get(key)
            .map(|m| m.id.clone())
            .unwrap_or_else(|| fallback.to_string())
    }

    fn interactive_token_budget(&self) -> usize {
        self.cfg
            .models
            .get("interactive")
            .and_then(|m| m.max_tokens)
            .unwrap_or(256) as usize
    }

    fn quality_token_budget(&self) -> usize {
        self.cfg
            .models
            .get("quality")
            .and_then(|m| m.max_tokens)
            .unwrap_or(1200) as usize
    }

    fn frugal_enabled(&self) -> bool {
        self.cfg
            .routing
            .as_ref()
            .and_then(|r| r.frugal_sovereign.as_ref())
            .and_then(|f| f.enabled)
            .unwrap_or(true)
    }

    fn block_private_raw_cloud(&self) -> bool {
        self.cfg
            .routing
            .as_ref()
            .and_then(|r| r.frugal_sovereign.as_ref())
            .and_then(|f| f.block_private_raw_cloud)
            .unwrap_or(true)
    }

    fn remote_allows_privacy_class(&self, privacy_class: &str) -> bool {
        let allowed = self
            .cfg
            .routing
            .as_ref()
            .and_then(|r| r.frugal_sovereign.as_ref())
            .and_then(|f| f.remote_allowed_privacy_classes.as_ref());
        match allowed {
            Some(classes) => classes.iter().any(|c| c == privacy_class),
            None => privacy_class == "public",
        }
    }

    fn classify(&self, prompt: &str, context: &ContextSelection) -> RoutingClassification {
        let text = prompt.to_lowercase();
        let word_count = prompt.split_whitespace().count();
        let task_difficulty = if contains_any(
            &text,
            &["audit-grade", "audit grade", "legal", "financial", "medical"],
        ) {
            "audit_grade"
        } else if contains_any(&text, &["strategy", "research", "architecture", "complex"]) {
            "expert"
        } else if word_count > self.quality_token_budget() {
            "complex"
        } else {
            "routine"
        };

        let knowledge_locality = if contains_any(
            &text,
            &[
                "latest",
                "current",
                "today",
                "recent",
                "web search",
                "search the web",
                "new paper",
                "news",
            ],
        ) {
            "needs_current_web"
        } else if !context.indices.is_empty() {
            "answerable_from_local_documents"
        } else if contains_any(&text, &["repo", "codebase", "file", "tool"]) {
            "needs_local_tools"
        } else {
            "answerable_from_memory"
        };

        let sensitivity = if contains_any(
            &text,
            &[
                "password",
                "secret",
                "api key",
                "token",
                "ssn",
                "identity",
                "medical",
            ],
        ) {
            "legal_financial_identity_sensitive"
        } else if contains_any(&text, &["private", "personal", "confidential", "legal", "financial"]) {
            "private"
        } else {
            "public"
        };

        let compute_budget = if word_count > self.quality_token_budget() {
            "requires_cloud_for_time_reasons"
        } else if word_count > self.interactive_token_budget() || context.total_tokens > self.interactive_token_budget() {
            "can_run_locally_soon"
        } else {
            "can_run_locally_now"
        };

        let risk = if contains_any(&text, &["send", "publish", "delete", "pay", "transfer"]) {
            "irreversible_action"
        } else if contains_any(&text, &["legal", "financial", "medical", "tax"]) {
            "legal_financial_advice"
        } else {
            "draft_only"
        };

        RoutingClassification {
            task_difficulty: task_difficulty.into(),
            knowledge_locality: knowledge_locality.into(),
            sensitivity: sensitivity.into(),
            compute_budget: compute_budget.into(),
            risk: risk.into(),
        }
    }

    fn decide_frugal(&self, prompt: &str, context: &ContextSelection) -> RoutingDecision {
        let classification = self.classify(prompt, context);
        let local_small = format!("local:{}", self.model_id("interactive", "local_small"));
        let local_large = format!("local:{}", self.model_id("quality", "local_medium"));
        let remote = "cloud:council".to_string();
        let mut models_considered = vec![local_small.clone(), local_large.clone()];

        let needs_remote = classification.knowledge_locality == "needs_current_web"
            || classification.task_difficulty == "audit_grade"
            || classification.compute_budget == "requires_cloud_for_time_reasons";
        let privacy_allows_remote = self.remote_allows_privacy_class(&classification.sensitivity);
        let raw_cloud_blocked = self.block_private_raw_cloud()
            && needs_remote
            && !privacy_allows_remote;

        if needs_remote {
            models_considered.push(remote.clone());
        }

        if needs_remote && !raw_cloud_blocked {
            return RoutingDecision {
                chosen_tier: ModelTier::RemoteHeavy,
                reason: "frugal_sovereign: local route insufficient for current/external/high-compute task".into(),
                selected_route: "local_first_then_council".into(),
                local_rationale: "local route classified as insufficient for this request class".into(),
                council_rationale: Some("current/external evidence or audit-grade disagreement checking is required".into()),
                classification,
                models_considered,
                models_used: vec![remote],
                redaction_required: true,
                privacy_class_sent: "public_only".into(),
                cost_estimate: "medium".into(),
                latency_estimate: "high".into(),
                quality_confidence: "medium".into(),
                policy_decision: "allowed".into(),
            };
        }

        if raw_cloud_blocked || classification.compute_budget == "can_run_locally_soon" {
            return RoutingDecision {
                chosen_tier: ModelTier::LocalLarge,
                reason: if raw_cloud_blocked {
                    "frugal_sovereign: remote need detected but raw private cloud escalation is blocked".into()
                } else {
                    "frugal_sovereign: prompt exceeds local-small budget".into()
                },
                selected_route: "local_large".into(),
                local_rationale: "use larger local model before any external escalation".into(),
                council_rationale: raw_cloud_blocked.then(|| {
                    "Council requires redaction or explicit approval before private context can leave the node".into()
                }),
                classification,
                models_considered,
                models_used: vec![local_large],
                redaction_required: raw_cloud_blocked,
                privacy_class_sent: "none".into(),
                cost_estimate: "low".into(),
                latency_estimate: "medium".into(),
                quality_confidence: if raw_cloud_blocked { "low" } else { "medium" }.into(),
                policy_decision: if raw_cloud_blocked { "blocked" } else { "allowed" }.into(),
            };
        }

        RoutingDecision {
            chosen_tier: ModelTier::LocalSmall,
            reason: "frugal_sovereign: local-small is sufficient and private".into(),
            selected_route: "local_small".into(),
            local_rationale: "routine request fits local-small budget".into(),
            council_rationale: None,
            classification,
            models_considered,
            models_used: vec![local_small],
            redaction_required: false,
            privacy_class_sent: "none".into(),
            cost_estimate: "low".into(),
            latency_estimate: "low".into(),
            quality_confidence: "medium".into(),
            policy_decision: "allowed".into(),
        }
    }

    fn choose_tier(&self, prompt: &str) -> ModelTier {
        let default = ModelTier::LocalSmall;
        let max_tokens = self.interactive_token_budget();
        if prompt.split_whitespace().count() > max_tokens {
            ModelTier::LocalLarge
        } else {
            default
        }
    }
}

#[async_trait]
impl Router for ConfigRouter {
    async fn decide(
        &self,
        prompt: &str,
        _context: &ContextSelection,
    ) -> (RoutingDecision, EscalationPolicy) {
        if self.frugal_enabled() {
            return (self.decide_frugal(prompt, _context), EscalationPolicy::None);
        }

        let tier = self.choose_tier(prompt);
        let selected_route = match tier {
            ModelTier::LocalSmall => "local_small",
            ModelTier::LocalLarge => "local_large",
            ModelTier::RemoteHeavy => "remote_heavy",
        };
        (
            RoutingDecision {
                chosen_tier: tier,
                reason: "config".into(),
                selected_route: selected_route.into(),
                local_rationale: "legacy local-first config route".into(),
                council_rationale: None,
                classification: self.classify(prompt, _context),
                models_considered: vec![
                    format!("local:{}", self.model_id("interactive", "local_small")),
                    format!("local:{}", self.model_id("quality", "local_medium")),
                ],
                models_used: vec![selected_route.into()],
                redaction_required: false,
                privacy_class_sent: "none".into(),
                cost_estimate: "low".into(),
                latency_estimate: "low".into(),
                quality_confidence: "unknown".into(),
                policy_decision: "allowed".into(),
            },
            EscalationPolicy::None,
        )
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> ConfigRouter {
        ConfigRouter::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../config/router.yaml"))
            .unwrap()
    }

    fn empty_context() -> ContextSelection {
        ContextSelection {
            indices: Vec::new(),
            total_tokens: 0,
        }
    }

    #[tokio::test]
    async fn routine_prompt_stays_local_small() {
        let (decision, _) = router().decide("Draft a friendly follow-up reminder.", &empty_context()).await;
        assert_eq!(decision.chosen_tier, ModelTier::LocalSmall);
        assert_eq!(decision.selected_route, "local_small");
        assert_eq!(decision.policy_decision, "allowed");
    }

    #[tokio::test]
    async fn long_prompt_routes_local_large() {
        let prompt = std::iter::repeat("token").take(300).collect::<Vec<_>>().join(" ");
        let (decision, _) = router().decide(&prompt, &empty_context()).await;
        assert_eq!(decision.chosen_tier, ModelTier::LocalLarge);
        assert_eq!(decision.selected_route, "local_large");
    }

    #[tokio::test]
    async fn public_current_web_prompt_routes_remote_heavy() {
        let (decision, _) = router()
            .decide("Research the latest public paper on LLM routing.", &empty_context())
            .await;
        assert_eq!(decision.chosen_tier, ModelTier::RemoteHeavy);
        assert_eq!(decision.selected_route, "local_first_then_council");
        assert_eq!(decision.privacy_class_sent, "public_only");
    }

    #[tokio::test]
    async fn private_current_web_prompt_blocks_raw_cloud() {
        let (decision, _) = router()
            .decide("Use my private notes and latest web sources for this legal strategy.", &empty_context())
            .await;
        assert_eq!(decision.chosen_tier, ModelTier::LocalLarge);
        assert_eq!(decision.policy_decision, "blocked");
        assert!(decision.redaction_required);
        assert_eq!(decision.privacy_class_sent, "none");
    }
}
