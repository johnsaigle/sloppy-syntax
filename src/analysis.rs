use std::collections::{BTreeMap, HashSet};

use serde::Serialize;

use crate::patterns::{Pattern, RawMatch};

#[derive(Debug, Clone, Serialize)]
pub struct MatchResult {
    pub pattern: String,
    pub pattern_name: String,
    #[serde(skip)]
    pub weight: u8,
    pub start: usize,
    pub end: usize,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_title: Option<String>,
    pub sentence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Output {
    pub score: u8,
    pub matches: Vec<MatchResult>,
    pub stats: Stats,
    pub text_length: usize,
    pub patterns_enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub total_matches: usize,
    pub flagged_sentences: usize,
    pub chain_items: usize,
    pub per_pattern: BTreeMap<String, usize>,
}

pub fn analyze(text: &str, patterns: &[Pattern], disabled: &[String]) -> Output {
    let disabled: HashSet<&str> = disabled.iter().map(String::as_str).collect();
    let enabled: HashSet<&str> = patterns
        .iter()
        .map(|p| p.id)
        .filter(|id| !disabled.contains(id))
        .collect();

    let (matches, per_pattern) = collect_matches(text, &enabled, patterns);
    let regions = build_regions(text, &matches);
    let chain_items = matches.iter().filter_map(|m| m.count).sum();
    let score = slop_score(text, &matches);
    let mut patterns_enabled: Vec<_> = enabled.iter().copied().map(String::from).collect();
    patterns_enabled.sort();

    Output {
        score,
        patterns_enabled,
        stats: Stats {
            total_matches: matches.len(),
            flagged_sentences: regions.len(),
            chain_items,
            per_pattern,
        },
        text_length: text.len(),
        matches,
    }
}

pub(crate) fn sentence_bounds(text: &str, start: usize, end: usize) -> (usize, usize) {
    let mut s = text[..start]
        .char_indices()
        .rev()
        .find_map(|(i, ch)| matches!(ch, '\n' | '.' | '!' | '?' | '…').then_some(i + ch.len_utf8()))
        .unwrap_or(0);
    while s < start {
        let Some(ch) = text[s..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        s += ch.len_utf8();
    }

    let mut e = text.len();
    for (offset, ch) in text[end..].char_indices() {
        if ch == '\n' {
            e = end + offset;
            break;
        }
        if matches!(ch, '.' | '!' | '?' | '…') {
            e = end + offset + ch.len_utf8();
            while e < text.len() {
                let Some(closer) = text[e..].chars().next() else {
                    break;
                };
                if !matches!(closer, '"' | '\'' | '’' | '”' | ')' | ']') {
                    break;
                }
                e += closer.len_utf8();
            }
            break;
        }
    }
    (s, e)
}

pub(crate) fn collect_matches(
    text: &str,
    enabled: &HashSet<&str>,
    patterns: &[Pattern],
) -> (Vec<MatchResult>, BTreeMap<String, usize>) {
    let mut raw_matches: Vec<(usize, usize, RawMatch, &str, &str, u8)> = Vec::new();

    for p in patterns {
        if !enabled.contains(p.id) {
            continue;
        }
        for m in (p.finder)(text) {
            raw_matches.push((m.start, m.end, m, p.id, p.name, p.weight));
        }
    }

    raw_matches.sort_by_key(|(s, e, ..)| (*s, std::cmp::Reverse(*e)));

    let mut per_pattern: BTreeMap<String, usize> = BTreeMap::new();
    let mut de_duped: Vec<(usize, usize, RawMatch, &str, &str, u8)> = Vec::new();

    for (start, end, rm, pid, pname, weight) in raw_matches {
        if let Some(last) = de_duped.last()
            && start < last.1
        {
            continue;
        }
        *per_pattern.entry(pid.to_string()).or_default() += 1;
        de_duped.push((start, end, rm, pid, pname, weight));
    }

    let matches: Vec<MatchResult> = de_duped
        .into_iter()
        .map(|(_, _, rm, pid, pname, weight)| {
            let (s, e) = sentence_bounds(text, rm.start, rm.end);
            MatchResult {
                pattern: pid.to_string(),
                pattern_name: pname.to_string(),
                weight,
                start: rm.start,
                end: rm.end,
                text: text[rm.start..rm.end].to_string(),
                count: rm.count,
                badge: rm.badge,
                badge_title: rm.badge_title,
                sentence: text[s..e].trim().to_string(),
            }
        })
        .collect();

    (matches, per_pattern)
}

pub(crate) fn slop_score(text: &str, matches: &[MatchResult]) -> u8 {
    let words = text.split_whitespace().count().max(50);
    let weighted_hits: usize = matches.iter().map(|m| usize::from(m.weight)).sum();
    if weighted_hits == 0 {
        return 1;
    }

    match weighted_hits * 100 / words {
        0..=4 => 2,
        5..=9 => 3,
        10..=19 => 4,
        _ => 5,
    }
}

pub(crate) fn build_regions(
    text: &str,
    matches: &[MatchResult],
) -> Vec<(usize, usize, Vec<usize>)> {
    let mut regions: Vec<(usize, usize, Vec<usize>)> = Vec::new();
    for (i, m) in matches.iter().enumerate() {
        let (s, e) = sentence_bounds(text, m.start, m.end);
        if let Some(last) = regions.last_mut()
            && s <= last.1
        {
            last.1 = last.1.max(e);
            last.2.push(i);
            continue;
        }
        regions.push((s, e, vec![i]));
    }
    regions
}
