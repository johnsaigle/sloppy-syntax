use std::collections::HashSet;

use crate::analysis::{MatchResult, analyze, collect_matches, sentence_bounds, slop_score};
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

    let found = (pattern.finder)("The condition is ready ∧ valid ∨ authorized ‖ approved.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));

    assert!((pattern.finder)("The model can't see -- but it can reason...").is_empty());
    assert!((pattern.finder)("Café déjà vu — résumé…").is_empty());
    assert!((pattern.finder)("One rare em dash — is not enough.").is_empty());
    assert!((pattern.finder)("The condition is valid ∧ ready.").is_empty());
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
fn test_wikipedia_language_patterns() {
    let patterns = build_patterns();
    let cases = [
        ("ai-vocab", "We delve into the intricate interplay.", 3),
        ("not-just", "This is not just a tool, but a philosophy.", 1),
        (
            "note-that",
            "It is important to note that timing matters.",
            1,
        ),
        (
            "testament",
            "The building stands as a testament to optimism.",
            1,
        ),
        (
            "crucial-role",
            "Volunteers play a crucial role in the program.",
            1,
        ),
        (
            "landscape",
            "They adapted to an ever-evolving landscape.",
            1,
        ),
        ("vague-experts", "Some critics have noted a decline.", 1),
        (
            "despite-challenges",
            "Despite these challenges, growth continued.",
            1,
        ),
        (
            "participle-tail",
            "Sales doubled, underscoring the strength of the brand.",
            1,
        ),
        ("promo", "A hidden gem with breathtaking views.", 2),
        ("ai-leftovers", "contentReference[oaicite:0]{index=0}", 2),
    ];

    for (id, text, expected) in cases {
        assert_eq!(
            (find_pattern(&patterns, id).finder)(text).len(),
            expected,
            "{id}"
        );
    }
}

#[test]
fn test_wikipedia_language_pattern_negative_cases() {
    let patterns = build_patterns();
    let cases = [
        ("ai-vocab", "The report was thorough and well organized."),
        ("not-just", "He did not buy it."),
        ("note-that", "Please note the door code."),
        ("testament", "He read from the Old Testament."),
        ("crucial-role", "He plays the role of the villain."),
        ("landscape", "The landscape outside was gray."),
        ("vague-experts", "Dr. Chen argued the opposite."),
        ("despite-challenges", "The climb was a challenge."),
        ("participle-tail", "She kept highlighting passages."),
        ("promo", "The soup was rich and hearty."),
        ("ai-leftovers", "The last update shipped Tuesday."),
    ];

    for (id, text) in cases {
        assert!(
            (find_pattern(&patterns, id).finder)(text).is_empty(),
            "{id} unexpectedly matched"
        );
    }
}

#[test]
fn test_narrower_claim() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "narrower-claim");

    assert_eq!(
        (p.finder)("What I am claiming is narrower than that, though.").len(),
        1
    );
    assert_eq!(
        (p.finder)("My point is narrower than the one being attributed.").len(),
        1
    );
    assert!(((p.finder)("The claim is broader than expected.").is_empty()));
    assert!(((p.finder)("Nothing narrower here.").is_empty()));
}

#[test]
fn test_worth_unpacking() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "worth-unpacking");

    assert_eq!((p.finder)("That distinction is worth unpacking.").len(), 1);
    assert_eq!((p.finder)("The detail is worth unpacking here.").len(), 1);
    assert_eq!(
        ((p.finder)("Not worth unpacking further after all.").len()),
        1
    );
    assert!(((p.finder)("Worth the unpacking, nonetheless.").is_empty()));
}

#[test]
fn test_byte_identical() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "byte-identical");

    assert_eq!(
        ((p.finder)("The output is byte-identical to the input.").len()),
        1
    );
    assert_eq!(((p.finder)("byte identical results both times").len()), 1);
    assert!(((p.finder)("byte-by-byte identical, but separate.").is_empty()));
    assert!(((p.finder)("identical to the byte, precisely.").is_empty()));
}

