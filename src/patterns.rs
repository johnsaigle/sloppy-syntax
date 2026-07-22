use fancy_regex::Regex as FancyRegex;
use regex::Regex;

pub struct Pattern {
    pub id: &'static str,
    pub name: &'static str,
    pub weight: u8,
    pub finder: Box<Finder>,
}

type Finder = dyn Fn(&str) -> Vec<RawMatch>;

#[derive(Debug, Clone)]
pub struct RawMatch {
    pub start: usize,
    pub end: usize,
    pub count: Option<usize>,
    pub badge: Option<String>,
    pub badge_title: Option<String>,
}

const CHAIN_BODY: &str = r"[^,.;:!?\n\u{2013}\u{2014}\u{2026}]*";
const CHAIN_SEP: &str = r"(?:\s*,\s*(?:and\s+|or\s+)?|\s+(?:and|or)\s+|\s*[;&\u{2013}\u{2014}]\s*(?:and\s+|or\s+)?|\s+-{1,2}\s+)";

fn make_chain_finder<'a>(
    head: &'a str,
    head_test: Regex,
    item_label: &str,
) -> impl Fn(&str) -> Vec<RawMatch> + use<'a> {
    let item = format!("{head}{CHAIN_BODY}");
    let chain = format!(r"\b{item}(?:{CHAIN_SEP}{item})+");
    let chain_re = Regex::new(&format!("(?i){chain}")).expect("invalid chain pattern regex");
    let split_re = Regex::new(CHAIN_SEP).expect("invalid chain split regex");
    let label = item_label.to_string();

    move |text: &str| {
        chain_re
            .find_iter(text)
            .map(|m| {
                let mut end = m.end();
                while end > m.start() && text.as_bytes()[end - 1].is_ascii_whitespace() {
                    end -= 1;
                }
                let count = split_re
                    .split(&text[m.start()..end])
                    .filter(|p| head_test.is_match(p.trim()))
                    .count();
                RawMatch {
                    start: m.start(),
                    end,
                    count: Some(count),
                    badge: Some(count.to_string()),
                    badge_title: Some(format!(
                        "{} {label}{}",
                        count,
                        if count == 1 { "" } else { "s" },
                    )),
                }
            })
            .collect()
    }
}

fn make_regex_finder(re: &str) -> impl Fn(&str) -> Vec<RawMatch> {
    let regex = Regex::new(re).expect("invalid client pattern regex");
    move |text: &str| {
        regex
            .find_iter(text)
            .map(|m| RawMatch {
                start: m.start(),
                end: m.end(),
                count: None,
                badge: None,
                badge_title: None,
            })
            .collect()
    }
}

fn make_fancy_regex_finder(re: &str) -> impl Fn(&str) -> Vec<RawMatch> {
    let regex = FancyRegex::new(re).expect("invalid client pattern regex");
    move |text: &str| {
        let mut results = Vec::new();
        let mut pos = 0;
        while let Ok(Some(m)) = regex.find_from_pos(text, pos) {
            results.push(RawMatch {
                start: m.start(),
                end: m.end(),
                count: None,
                badge: None,
                badge_title: None,
            });
            pos = m.end();
        }
        results
    }
}

fn make_threshold_regex_finder(re: &str, threshold: usize) -> impl Fn(&str) -> Vec<RawMatch> {
    let regex = Regex::new(re).expect("invalid threshold pattern regex");
    move |text: &str| {
        let matches: Vec<_> = regex.find_iter(text).collect();
        if matches.len() < threshold {
            return Vec::new();
        }
        matches
            .into_iter()
            .map(|m| RawMatch {
                start: m.start(),
                end: m.end(),
                count: None,
                badge: None,
                badge_title: None,
            })
            .collect()
    }
}

fn make_sentence_no_chain_finder() -> impl Fn(&str) -> Vec<RawMatch> {
    let chain = Regex::new(
        r"(?im)\bno\s+[^.!?\n]{1,100}[.!?](?:[ \t]+|\n+)no\s+[^.!?\n]{1,100}(?:[.!?]|$)(?:(?:[ \t]+|\n+)no\s+[^.!?\n]{1,100}(?:[.!?]|$))*",
    )
    .expect("invalid sentence chain regex");
    let split = Regex::new(r"[.!?]+(?:\s+|$)").expect("invalid sentence split regex");

    move |text: &str| {
        chain
            .find_iter(text)
            .map(|m| {
                let count = split
                    .split(m.as_str())
                    .filter(|part| part.trim().to_ascii_lowercase().starts_with("no "))
                    .count();
                RawMatch {
                    start: m.start(),
                    end: m.end(),
                    count: Some(count),
                    badge: Some(count.to_string()),
                    badge_title: Some(format!("{count} \"No ...\" sentences")),
                }
            })
            .collect()
    }
}

