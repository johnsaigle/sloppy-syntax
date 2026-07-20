use std::io::{self, Read};
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "sloppy-syntax",
    about = "Detect LLM cliches in text and output JSON"
)]
pub struct Args {
    #[arg(help = "Text to analyze directly", conflicts_with = "file")]
    pub text: Option<String>,

    #[arg(short, long, help = "Read text from a file")]
    pub file: Option<PathBuf>,

    #[arg(short, long, help = "Pretty-print JSON output")]
    pub pretty: bool,

    #[arg(long, value_delimiter = ',', help = "Disable specific patterns by id")]
    pub disable: Vec<String>,

    #[arg(long, help = "List available patterns")]
    pub list_patterns: bool,
}

/// Load input text from the positional argument, `--file`, or stdin.
///
/// # Errors
///
/// Returns any filesystem or stdin read error encountered while loading text.
pub fn load_text(args: &Args) -> io::Result<String> {
    if let Some(text) = &args.text {
        return Ok(text.clone());
    }
    if let Some(path) = &args.file {
        return std::fs::read_to_string(path);
    }
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positional_text() {
        let args = Args::try_parse_from(["sloppy-syntax", "Sit with that."]).unwrap();

        assert_eq!(args.text.as_deref(), Some("Sit with that."));
        assert!(args.file.is_none());
    }

    #[test]
    fn positional_text_conflicts_with_file() {
        let result =
            Args::try_parse_from(["sloppy-syntax", "Sit with that.", "--file", "draft.txt"]);

        assert!(result.is_err());
    }
}
