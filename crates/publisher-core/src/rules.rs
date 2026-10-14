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
    All { all: Vec<RuleGroup> },
    Any { any: Vec<RuleGroup> },
    List(Vec<RuleGroup>),
    Predicate(RulePredicate),
}

impl RuleGroup {
    pub fn all(rules: Vec<RuleGroup>) -> Self {
        RuleGroup::All { all: rules }
    }

    pub fn any(rules: Vec<RuleGroup>) -> Self {
        RuleGroup::Any { any: rules }
    }

    pub fn predicate(predicate: RulePredicate) -> Self {
        RuleGroup::Predicate(predicate)
    }
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
        RuleGroup::All { all } => all.iter().all(|g| evaluate_rule_group(g, metadata)),
        RuleGroup::Any { any } => any.iter().any(|g| evaluate_rule_group(g, metadata)),
        RuleGroup::List(list) => list.iter().all(|g| evaluate_rule_group(g, metadata)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_rule_group_all() {
        let json_str = r#"{
            "all": [
                {
                    "property": "status",
                    "operator": "equals",
                    "value": "completed"
                },
                {
                    "property": "share",
                    "operator": "equals",
                    "value": true
                }
            ]
        }"#;

        let parsed: RuleGroup = serde_json::from_str(json_str).expect("deserialize all rule group");
        match parsed {
            RuleGroup::All { all } => {
                assert_eq!(all.len(), 2);
            }
            other => panic!("expected RuleGroup::All, got {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_rule_group_any() {
        let json_str = r#"{
            "any": [
                {
                    "property": "visibility",
                    "operator": "equals",
                    "value": "public"
                }
            ]
        }"#;

        let parsed: RuleGroup = serde_json::from_str(json_str).expect("deserialize any rule group");
        match parsed {
            RuleGroup::Any { any } => {
                assert_eq!(any.len(), 1);
            }
            other => panic!("expected RuleGroup::Any, got {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_rule_group_list() {
        let json_str = r#"[
            {
                "property": "status",
                "operator": "equals",
                "value": "completed"
            }
        ]"#;

        let parsed: RuleGroup = serde_json::from_str(json_str).expect("deserialize list rule group");
        match parsed {
            RuleGroup::List(list) => {
                assert_eq!(list.len(), 1);
            }
            other => panic!("expected RuleGroup::List, got {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_rule_group_predicate() {
        let json_str = r#"{
            "property": "status",
            "operator": "equals",
            "value": "completed"
        }"#;

        let parsed: RuleGroup = serde_json::from_str(json_str).expect("deserialize predicate");
        match parsed {
            RuleGroup::Predicate(pred) => {
                assert_eq!(pred.property, "status");
                assert_eq!(pred.operator, Operator::Equals);
                assert_eq!(pred.value, Some(json!("completed")));
            }
            other => panic!("expected RuleGroup::Predicate, got {:?}", other),
        }
    }

    #[test]
    fn test_serialize_and_deserialize_roundtrip() {
        let group = RuleGroup::all(vec![RuleGroup::Predicate(RulePredicate {
            property: "status".to_string(),
            operator: Operator::Equals,
            value: Some(json!("completed")),
        })]);

        let serialized = serde_json::to_string(&group).expect("serialize");
        assert!(serialized.contains(r#""all":[{""#) || serialized.contains(r#""all":["#));

        let deserialized: RuleGroup = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(group, deserialized);
    }
}