fn find_unicode_typography(text: &str) -> Vec<RawMatch> {
    let mut found = Vec::new();
    let mut sentence_start = 0;

    for sentence_end in text
        .char_indices()
        .filter_map(|(i, ch)| {
            matches!(ch, '.' | '!' | '?' | '…' | '\n').then_some(i + ch.len_utf8())
        })
        .chain(std::iter::once(text.len()))
    {
        if sentence_end <= sentence_start {
            continue;
        }
        let sentence = &text[sentence_start..sentence_end];
        let ascii_letters = sentence.chars().filter(char::is_ascii_alphabetic).count();
        let total_letters = sentence.chars().filter(|ch| ch.is_alphabetic()).count();
        let typography: Vec<_> = sentence
            .char_indices()
            .filter(|(i, ch)| {
                if !matches!(
                    ch,
                    '–' | '—' | '…' | '\u{201c}' | '\u{201d}' | '\u{2018}' | '\u{2019}' | '\u{00a0}'
                ) {
                    return false;
                }
                if matches!(ch, '–' | '—') {
                    let left_ws = *i == 0
                        || sentence.as_bytes().get(i - 1).is_some_and(u8::is_ascii_whitespace);
                    let right_ws = i + ch.len_utf8() >= sentence.len()
                        || sentence
                            .as_bytes()
                            .get(i + ch.len_utf8())
                            .is_some_and(u8::is_ascii_whitespace);
                    return left_ws && right_ws;
                }
                true
            })
            .collect();

        if typography.len() >= 2 && total_letters > 0 && ascii_letters * 10 >= total_letters * 9 {
            let (offset, ch) = typography[0];
            found.push(RawMatch {
                start: sentence_start + offset,
                end: sentence_start + offset + ch.len_utf8(),
                count: Some(typography.len()),
                badge: Some(typography.len().to_string()),
                badge_title: Some(format!(
                    "{} typographic Unicode marks in this sentence",
                    typography.len()
                )),
            });
        }

        sentence_start = sentence_end;
    }

    found
}

#[must_use]
pub fn build_patterns() -> Vec<Pattern> {
    let mut patterns = build_original_patterns();
    patterns.extend(build_additional_patterns());
    patterns
}

#[allow(clippy::too_many_lines)] // Declarative registry; splitting obscures pattern order.
fn build_original_patterns() -> Vec<Pattern> {
    let no_head_test = Regex::new(r"(?i)^no[\-\s]").expect("invalid no-chain head regex");
    let did_not_test =
        Regex::new(r"(?i)^(?:did\s+not|didn['\u{2019}]t)\s").expect("invalid did-not head regex");

    vec![
        Pattern {
            id: "no-chain",
            name: "\"No X, no Y\" chains",
            weight: 3,
            finder: Box::new(make_chain_finder(r"no[-\s]", no_head_test, "\"no\" item")),
        },
        Pattern {
            id: "no-sentence-chain",
            name: "\"No X. No Y.\" sentence chains",
            weight: 3,
            finder: Box::new(make_sentence_no_chain_finder()),
        },
        Pattern {
            id: "whole",
            name: "\"That's the whole …\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\b(?:that|this)(?:['\u{2019}]s|\s+(?:is|was))\s+the\s+whole\b(?:\s+\w+)?",
            )),
        },
        Pattern {
            id: "did-not-chain",
            name: "\"Did not X, did not Y\" chains",
            weight: 3,
            finder: Box::new(make_chain_finder(
                r"(?:did\s+not|didn['\u{2019}]t)\s",
                did_not_test,
                "\"did not\" item",
            )),
        },
        Pattern {
            id: "dont-verb-it",
            name: "\"Don't VERB it … VERB it\"",
            weight: 3,
            finder: Box::new(make_fancy_regex_finder(
                r#"(?i)\b(?:do\s+not|don['\u{2019}]t)\s+(?:just\s+|simply\s+|merely\s+)?(\w+)(?:\s+(?:of|about|at|on|for|with|to))?\s+it\b[^.!?\n]*?[.!?;,:\u{2013}\u{2014}]['"\u{201d}\u{2019}]*\s*(?:just\s+|simply\s+|merely\s+)?\1(?:\s+(?:of|about|at|on|for|with|to))?\s+it\b"#,
            )),
        },
        Pattern {
            id: "sit-with",
            name: "\"Sit with that\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\bsit(?:s|ting)?\s+with\s+(?:that|this|it|(?:the|your)\s+(?:discomfort|feelings?|tension|weight|uncertainty|ambiguity|grief|silence|unease))\b(?:\s+for\s+a\s+\w+)?",
            )),
        },
        Pattern {
            id: "already-know",
            name: "\"You already know\"",
            weight: 2,
            finder: Box::new(make_fancy_regex_finder(
                r"(?i)\byou\s+already\s+knows?\s+(?:the\s+answer|what|how|why|this|that|it|who|where)\b|\byou\s+already\s+knows?\b(?![ \t]+\w)",
            )),
        },
        Pattern {
            id: "is-the-entire",
            name: "\"Is the entire …\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)(?:\b(?:is|was|are|were)|['\u{2019}]s)\s+the\s+entire\b(?:\s+\w+)?",
            )),
        },
        Pattern {
            id: "the-entire-is",
            name: "\"The entire … is\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\bthe\s+entire\s+[\w'\u{2019}-]+(?:\s+[\w'\u{2019}-]+){0,4}?\s+(?:is|was|are|were)\b",
            )),
        },
        Pattern {
            id: "is-real",
            name: "\"Is real … and / not\"",
            weight: 2,
            finder: Box::new(make_fancy_regex_finder(
                r"(?i)\bis\s+(?:(?:the|a)\s+real\b(?![\s-]+(?:estate|time|life|world|quick)\b)[^.!?\n]*?\b(?:and|not)\s+it\b|real\b(?![\s-]+(?:estate|time|life|world|quick)\b)[^.!?\n]*?\b(?:and|not)\b)",
            )),
        },
        Pattern {
            id: "punchline",
            name: "\"The punchline is\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\bthe\s+punchline(?:\s+(?:is|was|being)\b|\s*[:?])",
            )),
        },
        Pattern {
            id: "worth-naming",
            name: "\"Worth naming\"",
            weight: 2,
            finder: Box::new(make_fancy_regex_finder(
                r"(?i)(?:\b(?:is|are|was|were|feels?|felt|seems?|seemed)|['\u{2019}]s)\s+(?:\w+\s+){0,2}?worth\s+naming\b(?!\s+names\b)|\bworth\s+naming\s*:",
            )),
        },
        Pattern {
            id: "not-nothing",
            name: "\"That's not nothing\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\b(?:that|this|it|which)(?:['\u{2019}]s|\s+(?:is|was))\s+not\s+nothing\b",
            )),
        },
    ]
}

