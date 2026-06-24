use soroban_sdk::{Env, String, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

pub fn validate_payload(env: &Env, fields: &[(&str, Option<String>)]) -> ValidationReport {
    let mut issues = Vec::new(env);

    for (field_name, value) in fields.iter() {
        if let Some(value) = value {
            if value.len() == 0 {
                issues.push_back(ValidationIssue {
                    field: String::from_str(env, field_name),
                    message: String::from_str(env, "must not be empty"),
                });
            }
            if field_name == &"amount" && value.len() > 0 {
                let parsed: i128 = value.to_string().parse().unwrap_or(0);
                if parsed <= 0 {
                    issues.push_back(ValidationIssue {
                        field: String::from_str(env, field_name),
                        message: String::from_str(env, "must be positive"),
                    });
                }
            }
        } else {
            issues.push_back(ValidationIssue {
                field: String::from_str(env, field_name),
                message: String::from_str(env, "is required"),
            });
        }
    }

    ValidationReport { is_valid: issues.is_empty(), issues }
}

pub fn validate_string_length(env: &Env, value: &String, field_name: &str, max_len: u32) -> Result<(), ValidationIssue> {
    if value.len() > max_len {
        Err(ValidationIssue {
            field: String::from_str(env, field_name),
            message: String::from_str(env, "exceeds the maximum allowed length"),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_required_fields_and_positive_amounts() {
        let env = Env::default();
        let merchant_id = String::from_str(&env, "merchant-1");
        let amount = String::from_str(&env, "100");
        let report = validate_payload(&env, &[("merchant_id", Some(merchant_id.clone())), ("amount", Some(amount.clone())), ("reason", None)]);

        assert!(!report.is_valid);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues.get(0).unwrap().field, String::from_str(&env, "reason"));
    }

    #[test]
    fn rejects_non_positive_amounts() {
        let env = Env::default();
        let report = validate_payload(&env, &[("amount", Some(String::from_str(&env, "0")))]);

        assert!(!report.is_valid);
        assert_eq!(report.issues.len(), 1);
    }
}
