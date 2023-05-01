mod context;
mod pipeline;
use clap::Parser;
use context::Context;
use pipeline::Pipelines;
use std::io;

// Run pipelines defined in yml file, default to .arrow/*.yml
#[derive(Parser, Debug)]
struct Cli {
    refname: String,
    pre_rev: String,
    new_rev: String,
}

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    let args: Vec<String> = input.splitn(3, ' ').map(String::from).collect();
    if args.len() < 3 {
        eprintln!("Invalid input: {}", input);
        return;
    }
    let ctx = Context::resolve_on_hook(args[2].clone(), args[0].clone(), args[1].clone());
    let mut pipelines = Pipelines::new(ctx);
    pipelines.run();
}
