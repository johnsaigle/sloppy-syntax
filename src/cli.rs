use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

pub const MAX_INPUT_SIZE: usize = 16 * 1024 * 1024;

#[derive(Parser)]
#[command(name = "sloppy-syntax", about = "Detect LLM cliches in text")]
pub struct Args {
    #[arg(help = "Text to analyze directly", conflicts_with = "file")]
    pub text: Option<String>,

    #[arg(short, long, help = "Read text from a file")]
    pub file: Option<PathBuf>,

    #[arg(long, help = "Output as JSON instead of human-readable text")]
    pub json: bool,

    #[arg(short, long, help = "Pretty-print JSON output (only with --json)")]
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
        ensure_size(text.len(), MAX_INPUT_SIZE)?;
        return Ok(text.clone());
    }
    if let Some(path) = &args.file {
        return read_text_file(path);
    }
    read_bounded_utf8(io::stdin().lock(), MAX_INPUT_SIZE)
}

fn read_text_file(path: &Path) -> io::Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input file must not be a symbolic link",
        ));
    }
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input path must be a regular file",
        ));
    }
    let file_size = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    ensure_size(file_size, MAX_INPUT_SIZE)?;
    read_bounded_utf8(File::open(path)?, MAX_INPUT_SIZE)
}

fn read_bounded_utf8(mut reader: impl Read, max_size: usize) -> io::Result<String> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(max_size.min(8 * 1024))
        .map_err(|_| io::Error::other("could not allocate memory for input"))?;
    let mut chunk = [0_u8; 8 * 1024];

    loop {
        let bytes_read = reader.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }
        let new_len = bytes
            .len()
            .checked_add(bytes_read)
            .ok_or_else(|| input_too_large(max_size))?;
        ensure_size(new_len, max_size)?;
        bytes
            .try_reserve(bytes_read)
            .map_err(|_| io::Error::other("could not allocate memory for input"))?;
        bytes.extend_from_slice(&chunk[..bytes_read]);
    }

    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn ensure_size(size: usize, max_size: usize) -> io::Result<()> {
    if size > max_size {
        return Err(input_too_large(max_size));
    }
    Ok(())
}

fn input_too_large(max_size: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("input exceeds the {max_size}-byte size limit"),
    )
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

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

    #[test]
    fn bounded_reader_rejects_oversized_input() {
        let error = read_bounded_utf8(Cursor::new(vec![b'x'; 9]), 8).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("size limit"));
    }

    #[test]
    fn bounded_reader_rejects_invalid_utf8() {
        let error = read_bounded_utf8(Cursor::new([0xff]), 8).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn positional_text_is_bounded() {
        let args = Args {
            text: Some("x".repeat(MAX_INPUT_SIZE + 1)),
            file: None,
            json: false,
            pretty: false,
            disable: Vec::new(),
            list_patterns: false,
        };

        let error = load_text(&args).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[cfg(unix)]
    #[test]
    fn file_input_rejects_symbolic_links() {
        use std::os::unix::fs::symlink;

        let directory =
            std::env::temp_dir().join(format!("sloppy-syntax-symlink-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir(&directory).unwrap();
        let target = directory.join("target.txt");
        let link = directory.join("link.txt");
        fs::write(&target, "plain text").unwrap();
        symlink(&target, &link).unwrap();

        let error = read_text_file(&link).unwrap_err();

        fs::remove_dir_all(directory).unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