fn build_additional_patterns() -> Vec<Pattern> {
    vec![
        Pattern {
            id: "contrastive-negation",
            name: "\"It's not X — it's Y\"",
            weight: 3,
            finder: Box::new(make_regex_finder(
                r"(?i)\b(?:it|this|that)(?:['\u{2019}]s|\s+is)\s+not\s+[^.!?\n\u{2013}\u{2014}]{1,80}\s*[-\u{2013}\u{2014},:;]\s*(?:it|this|that)(?:['\u{2019}]s|\s+is)\s+[^.!?\n]{1,80}",
            )),
        },
        Pattern {
            id: "meta-commentary",
            name: "\"Here's the interesting part\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\b(?:so\s+)?here['\u{2019}]s\s+(?:the\s+)?(?:question|interesting\s+part|thing|catch|kicker|rub)\b",
            )),
        },
        Pattern {
            id: "math-is-simple",
            name: "\"The math is simple\"",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?i)\bthe\s+math\s+(?:is|was|looks?|seems?)\s+(?:simple|easy|straightforward)\b",
            )),
        },
        Pattern {
            id: "nothing-specific",
            name: "\"Nothing here is specific to …\"",
            weight: 3,
            finder: Box::new(make_regex_finder(
                r"(?i)\bnothing\s+(?:here\s+)?is\s+specific\s+to\s+(?:this|that|these|those|it|them)\b",
            )),
        },
        Pattern {
            id: "em-dash-asides",
            name: "Repeated em-dash asides",
            weight: 1,
            finder: Box::new(make_threshold_regex_finder(r"(?:\s|^)\u{2014}(?:\s|$)", 2)),
        },
        Pattern {
            id: "unicode-typography",
            name: "Polished Unicode punctuation",
            weight: 1,
            finder: Box::new(find_unicode_typography),
        },
        Pattern {
            id: "sentence-final-obviously",
            name: "Sentence-final \"obviously\"",
            weight: 1,
            finder: Box::new(make_regex_finder(r"(?i)\bobviously\s*[.!?]")),
        },
        Pattern {
            id: "real-burden",
            name: "\"Real operational burden\"",
            weight: 1,
            finder: Box::new(make_regex_finder(
                r"(?i)\breal\s+(?:operational\s+)?(?:burden|cost|risk|work|value|problem|constraint|tradeoff|impact)\b",
            )),
        },
        Pattern {
            id: "maintenance-love",
            name: "\"Maintenance love\"",
            weight: 2,
            finder: Box::new(make_regex_finder(r"(?i)\bmaintenance\s+love\b")),
        },
        Pattern {
            id: "invisible-infrastructure",
            name: "\"Invisible infrastructure\"",
            weight: 2,
            finder: Box::new(make_regex_finder(r"(?i)\binvisible\s+infrastructure\b")),
        },
        Pattern {
            id: "bottom-line-heading",
            name: "\"The Bottom Line\" heading",
            weight: 2,
            finder: Box::new(make_regex_finder(
                r"(?im)^#{1,6}[ \t]+(?:the[ \t]+)?bottom[ \t]+line[ \t]*$",
            )),
        },
        Pattern {
            id: "bold-emphasis",
            name: "Markdown bold emphasis",
            weight: 1,
            finder: Box::new(make_regex_finder(r"\*\*[^*\n]{1,120}\*\*")),
        },
        Pattern {
            id: "stock-ai-role",
            name: "\"AI assistant\" / \"pair programmer\"",
            weight: 1,
            finder: Box::new(make_regex_finder(
                r"(?i)\b(?:AI\s+assistant|pair\s+programmer)\b",
            )),
        },
    ]
}
