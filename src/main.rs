use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use exitsplain::render;
use exitsplain::runner;
use exitsplain::{Catalog, Engine};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Explain failed commands with deterministic, offline rules."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a command, preserve its output, then explain a failure.
    Run {
        /// Maximum command-output bytes retained for analysis.
        #[arg(long, default_value_t = 2_097_152, value_parser = parse_log_limit)]
        max_log_bytes: usize,
        /// Disable ANSI styling.
        #[arg(long)]
        no_color: bool,
        /// Command and arguments. Put these after `--`.
        #[arg(required = true, trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Analyze a saved log, or stdin when no file is given.
    Analyze {
        /// Log file. Use `-` or omit the path for stdin.
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
        #[arg(long)]
        no_color: bool,
    },
    /// List built-in rules.
    Rules {
        /// Only show rules with this tag.
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Human,
    Json,
}

fn parse_log_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| "log limit must be a whole number".to_owned())?;
    if limit < 1024 {
        return Err("log limit must be at least 1024 bytes".to_owned());
    }
    Ok(limit)
}

fn main() {
    let exit_code = match execute(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("exitsplain: {error}");
            2
        }
    };
    process::exit(exit_code.clamp(0, 255));
}

fn execute(cli: Cli) -> Result<i32, Box<dyn std::error::Error>> {
    let catalog = Catalog::builtin()?;
    let engine = Engine::new(&catalog);

    match cli.command {
        Commands::Run {
            max_log_bytes,
            no_color,
            command,
        } => {
            let outcome = runner::run(&command, max_log_bytes)?;
            if outcome.exit_code != 0 {
                let diagnostics = engine.analyze(&outcome.captured);
                let color = !no_color && io::stderr().is_terminal();
                render::human(&diagnostics, io::stderr().lock(), color, outcome.truncated)?;
            }
            Ok(outcome.exit_code)
        }
        Commands::Analyze {
            path,
            format,
            no_color,
        } => {
            let log = read_log(path)?;
            let diagnostics = engine.analyze(&log);
            match format {
                Format::Human => {
                    let color = !no_color && io::stdout().is_terminal();
                    render::human(&diagnostics, io::stdout().lock(), color, false)?;
                }
                Format::Json => render::json(&diagnostics, io::stdout().lock())?,
            }
            Ok(0)
        }
        Commands::Rules { tag, format } => {
            let filtered: Vec<_> = catalog
                .rules()
                .filter(|rule| {
                    tag.as_ref()
                        .is_none_or(|tag| rule.tags.iter().any(|candidate| candidate == tag))
                })
                .collect();

            match format {
                Format::Human => render::rule_list(filtered.into_iter(), io::stdout().lock())?,
                Format::Json => {
                    serde_json::to_writer_pretty(io::stdout().lock(), &filtered)?;
                    println!();
                }
            }
            Ok(0)
        }
    }
}

fn read_log(path: Option<PathBuf>) -> io::Result<String> {
    match path {
        Some(path) if path.as_os_str() != "-" => fs::read_to_string(path),
        _ => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            Ok(input)
        }
    }
}
