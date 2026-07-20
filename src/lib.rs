pub mod analysis;
pub mod cli;
pub mod patterns;

pub use analysis::{MatchResult, Output, Stats, analyze};
pub use patterns::{Pattern, RawMatch, build_patterns};

#[cfg(test)]
mod tests;
