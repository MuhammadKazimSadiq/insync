use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Operator {
    Equals,
    NotEquals,
    Exists,
    NotExists,
    Contains,
    In,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulePredicate {
    pub property: String,
    pub operator: Operator,
    #[serde(default)]
    pub value: Option<JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleGroup {
    All(Vec<RuleGroup>),
    Any(Vec<RuleGroup>),
    Predicate(RulePredicate),
}

/// Convert YAML value to JSON value while preserving exact types
pub fn yaml_to_json(val: &YamlValue) -> JsonValue {
    match val {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(b) => JsonValue::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::from(i)
            } else if let Some(u) = n.as_u64() {
                JsonValue::from(u)
            } else if let Some(f) = n.as_f64() {
                JsonValue::from(f)
            } else {
                JsonValue::Null
            }
        }
        YamlValue::String(s) => JsonValue::String(s.clone()),
        YamlValue::Sequence(seq) => JsonValue::Array(seq.iter().map(yaml_to_json).collect()),
        YamlValue::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let YamlValue::String(key_str) = k {
                    obj.insert(key_str.clone(), yaml_to_json(v));
                }
            }
            JsonValue::Object(obj)
        }
        YamlValue::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

pub fn evaluate_predicate(predicate: &RulePredicate, metadata: &YamlValue) -> bool {
    let prop_yaml = match metadata {
        YamlValue::Mapping(map) => map.get(&YamlValue::String(predicate.property.clone())),
        _ => None,
    };

    let prop_json = prop_yaml.map(yaml_to_json);

    match predicate.operator {
        Operator::Exists => prop_json.is_some() && !prop_json.as_ref().unwrap().is_null(),
        Operator::NotExists => prop_json.is_none() || prop_json.as_ref().unwrap().is_null(),
        Operator::Equals => {
            match (&prop_json, &predicate.value) {
                (Some(actual), Some(expected)) => actual == expected,
                _ => false,
            }
        }
        Operator::NotEquals => {
            match (&prop_json, &predicate.value) {
                (Some(actual), Some(expected)) => actual != expected,
                (None, Some(_)) => true,
                _ => false,
            }
        }
        Operator::Contains => {
            match (&prop_json, &predicate.value) {
                (Some(JsonValue::Array(arr)), Some(expected)) => arr.contains(expected),
                (Some(JsonValue::String(s)), Some(JsonValue::String(sub))) => s.contains(sub),
                _ => false,
            }
        }
        Operator::In => {
            match (&prop_json, &predicate.value) {
                (Some(actual), Some(JsonValue::Array(arr))) => arr.contains(actual),
                _ => false,
            }
        }
    }
}

pub fn evaluate_rule_group(group: &RuleGroup, metadata: &YamlValue) -> bool {
    match group {
        RuleGroup::Predicate(pred) => evaluate_predicate(pred, metadata),
        RuleGroup::All(groups) => groups.iter().all(|g| evaluate_rule_group(g, metadata)),
        RuleGroup::Any(groups) => groups.iter().any(|g| evaluate_rule_group(g, metadata)),
    }
}
