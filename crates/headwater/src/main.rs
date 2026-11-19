//! `headwater`: what a repository depends on, and every newer release of
//! each, with upstream's notes and its level.

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use headwater::{render, scan};
use headwater_core::Options;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    ", revision ",
    env!("HEADWATER_REVISION"),
    ", built by ",
    env!("HEADWATER_BUILD")
);

#[derive(Parser)]
#[command(name = "headwater", version = VERSION, about = "Every release newer than what a repository pins, with upstream's notes and how big a step it is")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read the repository's dependencies, ask where each is published, and
    /// report every newer release with its level and notes
    Scan {
        /// The checkout to read; the current directory by default
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Print the answer as one JSON document
        #[arg(long)]
        json: bool,
        /// The leading portion of each release's notes to give, in bytes,
        /// cut at a line's end; 0 gives them whole. Whole in JSON, 600
        /// bytes for a person, by default
        #[arg(long)]
        notes_bytes: Option<usize>,
    },
    /// The Boomdrive plugin's adapter: one capture per release newer than
    /// its pin, in the capture contract, then the completion
    Capture {
        /// The raw place a run hands the adapter, empty, for the notes and
        /// the lookups' answers
        #[arg(long)]
        raw_dir: PathBuf,
        /// The cursor of the last run, which headwater does not need: the
        /// pin alone decides what is new
        #[arg(long)]
        cursor: Option<String>,
        /// The checkout to read; the configuration's `repository`, or the
        /// current directory
        #[arg(long)]
        repo: Option<PathBuf>,
    },
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .ok()
        .and_then(|t| t.format(&Rfc3339).ok())
        .unwrap_or_default()
}

fn options(notes_bytes: Option<usize>) -> Options {
    Options {
        notes_bytes: notes_bytes.filter(|n| *n > 0),
        asked_at: now(),
        headwater: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// The repository a capture run reads: the flag, else the configuration
/// file CAPTURE_CONFIG names, `{"repository": "<path>"}`, else here.
fn capture_repo(flag: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(repo) = flag {
        return Ok(repo);
    }
    let Some(file) = std::env::var_os("CAPTURE_CONFIG") else {
        return Ok(PathBuf::from("."));
    };
    let text = std::fs::read_to_string(&file)
        .map_err(|e| format!("CAPTURE_CONFIG {}: {e}", PathBuf::from(&file).display()))?;
    let config: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        format!(
            "CAPTURE_CONFIG {}: not JSON: {e}",
            PathBuf::from(&file).display()
        )
    })?;
    Ok(config
        .get("repository")
        .and_then(|r| r.as_str())
        .map_or_else(|| PathBuf::from("."), PathBuf::from))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = std::io::stdout().lock();
    match cli.command {
        Command::Scan {
            repo,
            json,
            notes_bytes,
        } => {
            let part = if json {
                notes_bytes
            } else {
                Some(notes_bytes.unwrap_or(600))
            };
            let report = scan::scan(&repo, scan::Lookups::default(), &options(part));
            let written = if json {
                serde_json::to_writer_pretty(&mut out, &report)
                    .map_err(std::io::Error::other)
                    .and_then(|()| writeln!(out))
            } else {
                write!(out, "{}", render::text(&report))
            };
            match written {
                Ok(()) => ExitCode::SUCCESS,
                Err(_) => ExitCode::FAILURE,
            }
        }
        Command::Capture {
            raw_dir,
            cursor: _,
            repo,
        } => {
            let repo = match capture_repo(repo) {
                Ok(repo) => repo,
                Err(why) => {
                    eprintln!("headwater capture: {why}");
                    return ExitCode::FAILURE;
                }
            };
            let report = scan::scan(&repo, scan::Lookups::default(), &options(None));
            let run = headwater_boomdrive::run(&report);
            match run.print(&raw_dir, &mut out) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("headwater capture: {error}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