#[test]
fn test_semicolon_in_sentence() {
    let patterns = build_patterns();
    let semi = find_pattern(&patterns, "semicolon-in-sentence");
    let combo = find_pattern(&patterns, "semicolon-and-emdash");

    assert_eq!((semi.finder)("This is a test; however, it fails.").len(), 1);
    assert!((combo.finder)("This is a test; however, it fails.").is_empty());

    assert!((semi.finder)("This is plain text.").is_empty());
    assert!((combo.finder)("This is plain text.").is_empty());

    let text = "First sentence. Second one; with semicolon. Third one.";
    assert_eq!((semi.finder)(text).len(), 1);

    assert!((combo.finder)("Just a dash \u{2014} no semicolon here.").is_empty());

    assert_eq!(
        (combo.finder)("This uses both \u{2014} a dash; and a semicolon.").len(),
        1
    );
    assert!((semi.finder)("This uses both \u{2014} a dash; and a semicolon.").is_empty());

    assert!((semi.finder)("No punctuation here.").is_empty());
    assert!((combo.finder)("No punctuation here.").is_empty());
}

#[test]
fn test_overloaded_asides() {
    let patterns = build_patterns();
    let pattern = find_pattern(&patterns, "overloaded-asides");
    let sample = "-- 'WormholeMessage'. The message carries the emitter's identity components\n\
-- (the watcher derives the 32-byte address as keccak256 over them); the second\n\
-- publish addresses the sequence-bumped successor directly via the cid\n\
-- 'PublishMessage' returns — no ACS re-resolution needed.";

    let found = (pattern.finder)(sample);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));
    assert_eq!(pattern.weight, 8);

    assert!(
        (pattern.finder)(
            "The watcher derives the address (from the emitter identity); the publisher then sends the resulting message."
        )
        .is_empty(),
        "two interruptions are not enough"
    );
    assert!(
        (pattern.finder)(
            "-- This long source comment wraps onto another line with enough ordinary words\n-- to clear the length threshold but contains no actual aside punctuation."
        )
        .is_empty(),
        "line-comment prefixes are not prose dashes"
    );
}

#[test]
fn test_overloaded_asides_produce_high_score() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();
    let text = "The message carries the emitter identity (which the watcher hashes into an address); the publisher then routes the sequence-bumped successor directly — without resolving the account again.";
    let (matches, _) = collect_matches(text, &all, &patterns);

    assert!(matches.iter().any(|m| m.pattern == "overloaded-asides"));
    assert!(slop_score(text, &matches) >= 4);
}

#[test]
fn test_alliteration() {
    let patterns = build_patterns();
    let single = find_pattern(&patterns, "alliteration");
    let multiple = find_pattern(&patterns, "multiple-alliteration");

    let found = (single.finder)("Clear, concise, compelling communication matters.");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(4));
    assert_eq!(found[0].badge.as_deref(), Some("c x4"));

    assert!((single.finder)("Funny failure is possible.").is_empty());
    assert!(
        (single.finder)("Clear APIs help developers produce concise documentation.").is_empty()
    );
    assert!((single.finder)("Very very very useful.").is_empty());
    assert!((multiple.finder)("Funny failure is possible.").is_empty());
}

#[test]
fn test_multiple_alliterations_in_one_sentence() {
    let patterns = build_patterns();
    let single = find_pattern(&patterns, "alliteration");
    let multiple = find_pattern(&patterns, "multiple-alliteration");
    let text =
        "I didn't want to sandpaper the sentences into sameness, as that would be a funny failure.";

    assert!((single.finder)(text).is_empty());
    let found = (multiple.finder)(text);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(5));
    assert_eq!(found[0].badge.as_deref(), Some("s x3, f x2"));
    assert_eq!(multiple.weight, 4);

    assert!(
        (multiple.finder)("Simple systems save time. Funny failures happen elsewhere.").is_empty(),
        "alliterations in separate sentences must not be combined"
    );
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

#[test]
fn test_byte_identical_variants() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "byte-identical");

    assert_eq!(
        ((p.finder)("The output is bit-exact to the reference.").len()),
        1
    );
    assert_eq!(((p.finder)("The outputs match byte-for-byte.").len()), 1);
    assert_eq!(
        ((p.finder)("A bit-for-bit reproduction of the build.").len()),
        1
    );
    assert_eq!(((p.finder)("bitwise identical, every run").len()), 1);
    assert!(((p.finder)("byte-by-byte identical, but separate.").is_empty()));
    assert!(((p.finder)("grabitexact and bitwise-identically shaped").is_empty()));
}

