mod commands;
use clap::Parser;

use crate::commands::Commands;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

fn main() {
    println!("Hello, world!");
}
