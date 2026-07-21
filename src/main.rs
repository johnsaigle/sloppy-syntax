use std::io;

use clap::Parser;
use sloppy_syntax::analysis::analyze;
use sloppy_syntax::cli::{Args, load_text};
use sloppy_syntax::patterns::build_patterns;

fn main() -> io::Result<()> {
    let args = Args::parse();
    let patterns = build_patterns();

    if args.list_patterns {
        println!("Available patterns:");
        for p in &patterns {
            println!("  {:16} {}", p.id, p.name);
        }
        return Ok(());
    }

    let text = load_text(&args)?;
    let output = analyze(&text, &patterns, &args.disable);

    if args.json {
        if args.pretty {
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        } else {
            serde_json::to_writer(io::stdout(), &output)?;
        }
    } else {
        display(&output);
    }

    Ok(())
}

fn display(output: &sloppy_syntax::analysis::Output) {
    if output.matches.is_empty() {
        println!("No cliches detected.");
        return;
    }

    println!("Score: {}/5", output.score);
    println!(
        "Found {} match(es) across {} sentence(s) in {} characters.",
        output.stats.total_matches,
        output.stats.flagged_sentences,
        output.text_length,
    );

    if output.stats.chain_items > 0 {
        println!("Chain items: {}", output.stats.chain_items);
    }

    println!();
    println!("By pattern:");
    for (id, count) in &output.stats.per_pattern {
        let label = output
            .matches
            .iter()
            .find(|m| &m.pattern == id)
            .map_or(id.as_str(), |m| m.pattern_name.as_str());
        println!("  {count:2}  {label}");
    }

    println!();
    println!("Flagged sentences:");
    for m in &output.matches {
        let badge = m
            .badge
            .as_deref()
            .map(|b| format!(" [{b}]"))
            .unwrap_or_default();
        let chain = m
            .count
            .map(|c| format!(" ({c} items)"))
            .unwrap_or_default();
        println!();
        println!("  {} @ char {}-{}{}{}", m.pattern_name, m.start, m.end, badge, chain);
        for line in m.sentence.lines() {
            println!("  > {line}");
        }
    }
}