#[test]
fn test_is_the_whole() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "is-the-whole");

    assert_eq!((p.finder)("Consistency is the whole game.").len(), 1);
    assert_eq!((p.finder)("Here's the whole secret.").len(), 1);
    assert!((p.finder)("The whole team was there.").is_empty());
}

#[test]
fn test_performative_honesty() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "performative-honesty");

    assert_eq!(
        (p.finder)("I won't pretend the rollout was smooth.").len(),
        1
    );
    assert_eq!((p.finder)("Let's be honest about the numbers.").len(), 1);
    assert_eq!((p.finder)("Honestly, it was fine.").len(), 1);
    assert!((p.finder)("He was honest about the delay.").is_empty());
}

#[test]
fn test_thats_the_part() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "thats-the-part");

    assert_eq!(
        (p.finder)("That's the part a counter can't reach.").len(),
        1
    );
    assert_eq!(
        (p.finder)("The part that makes me trust the rest.").len(),
        1
    );
    assert!((p.finder)("The part arrived yesterday.").is_empty());
}

#[test]
fn test_the_only_i_trust() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "the-only-i-trust");

    assert_eq!((p.finder)("It's the only estimate I trust.").len(), 1);
    assert_eq!(
        (p.finder)("The only thing that matters is shipping.").len(),
        1
    );
    assert!((p.finder)("The only option is to restart.").is_empty());
}

#[test]
fn test_take_my_word() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "take-my-word");

    assert_eq!(
        (p.finder)("You don't have to take my word for it.").len(),
        1
    );
    assert_eq!((p.finder)("Don't take my word for any of this.").len(), 1);
    assert!((p.finder)("Take my word for it.").is_empty());
}

#[test]
fn test_turns_out() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "turns-out");

    assert_eq!(
        (p.finder)("It turns out that nobody reads the changelog.").len(),
        1
    );
    assert_eq!((p.finder)("Turns out the importer was fine.").len(), 1);
    assert!((p.finder)("The road turns out toward the coast.").is_empty());
}

#[test]
fn test_fits_in_your_head() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "fits-in-your-head");

    assert_eq!(
        (p.finder)("The small core still fits in your head.").len(),
        1
    );
    assert_eq!((p.finder)("It just works.").len(), 1);
    assert_eq!((p.finder)("Zero config, sane defaults.").len(), 2);
    assert!((p.finder)("Hold the door in your head.").is_empty());
}

#[test]
fn test_colon_triple() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "colon-triple");

    assert_eq!(
        (p.finder)(
            "The launch needed three things: a blog post, a demo video, and a pricing page."
        )
        .len(),
        1
    );
    assert!((p.finder)("The settings: ports and processes.").is_empty());
}

#[test]
fn test_x_is_dead() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "x-is-dead");

    assert_eq!((p.finder)("Peer code review is dead.").len(), 1);
    assert_eq!((p.finder)("AI is dead.").len(), 1);
    assert_eq!((p.finder)("Long live botd.").len(), 1);
    assert!((p.finder)("The dead letter queue is full.").is_empty());
}

#[test]
fn test_thats_why_mattered() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "thats-why-mattered");

    assert_eq!(
        (p.finder)("That's why the export button mattered.").len(),
        1
    );
    assert!((p.finder)("That is why the button exists.").is_empty());
}

#[test]
fn test_stranded_auxiliary() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "stranded-auxiliary");

    assert_eq!((p.finder)("The tool died; the data didn't.").len(), 1);
    assert_eq!((p.finder)("Maybe it wouldn't have.").len(), 1);
    assert!((p.finder)("The tool died; the data survived.").is_empty());
}

#[test]
fn test_not_but() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "not-but");

    assert_eq!(
        (p.finder)("The empty state isn't filler. It's orientation.").len(),
        1
    );
    assert_eq!((p.finder)("not a wrapper, but a runtime").len(), 1);
    assert!((p.finder)("not just a wrapper, but a runtime").is_empty());
}

#[test]
fn test_stacked_questions() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "stacked-questions");

    let found = (p.finder)("Do I know how it works? Where it breaks?");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    assert!((p.finder)("How does it work? Nobody knows.").is_empty());
}

