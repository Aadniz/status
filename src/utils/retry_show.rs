use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetryShow {
    Best,
    CombinedBest,
    Median,
    Worst,
    CombinedWorst,
}

impl RetryShow {
    pub fn as_str(&self) -> &'static str {
        match *self {
            RetryShow::Best => "best",
            RetryShow::CombinedBest => "combined_best",
            RetryShow::Median => "median",
            RetryShow::Worst => "worst",
            RetryShow::CombinedWorst => "combined_worst",
        }
    }

    pub fn from_str(retry_show: &str) -> Option<RetryShow> {
        match retry_show {
            "best" => Some(RetryShow::Best),
            "combined_best" => Some(RetryShow::CombinedBest),
            "median" => Some(RetryShow::Median),
            "worst" => Some(RetryShow::Worst),
            "combined_worst" => Some(RetryShow::CombinedWorst),
            _ => None,
        }
    }
}

impl Display for RetryShow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
