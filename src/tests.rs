use std::collections::HashSet;

use crate::analysis::{collect_matches, sentence_bounds, slop_score};
use crate::patterns::{Pattern, build_patterns};

const EXAMPLE: &str = "We rebuilt the editor from the ground up. No sign-ups, no downloads, no hassle \u{2014} just paste your text and start writing. Everything runs locally in your browser.

The reviewer read the draft twice. Did not flinch, did not blink, did not reach for the red pen. That's the whole review, honestly.

Don't call it a rewrite \u{2014} call it a rescue. The improvement is real, and it's not subtle. That loss is worth naming. Sit with that for a moment. The gains were modest, but that's not nothing.

You already know the answer, of course. Consistency is the entire game, and the punchline is that nobody wants to hear it. The entire pitch is one sentence long.

This closing paragraph is deliberately ordinary, with no list patterns at all, so nothing here should light up.";

fn find_pattern<'a>(patterns: &'a [Pattern], id: &str) -> &'a Pattern {
    patterns.iter().find(|pattern| pattern.id == id).unwrap()
}

#[test]
fn test_example_trips_every_original_pattern_once() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();
    let (matches, _) = collect_matches(EXAMPLE, &all, &patterns);
    let original = [
        "no-chain",
        "whole",
        "did-not-chain",
        "dont-verb-it",
        "sit-with",
        "already-know",
        "is-the-entire",
        "the-entire-is",
        "is-real",
        "punchline",
        "worth-naming",
        "not-nothing",
    ];

    for id in original {
        assert_eq!(
            matches.iter().filter(|m| m.pattern == id).count(),
            1,
            "expected one {id} match"
        );
    }
}

