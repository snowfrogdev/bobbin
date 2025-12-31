//! Fuzzy string matching adapter for "did you mean?" suggestions.
//!
//! The `Matcher` trait abstracts over string similarity algorithms,
//! allowing the matching implementation to be swapped without changing
//! the diagnostic logic.

use strsim::jaro_winkler;

/// Trait for fuzzy string matching.
///
/// Implementations find the best match for a query string among candidates,
/// used for "did you mean 'X'?" suggestions on undefined variables.
pub trait Matcher {
    /// Find the best match for `query` among `candidates`.
    ///
    /// Returns the best matching candidate and its similarity score (0.0 to 1.0),
    /// or `None` if no candidate meets the minimum threshold.
    fn best_match<'a>(&self, query: &str, candidates: &'a [String]) -> Option<(&'a str, f64)>;

    /// Find all matches above the threshold, sorted by score descending.
    fn find_similar<'a>(&self, query: &str, candidates: &'a [String]) -> Vec<(&'a str, f64)>;
}

/// Jaro-Winkler based matcher using the strsim crate.
///
/// Jaro-Winkler is well-suited for matching variable names because it:
/// - Favors matching prefixes (good for `player_name` vs `player_naem`)
/// - Handles transpositions well (catches common typos)
/// - Works well with descriptive names common in narrative scripts
#[derive(Debug, Clone)]
pub struct JaroWinklerMatcher {
    /// Minimum similarity score (0.0 to 1.0) to consider a match.
    /// Typical values: 0.7 for loose matching, 0.8 for stricter matching.
    pub threshold: f64,
}

impl JaroWinklerMatcher {
    /// Create a new matcher with the given threshold.
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Create a new matcher with a sensible default threshold (0.7).
    pub fn default_threshold() -> Self {
        Self::new(0.7)
    }
}

impl Default for JaroWinklerMatcher {
    fn default() -> Self {
        Self::default_threshold()
    }
}

impl Matcher for JaroWinklerMatcher {
    fn best_match<'a>(&self, query: &str, candidates: &'a [String]) -> Option<(&'a str, f64)> {
        candidates
            .iter()
            .map(|c| (c.as_str(), jaro_winkler(query, c)))
            .filter(|(_, score)| *score >= self.threshold)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn find_similar<'a>(&self, query: &str, candidates: &'a [String]) -> Vec<(&'a str, f64)> {
        let mut matches: Vec<_> = candidates
            .iter()
            .map(|c| (c.as_str(), jaro_winkler(query, c)))
            .filter(|(_, score)| *score >= self.threshold)
            .collect();

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_typo() {
        let matcher = JaroWinklerMatcher::default();
        let candidates = vec!["player_name".to_string(), "gold".to_string()];

        let result = matcher.best_match("player_naem", &candidates);
        assert!(result.is_some());
        let (matched, score) = result.unwrap();
        assert_eq!(matched, "player_name");
        assert!(score > 0.9);
    }

    #[test]
    fn no_match_for_unrelated() {
        let matcher = JaroWinklerMatcher::default();
        let candidates = vec!["player_name".to_string()];

        let result = matcher.best_match("completely_different", &candidates);
        assert!(result.is_none());
    }

    #[test]
    fn finds_multiple_similar() {
        let matcher = JaroWinklerMatcher::new(0.6);
        let candidates = vec![
            "player_name".to_string(),
            "player_health".to_string(),
            "enemy_name".to_string(),
        ];

        let results = matcher.find_similar("player", &candidates);
        assert!(!results.is_empty());
        // Should find player_name and player_health
        assert!(results.iter().any(|(s, _)| *s == "player_name"));
        assert!(results.iter().any(|(s, _)| *s == "player_health"));
    }

    #[test]
    fn threshold_filters_results() {
        let strict_matcher = JaroWinklerMatcher::new(0.95);
        let candidates = vec!["name".to_string()];

        // "naem" has a high score but might not hit 0.95
        // With jaro_winkler, "name" vs "naem" scores around 0.93, so this should be None
        let strict_result = strict_matcher.best_match("naem", &candidates);
        assert!(
            strict_result.is_none(),
            "strict matcher (0.95 threshold) should reject ~0.93 similarity"
        );

        let loose_matcher = JaroWinklerMatcher::new(0.8);
        let result = loose_matcher.best_match("naem", &candidates);
        assert!(result.is_some());
    }
}
