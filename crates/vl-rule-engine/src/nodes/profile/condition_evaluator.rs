use serde::{Deserialize, Serialize};

/// Evaluates ThingsBoard alarm rule conditions against a telemetry/attribute message.
/// Java: AlarmConditionSpec, KeyFilterPredicate, SimpleAlarmConditionSpec

/// Top-level condition — wraps a predicate applied to a specific key
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmConditionSpec {
    #[serde(rename = "type", default = "default_simple")]
    pub condition_type: ConditionType,
    /// The key to evaluate (TIME_SERIES or ATTRIBUTE)
    pub key: Option<AlarmConditionKey>,
    /// The predicate to apply to the key value
    pub predicate: Option<KeyFilterPredicate>,
    /// Duration condition: how long the condition must hold (ms)
    #[serde(rename = "durationTimeUnit")]
    pub duration_time_unit: Option<String>,
    #[serde(rename = "durationValue")]
    pub duration_value: Option<u64>,
}

fn default_simple() -> ConditionType { ConditionType::Simple }

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConditionType {
    Simple,
    Duration,
    Repeating,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmConditionKey {
    #[serde(rename = "type")]
    pub key_type: String, // "TIME_SERIES" | "ATTRIBUTE"
    pub key: String,
}

/// Predicate that tests a single value
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyFilterPredicate {
    Numeric {
        operation: NumericOperation,
        value: PredicateValue,
    },
    String {
        operation: StringOperation,
        value: PredicateValue,
        #[serde(rename = "ignoreCase", default)]
        ignore_case: bool,
    },
    Boolean {
        operation: BooleanOperation,
        value: PredicateValue,
    },
    Complex {
        operation: ComplexOperation,
        predicates: Vec<KeyFilterPredicate>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NumericOperation {
    Equal, NotEqual, Greater, Less, GreaterOrEqual, LessOrEqual,
    #[serde(rename = "IN_RANGE")]
    InRange,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StringOperation {
    Equal, NotEqual, StartsWith, EndsWith, Contains, NotContains,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BooleanOperation {
    Equal, NotEqual,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplexOperation {
    And, Or,
}

/// A predicate value — may be constant or reference another key
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredicateValue {
    #[serde(rename = "defaultValue")]
    pub default_value: serde_json::Value,
    /// Optional: key whose value to use instead of defaultValue
    #[serde(rename = "dynamicValue")]
    pub dynamic_value: Option<DynamicValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicValue {
    #[serde(rename = "sourceType")]
    pub source_type: String, // "CURRENT_DEVICE", "TENANT", "CUSTOMER"
    #[serde(rename = "sourceAttribute")]
    pub source_attribute: String,
    #[serde(rename = "inherit", default)]
    pub inherit: bool,
}

impl PredicateValue {
    pub fn as_f64(&self) -> Option<f64> { self.default_value.as_f64() }
    pub fn as_str(&self) -> Option<&str> { self.default_value.as_str() }
    pub fn as_bool(&self) -> Option<bool> { self.default_value.as_bool() }
}

/// Evaluate a predicate against a value extracted from a message
pub fn evaluate_predicate(predicate: &KeyFilterPredicate, value: &serde_json::Value) -> bool {
    match predicate {
        KeyFilterPredicate::Numeric { operation, value: pred_val } => {
            let v = match value.as_f64() {
                Some(n) => n,
                None => return false,
            };
            let threshold = match pred_val.as_f64() {
                Some(n) => n,
                None => return false,
            };
            match operation {
                NumericOperation::Equal          => (v - threshold).abs() < f64::EPSILON,
                NumericOperation::NotEqual       => (v - threshold).abs() >= f64::EPSILON,
                NumericOperation::Greater        => v > threshold,
                NumericOperation::Less           => v < threshold,
                NumericOperation::GreaterOrEqual => v >= threshold,
                NumericOperation::LessOrEqual    => v <= threshold,
                NumericOperation::InRange        => {
                    // For InRange, defaultValue is the lower bound; upper bound TBD
                    v >= threshold
                }
            }
        }
        KeyFilterPredicate::String { operation, value: pred_val, ignore_case } => {
            let v = match value.as_str() {
                Some(s) => s.to_string(),
                None => value.to_string(),
            };
            let threshold = pred_val.as_str().unwrap_or("").to_string();
            let (v_cmp, t_cmp) = if *ignore_case {
                (v.to_lowercase(), threshold.to_lowercase())
            } else {
                (v.clone(), threshold.clone())
            };
            match operation {
                StringOperation::Equal       => v_cmp == t_cmp,
                StringOperation::NotEqual    => v_cmp != t_cmp,
                StringOperation::StartsWith  => v_cmp.starts_with(&t_cmp),
                StringOperation::EndsWith    => v_cmp.ends_with(&t_cmp),
                StringOperation::Contains    => v_cmp.contains(&t_cmp),
                StringOperation::NotContains => !v_cmp.contains(&t_cmp),
            }
        }
        KeyFilterPredicate::Boolean { operation, value: pred_val } => {
            let v = value.as_bool().unwrap_or(false);
            let threshold = pred_val.as_bool().unwrap_or(false);
            match operation {
                BooleanOperation::Equal    => v == threshold,
                BooleanOperation::NotEqual => v != threshold,
            }
        }
        KeyFilterPredicate::Complex { operation, predicates } => {
            match operation {
                ComplexOperation::And => predicates.iter().all(|p| evaluate_predicate(p, value)),
                ComplexOperation::Or  => predicates.iter().any(|p| evaluate_predicate(p, value)),
            }
        }
    }
}

/// Extract the value for a key from the message data JSON
pub fn extract_key_value(data: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    data.get(key).cloned()
}

/// Evaluate a full AlarmConditionSpec against the message data
pub fn evaluate_condition(spec: &AlarmConditionSpec, data: &serde_json::Value) -> bool {
    let key = match &spec.key {
        Some(k) => &k.key,
        None    => return false,
    };
    let predicate = match &spec.predicate {
        Some(p) => p,
        None    => return false,
    };
    let value = match extract_key_value(data, key) {
        Some(v) => v,
        None    => return false,
    };
    evaluate_predicate(predicate, &value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numeric_greater() {
        let pred = KeyFilterPredicate::Numeric {
            operation: NumericOperation::Greater,
            value: PredicateValue { default_value: json!(50.0), dynamic_value: None },
        };
        assert!(evaluate_predicate(&pred, &json!(55.0)));
        assert!(!evaluate_predicate(&pred, &json!(45.0)));
        assert!(!evaluate_predicate(&pred, &json!(50.0)));
    }

    #[test]
    fn numeric_less_or_equal() {
        let pred = KeyFilterPredicate::Numeric {
            operation: NumericOperation::LessOrEqual,
            value: PredicateValue { default_value: json!(40.0), dynamic_value: None },
        };
        assert!(evaluate_predicate(&pred, &json!(40.0)));
        assert!(evaluate_predicate(&pred, &json!(39.9)));
        assert!(!evaluate_predicate(&pred, &json!(40.1)));
    }

    #[test]
    fn string_contains() {
        let pred = KeyFilterPredicate::String {
            operation: StringOperation::Contains,
            value: PredicateValue { default_value: json!("error"), dynamic_value: None },
            ignore_case: true,
        };
        assert!(evaluate_predicate(&pred, &json!("ERROR: sensor offline")));
        assert!(!evaluate_predicate(&pred, &json!("all systems nominal")));
    }

    #[test]
    fn boolean_equal() {
        let pred = KeyFilterPredicate::Boolean {
            operation: BooleanOperation::Equal,
            value: PredicateValue { default_value: json!(true), dynamic_value: None },
        };
        assert!(evaluate_predicate(&pred, &json!(true)));
        assert!(!evaluate_predicate(&pred, &json!(false)));
    }

    #[test]
    fn complex_and() {
        let pred = KeyFilterPredicate::Complex {
            operation: ComplexOperation::And,
            predicates: vec![
                KeyFilterPredicate::Numeric {
                    operation: NumericOperation::Greater,
                    value: PredicateValue { default_value: json!(0.0), dynamic_value: None },
                },
                KeyFilterPredicate::Numeric {
                    operation: NumericOperation::Less,
                    value: PredicateValue { default_value: json!(100.0), dynamic_value: None },
                },
            ],
        };
        assert!(evaluate_predicate(&pred, &json!(50.0)));
        assert!(!evaluate_predicate(&pred, &json!(150.0)));
        assert!(!evaluate_predicate(&pred, &json!(-1.0)));
    }

    #[test]
    fn extract_key_from_data() {
        let data = json!({ "temperature": 25.5, "humidity": 60 });
        assert_eq!(extract_key_value(&data, "temperature"), Some(json!(25.5)));
        assert_eq!(extract_key_value(&data, "missing"), None);
    }
}