#[test]
fn test_no_chain() {
    let patterns = build_patterns();
    let no_chain = find_pattern(&patterns, "no-chain");

    let found = (no_chain.finder)("No sign-ups, no downloads, no hassle \u{2014} just paste.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));

    let found = (no_chain.finder)("The plan has no hidden fees and no long-term contracts.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    let found = (no_chain.finder)("No fluff, no filler, no jargon, no corporate buzzwords.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(4));

    assert!((no_chain.finder)("There is no catch here, honestly.").is_empty());
    assert!((no_chain.finder)("No, no, I insist.").is_empty());

    let found = (no_chain.finder)("It ships with no bells and whistles, no fluff.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    assert!((no_chain.finder)("no no no").is_empty());

    let found = (no_chain.finder)("no fluff; no filler");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    let found = (no_chain.finder)("no time, no money, no way to say no thanks");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));

    let found = (no_chain.finder)("no-code, no-fuss setup");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    assert!((no_chain.finder)("I know nothing, notice nothing.").is_empty());

    let found = (no_chain.finder)("No fluff, no filler.\nNo ads here.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));
}

#[test]
fn test_original_pattern_cases() {
    let patterns = build_patterns();

    let whole = find_pattern(&patterns, "whole");
    assert_eq!((whole.finder)("That's the whole point.").len(), 1);
    assert_eq!((whole.finder)("This is the whole game, really.").len(), 1);
    assert_eq!((whole.finder)("That was the whole pitch.").len(), 1);
    assert!((whole.finder)("The whole team showed up.").is_empty());

    let chain = find_pattern(&patterns, "did-not-chain");
    let found = (chain.finder)("Did not flinch, did not blink, did not apologize.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));
    let found = (chain.finder)("He didn't call and didn't write.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));
    assert!((chain.finder)("She did not go.").is_empty());
    let found = (chain.finder)("Did not know why, did not care.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    let p = find_pattern(&patterns, "dont-verb-it");
    assert_eq!(
        (p.finder)("Don't call it a comeback. Call it a return.").len(),
        1
    );
    assert_eq!(
        (p.finder)("Do not think of it as a burden. Think of it as fuel.").len(),
        1
    );
    assert!((p.finder)("Don't fear it. Name it.").is_empty());
    assert!((p.finder)("Don't overthink it.").is_empty());

    let p = find_pattern(&patterns, "sit-with");
    assert_eq!((p.finder)("Sit with that for a moment.").len(), 1);
    assert_eq!((p.finder)("Just sit with it.").len(), 1);
    assert_eq!((p.finder)("She was sitting with the discomfort.").len(), 1);
    assert!((p.finder)("Come sit with us at lunch.").is_empty());

    let p = find_pattern(&patterns, "already-know");
    assert_eq!((p.finder)("You already know the answer.").len(), 1);
    assert_eq!((p.finder)("Deep down, you already know.").len(), 1);
    assert!((p.finder)("If you already know Python, skip ahead.").is_empty());
    assert_eq!((p.finder)("You already know what to do.").len(), 1);
    assert_eq!((p.finder)("Part of you already knows it.").len(), 1);
}

#[test]
fn test_entire_real_punchline_worth_nothing_cases() {
    let patterns = build_patterns();

    let p = find_pattern(&patterns, "is-the-entire");
    assert_eq!((p.finder)("Consistency is the entire game.").len(), 1);
    assert_eq!((p.finder)("That's the entire business model.").len(), 1);
    assert!((p.finder)("He toured the entire factory.").is_empty());

    let p = find_pattern(&patterns, "the-entire-is");
    assert_eq!(
        (p.finder)("The entire point is that nobody reads.").len(),
        1
    );
    assert_eq!(
        (p.finder)("The entire business model is built on churn.").len(),
        1
    );
    assert!((p.finder)("He ate the entire pizza.").is_empty());
    assert_eq!((p.finder)("The entire team was exhausted.").len(), 1);
    assert!(
        (p.finder)("The entire history of the modern industrial world economy is complex.")
            .is_empty()
    );

    let p = find_pattern(&patterns, "is-real");
    assert_eq!(
        (p.finder)("The improvement is real, and it's not subtle.").len(),
        1
    );
    assert_eq!(
        (p.finder)("This is the real work, and it never ends.").len(),
        1
    );
    assert_eq!((p.finder)("The demand is real and growing.").len(), 1);
    assert!((p.finder)("He is a real estate agent and it shows.").is_empty());
    assert!((p.finder)("Is it real? And does it matter?").is_empty());
    assert!((p.finder)("The painting is real, but stolen.").is_empty());

    let p = find_pattern(&patterns, "punchline");
    assert_eq!((p.finder)("The punchline is that nobody laughed.").len(), 1);
    assert_eq!((p.finder)("The punchline: nothing changed.").len(), 1);
    assert_eq!((p.finder)("And the punchline? You knew.").len(), 1);
    assert!((p.finder)("He forgot the punchline entirely.").is_empty());

    let p = find_pattern(&patterns, "worth-naming");
    assert_eq!(
        (p.finder)("That loss is real and it's worth naming.").len(),
        1
    );
    assert_eq!((p.finder)("It's worth naming that this hurts.").len(), 1);
    assert_eq!((p.finder)("The grief here is worth naming.").len(), 1);
    assert_eq!(
        (p.finder)("That anger feels worth naming out loud.").len(),
        1
    );
    assert_eq!((p.finder)("Worth naming: nobody asked for this.").len(), 1);
    assert!((p.finder)("It's not worth naming names here.").is_empty());
    assert!((p.finder)("They spent the meeting naming the new mascot.").is_empty());
    assert!((p.finder)("The naming convention is worth documenting.").is_empty());

    let p = find_pattern(&patterns, "not-nothing");
    assert_eq!((p.finder)("That's not nothing.").len(), 1);
    assert_eq!(
        (p.finder)("Ten sign-ups in a week \u{2014} that is not nothing.").len(),
        1
    );
    assert_eq!(
        (p.finder)("It's not nothing, even if it's not everything.").len(),
        1
    );
    assert_eq!(
        (p.finder)("The launch drew a small crowd, which was not nothing.").len(),
        1
    );
    assert!((p.finder)("She insisted that nothing was wrong.").is_empty());
    assert!((p.finder)("There is nothing left to say.").is_empty());
}

#[test]
fn test_sentence_bounds_simple() {
    let t = "First sentence here. No fluff, no filler. Last one.";
    let patterns = build_patterns();
    let no_chain = find_pattern(&patterns, "no-chain");
    let m = &(no_chain.finder)(t)[0];
    let (s, e) = sentence_bounds(t, m.start, m.end);
    assert_eq!(&t[s..e], "No fluff, no filler.");
}

#[test]
fn test_disabled_pattern_is_not_used() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();
    let (matches_full, _) = collect_matches(EXAMPLE, &all, &patterns);

    let mut partial = all.clone();
    partial.remove("no-chain");
    let (matches_partial, _) = collect_matches(EXAMPLE, &partial, &patterns);

    assert!(matches_partial.len() < matches_full.len());
}

#[test]
fn test_sentence_no_chain() {
    let patterns = build_patterns();
    let pattern = find_pattern(&patterns, "no-sentence-chain");

    let found = (pattern.finder)("No model switching. No routing. No hidden proxy.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));
    assert!((pattern.finder)("No routing is required. The model stays loaded.").is_empty());
}

#[test]
fn test_rhetorical_templates() {
    let patterns = build_patterns();

    let contrast = find_pattern(&patterns, "contrastive-negation");
    assert_eq!(
        (contrast.finder)("It's not a wrapper — it's a new runtime.").len(),
        1
    );
    assert!((contrast.finder)("It is not available in this runtime.").is_empty());

    let meta = find_pattern(&patterns, "meta-commentary");
    assert_eq!(
        (meta.finder)("So here's the interesting part: it works.").len(),
        1
    );
    assert!((meta.finder)("The interesting part arrived yesterday.").is_empty());

    let math = find_pattern(&patterns, "math-is-simple");
    assert_eq!((math.finder)("The math is simple: divide by two.").len(), 1);
    assert!((math.finder)("The math is described below.").is_empty());

    let specific = find_pattern(&patterns, "nothing-specific");
    assert_eq!(
        (specific.finder)("Nothing here is specific to those models.").len(),
        1
    );
    assert!((specific.finder)("Nothing here depends on that model.").is_empty());
}

#[test]
fn test_low_weight_style_markers() {
    let patterns = build_patterns();

    let dashes = find_pattern(&patterns, "em-dash-asides");
    assert_eq!((dashes.finder)("One — aside — is enough.").len(), 2);
    assert!((dashes.finder)("One — rare dash is fine.").is_empty());
    assert!(
        (dashes.finder)("Use —quiet —verbose for more output.").is_empty(),
        "CLI flags with dashes should not trigger"
    );

    let obviously = find_pattern(&patterns, "sentence-final-obviously");
    assert_eq!((obviously.finder)("Testnet first, obviously.").len(), 1);
    assert_eq!(
        (obviously.finder)("Testnet first, obviously.\nNext step.").len(),
        1
    );
    assert!((obviously.finder)("Obviously, testnet comes first.").is_empty());

    let bold = find_pattern(&patterns, "bold-emphasis");
    assert_eq!((bold.finder)("This is **very important** text.").len(), 1);
    assert!((bold.finder)("This is plain text.").is_empty());
}

#[test]
fn test_unicode_typography_in_ascii_english() {
    let patterns = build_patterns();
    let pattern = find_pattern(&patterns, "unicode-typography");

    let found = (pattern.finder)("The model can’t see — but it can reason…");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));

    assert!((pattern.finder)("The model can't see -- but it can reason...").is_empty());
    assert!((pattern.finder)("Café déjà vu — résumé…").is_empty());
    assert!((pattern.finder)("One rare em dash — is not enough.").is_empty());
    assert!(
        (pattern.finder)("Run with —quiet —no-progress to avoid clutter.").is_empty(),
        "CLI flag dashes should not count as typography marks"
    );
}

#[test]
fn test_stock_phrases_from_samples() {
    let patterns = build_patterns();
    let cases = [
        (
            "real-burden",
            "Validator operation is a real operational burden.",
        ),
        (
            "maintenance-love",
            "The code has not received much maintenance love.",
        ),
        (
            "invisible-infrastructure",
            "The vision model is invisible infrastructure.",
        ),
        ("bottom-line-heading", "## The Bottom Line"),
        ("stock-ai-role", "Use it as an AI assistant."),
    ];

    for (id, text) in cases {
        assert_eq!((find_pattern(&patterns, id).finder)(text).len(), 1, "{id}");
    }
}

#[test]
fn test_score_is_one_to_five() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();

    let clean = "The service reads a file and returns a parsed record.";
    let (matches, _) = collect_matches(clean, &all, &patterns);
    assert_eq!(slop_score(clean, &matches), 1);

    let light = "Sit with that for a moment.";
    let (matches, _) = collect_matches(light, &all, &patterns);
    assert_eq!(slop_score(light, &matches), 2);

    let heavy = "No switching. No routing. It's not a wrapper — it's a revolution. So here's the interesting part. The math is simple. Nothing here is specific to those models. Testnet first, obviously.";
    let (matches, _) = collect_matches(heavy, &all, &patterns);
    assert!(slop_score(heavy, &matches) >= 4);
}
