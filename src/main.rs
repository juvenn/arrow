mod action;
mod context;
mod decode;
mod pipeline;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use context::Context;
use pipeline::Pipelines;
use std::env;
use std::io;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run pipeline from file or directory on current HEAD
    Run {
        /// Path to pipeline file or directory
        path: String,
    },
}

fn main() -> anyhow::Result<()> {
    if let Ok(_) = env::var("GIT_DIR") {
        return run_hook();
    }
    let cli = Cli::parse();

    Ok(())
}

fn run_hook() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    let args: Vec<String> = input.split_whitespace().map(String::from).collect();
    if args.len() < 3 {
        return Err(anyhow!("Not enough arguments"));
    }
    let ctx = Context::resolve_on_hook(args[2].clone(), args[0].clone(), args[1].clone())?;
    let mut pipelines = Pipelines::new();
    pipelines.run(ctx)?;
    Ok(())
}
