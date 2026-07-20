# sloppy-syntax

Detect LLM cliches and AI-slop rhetorical patterns in text.

`sloppy-syntax` is a fast, rule-based CLI that scans text for recurring stylistic tics characteristic of LLM-generated prose and outputs structured JSON with matches, sentence context, and a weighted score.

## Installation

```bash
cargo install --git https://github.com/johnsaigle/llm-cliche-finder
```

Or build from source:

```bash
git clone https://github.com/johnsaigle/llm-cliche-finder
cd llm-cliche-finder
cargo build --release
```

## Usage

```bash
# Analyze text from stdin
echo "it's not bad — it's good. and that's the whole point." | sloppy-syntax

# Analyze a file
sloppy-syntax --file essay.txt

# Analyze inline text
sloppy-syntax "The math is simple: no work, no progress."

# Pretty-print the JSON
sloppy-syntax --pretty --file essay.txt

# Disable specific patterns
sloppy-syntax --file essay.txt --disable no-chain,sit-with

# List all available patterns
sloppy-syntax --list-patterns
```

## Output

The tool outputs JSON with this structure:

| Field | Description |
|---|---|
| `score` | 1–5 severity rating of cliche density |
| `matches` | Array of detected pattern hits with byte offsets, matched text, sentence context, and optional count badges |
| `stats` | Per-pattern counts and aggregate totals |
| `text_length` | Character count of the input text |
| `patterns_enabled` | Pattern IDs that were active |

## Score interpretation

The score is a density metric: weighted pattern hits per 50 words.

| Score | Density range | Meaning |
|---|---|---|
| 1 | 0 hits | Clean |
| 2 | 1–4 | Light |
| 3 | 5–9 | Moderate |
| 4 | 10–19 | Heavy |
| 5 | 20+ | Saturated |

## Detected patterns

| ID | Pattern | Weight |
|---|---|---|
| `no-chain` | "No X, no Y" chains | 3 |
| `no-sentence-chain` | "No X. No Y." sentence chains | 3 |
| `did-not-chain` | "Did not X, did not Y" chains | 3 |
| `dont-verb-it` | "Don't VERB it — VERB it" | 3 |
| `contrastive-negation` | "It's not X — it's Y" | 3 |
| `nothing-specific` | "Nothing here is specific to …" | 3 |
| `whole` | "That's the whole …" | 2 |
| `sit-with` | "Sit with that" | 2 |
| `already-know` | "You already know" | 2 |
| `is-the-entire` | "Is the entire …" | 2 |
| `the-entire-is` | "The entire … is" | 2 |
| `is-real` | "Is real … and / not" | 2 |
| `punchline` | "The punchline is" | 2 |
| `worth-naming` | "Worth naming" | 2 |
| `not-nothing` | "That's not nothing" | 2 |
| `meta-commentary` | "Here's the interesting part" | 2 |
| `math-is-simple` | "The math is simple" | 2 |
| `maintenance-love` | "Maintenance love" | 2 |
| `invisible-infrastructure` | "Invisible infrastructure" | 2 |
| `bottom-line-heading` | "The Bottom Line" heading | 2 |
| `em-dash-asides` | Repeated em-dash asides | 1 |
| `unicode-typography` | Polished Unicode punctuation | 1 |
| `sentence-final-obviously` | Sentence-final "obviously" | 1 |
| `real-burden` | "Real operational burden" | 1 |
| `bold-emphasis` | Markdown bold emphasis | 1 |
| `stock-ai-role` | "AI assistant" / "pair programmer" | 1 |

## Library usage

```rust
use sloppy_syntax::{analyze, build_patterns};

let patterns = build_patterns();
let output = analyze("some text to check", &patterns, &[]);
println!("{}", output.score);
```

## Acknowledgments

This project was inspired by [Simon Willison's LLM cliche highlighter](https://tools.simonwillison.net/llm-cliche-highlighter) and [Sloppy Syntax](https://johnsaigle.com/posts/sloppy-syntax).
