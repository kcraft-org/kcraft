use serde::{Deserialize, Serialize};
use crate::OpSys;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Disallow,
    Defer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rule {
    Os(OsRuleData),
    Implicit(ImplicitRule),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRuleData {
    pub action: RuleAction,
    pub system: Option<String>,
    pub version_regex: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplicitRule {
    pub action: RuleAction,
}

impl Rule {
    pub fn action(&self) -> RuleAction {
        match self {
            Rule::Os(r) => r.action,
            Rule::Implicit(r) => r.action,
        }
    }

    pub fn apply(&self, runtime_os: &OpSys) -> RuleAction {
        match self {
            Rule::Os(r) => {
                if let Some(ref system) = r.system {
                    if system != runtime_os.classifier() {
                        return RuleAction::Defer;
                    }
                }
                r.action
            }
            Rule::Implicit(_) => RuleAction::Allow,
        }
    }
}

pub fn rules_from_json(rules: &[serde_json::Value]) -> Vec<Rule> {
    let mut result = Vec::new();
    for rule_val in rules {
        if let Some(obj) = rule_val.as_object() {
            let action = match obj.get("action").and_then(|v| v.as_str()) {
                Some("allow") => RuleAction::Allow,
                Some("disallow") => RuleAction::Disallow,
                _ => RuleAction::Defer,
            };

            if let Some(os_obj) = obj.get("os").and_then(|v| v.as_object()) {
                let system = os_obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                let version_regex = os_obj.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
                let arch = os_obj.get("arch").and_then(|v| v.as_str()).map(|s| s.to_string());
                result.push(Rule::Os(OsRuleData { action, system, version_regex, arch }));
            } else {
                result.push(Rule::Implicit(ImplicitRule { action }));
            }
        }
    }
    result
}
