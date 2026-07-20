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

    if args.pretty {
        serde_json::to_writer_pretty(io::stdout(), &output)?;
    } else {
        serde_json::to_writer(io::stdout(), &output)?;
    }

    Ok(())
}
