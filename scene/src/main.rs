use std::str::FromStr;

use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Convert(ConvertArgs),
}

use embree::Device;

fn xx() {
    let x = Device::new();
}

#[derive(Args)]
struct ConvertArgs {
    #[arg(long, conflicts_with = "ron")]
    json: Option<String>,

    #[arg(long, conflicts_with = "json")]
    ron: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    xx();
}