#[test]
fn test_sentence_anaphora() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "sentence-anaphora");

    let found = (p.finder)(
        "Maybe nobody needed the importer. Maybe the shortcut confused people. Maybe the redesign was overdue.",
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(3));

    assert!((p.finder)("The importer was fine. The shortcut confused people.").is_empty());
    assert!(
        (p.finder)("It worked. It failed. It worked again.").is_empty(),
        "pronoun openers are skipped"
    );
}

#[test]
fn test_echo_triad() {
    let patterns = build_patterns();
    let p = find_pattern(&patterns, "echo-triad");

    let found = (p.finder)(
        "A shopping cart is an object in the system. A chat room is an object in the system.",
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, Some(2));

    assert!(
        (p.finder)("A shopping cart holds items. A chat room holds messages.").is_empty(),
        "sentences without a shared 4-gram do not echo"
    );
}

#[test]
fn test_urls_are_not_flagged_as_prose() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();

    let cases = [
        // utm parameters used to trip ai-leftovers.
        "Track conversions with https://example.com/landing?utm_source=newsletter&utm_medium=email.",
        // Comma-separated path segments used to trip no-chain.
        "Block https://cdn.example.com/no-transform,no-cache in the CDN config.",
        // URL path words used to trip ai-vocab and alliteration.
        "Read https://blog.example.com/delving-into-rust for the deep dive.",
        "Docs: https://docs.example.com/ever-evolving-notes today.",
        // Repeated URL sentences used to trip sentence-anaphora.
        "https://localhost:8080 is up. https://localhost:9090 is up. https://localhost:7070 is up.",
        // A host token used to trip x-is-dead ("com is dead").
        "https://ai.example.com is dead.",
    ];

    for text in cases {
        let (matches, _) = collect_matches(text, &all, &patterns);
        assert!(matches.is_empty(), "URL text unexpectedly flagged: {matches:?}");
    }
}

#[test]
fn test_slop_next_to_urls_still_flagged() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();

    let text = "Sit with that for a moment. Docs: https://docs.example.com/ever-evolving-notes today.";
    let (matches, _) = collect_matches(text, &all, &patterns);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern, "sit-with");
}

#[test]
fn test_ai_leftovers_still_flagged_inside_urls() {
    let patterns = build_patterns();
    let all: HashSet<&str> = patterns.iter().map(|p| p.id).collect();

    let text = "Source: https://trace.example.com/turn0search5 for the citation.";
    let (matches, _) = collect_matches(text, &all, &patterns);
    assert!(matches.iter().any(|m| m.pattern == "ai-leftovers"));
}

#[test]
fn test_score_density_bands_per_100_words() {
    let text = "word ".repeat(100);
    let hits = |n: usize| {
        (0..n)
            .map(|_| MatchResult {
                pattern: "test".into(),
                pattern_name: "test".into(),
                weight: 1,
                start: 0,
                end: 0,
                text: String::new(),
                count: None,
                badge: None,
                badge_title: None,
                sentence: String::new(),
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(slop_score(&text, &hits(1)), 2);
    assert_eq!(slop_score(&text, &hits(4)), 2);
    assert_eq!(slop_score(&text, &hits(5)), 3);
    assert_eq!(slop_score(&text, &hits(9)), 3);
    assert_eq!(slop_score(&text, &hits(10)), 4);
    assert_eq!(slop_score(&text, &hits(19)), 4);
    assert_eq!(slop_score(&text, &hits(20)), 5);
}

#[test]
fn test_chain_items_count_chain_patterns_only() {
    let patterns = build_patterns();
    let text = "Clear, concise, compelling communication matters. No fluff, no filler.";
    let output = analyze(text, &patterns, &[]);

    let no_chain = output
        .matches
        .iter()
        .find(|m| m.pattern == "no-chain")
        .expect("no-chain should match");
    assert_eq!(no_chain.count, Some(2));
    assert_eq!(output.stats.chain_items, 2);
}

#[test]
fn test_text_length_counts_characters() {
    let patterns = build_patterns();
    let output = analyze("café", &patterns, &[]);
    assert_eq!(output.text_length, 4);
}
