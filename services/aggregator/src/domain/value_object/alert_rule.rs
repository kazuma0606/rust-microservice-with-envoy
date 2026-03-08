#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertRuleName {
    DenyThresholdExceeded,
    ConsecutiveAuthFailure,
}

impl AlertRuleName {
    pub fn as_str(&self) -> &str {
        match self {
            AlertRuleName::DenyThresholdExceeded => "DenyThresholdExceeded",
            AlertRuleName::ConsecutiveAuthFailure => "ConsecutiveAuthFailure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    High,
    Medium,
    Low,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            AlertSeverity::High => "HIGH",
            AlertSeverity::Medium => "MEDIUM",
            AlertSeverity::Low => "LOW",
        }
    }

    pub fn to_proto_i32(&self) -> i32 {
        match self {
            AlertSeverity::High => 1,
            AlertSeverity::Medium => 2,
            AlertSeverity::Low => 3,
        }
    }
}
