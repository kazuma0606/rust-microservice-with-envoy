use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

impl Decision {
    pub fn from_proto(value: i32) -> Result<Self, DomainError> {
        match value {
            1 => Ok(Decision::Allow),
            2 => Ok(Decision::Deny),
            _ => Err(DomainError::Validation(format!(
                "Invalid decision value: {} (expected 1=ALLOW or 2=DENY)",
                value
            ))),
        }
    }

    pub fn to_db_string(&self) -> &str {
        match self {
            Decision::Allow => "ALLOW",
            Decision::Deny => "DENY",
        }
    }
}
