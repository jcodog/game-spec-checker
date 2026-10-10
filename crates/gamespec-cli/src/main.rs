use clap::{Parser, Subcommand};
use gamespec_core::*;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "Offline Windows game requirements assessment")]
struct Args {
    /// Validated offline benchmark catalog. Omit to use the bundled baseline.
    #[arg(long, global = true)]
    catalog: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Scan {
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Check {
        profile: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        gpu: Option<String>,
    },
    Report {
        profile: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        gpu: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
}
fn run() -> Result<(), String> {
    let args = Args::parse();
    let catalog = catalog::load(args.catalog.as_deref())?;
    let report_output = match &args.command {
        Command::Report { output, .. } => Some(output.clone()),
        _ => None,
    };
    let snapshot = scan();
    let (json, output) = match args.command {
        Command::Scan { output } => (
            serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?,
            output,
        ),
        Command::Check { profile, path, gpu }
        | Command::Report {
            profile,
            path,
            gpu,
            output: _,
        } => {
            let profile =
                profiles::parse(&fs::read_to_string(profile).map_err(|e| e.to_string())?)?;
            let volume = resolve_location(&path, &snapshot.volumes)?;
            let selected =
                gpu.or_else(|| (snapshot.gpus.len() == 1).then(|| snapshot.gpus[0].id.clone()));
            if selected
                .as_ref()
                .is_some_and(|id| !snapshot.gpus.iter().any(|g| &g.id == id))
            {
                return Err("GPU ID not present in scan. Run scan to list adapters.".into());
            }
            let report = assess_with_catalog(
                &profile,
                &snapshot,
                selected.as_deref(),
                &path,
                Some(volume),
                &catalog,
            );
            (
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
                None,
            )
        }
    };
    let output = report_output.or(output);
    if let Some(output) = output {
        fs::write(output, json).map_err(|e| e.to_string())?;
    } else {
        println!("{json}");
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("GameSpec: {error}");
        std::process::exit(1);
    }
}
