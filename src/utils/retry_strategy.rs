use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    Best,
    CombinedBest,
    Median,
    Worst,
    CombinedWorst,
}

impl RetryStrategy {
    pub fn as_str(&self) -> &'static str {
        match *self {
            RetryStrategy::Best => "best",
            RetryStrategy::CombinedBest => "combined_best",
            RetryStrategy::Median => "median",
            RetryStrategy::Worst => "worst",
            RetryStrategy::CombinedWorst => "combined_worst",
        }
    }

    pub fn from_str(retry_strategy: &str) -> Option<RetryStrategy> {
        match retry_strategy {
            "best" => Some(RetryStrategy::Best),
            "combined_best" => Some(RetryStrategy::CombinedBest),
            "median" => Some(RetryStrategy::Median),
            "worst" => Some(RetryStrategy::Worst),
            "combined_worst" => Some(RetryStrategy::CombinedWorst),
            _ => None,
        }
    }
}

impl Display for RetryStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
