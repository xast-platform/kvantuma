use clap::{Args, Parser, Subcommand};
use serde::{Serialize, Serializer, ser::SerializeMap};
use xastge::scene::{Scene, SerializableEntity};

type AnyError = anyhow::Error;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Convert(ConvertArgs),
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
struct ConvertArgs {
    #[arg(long, conflicts_with = "ron")]
    json: Option<String>,

    #[arg(long, conflicts_with = "json")]
    ron: Option<String>,
}

impl ConvertArgs {
    pub fn convert(self) {
        if let Some(json) = self.json {
            let result: Result<String, AnyError> = (|| {
                let scene: Scene = serde_json::from_str(&json)?;
                let res = ron::to_string(&scene)?;
                
                Ok(res)
            })();

            CliResult::from_result(result).expose();
        } else if let Some(ron) = self.ron {
            let result: Result<String, AnyError> = (|| {
                let scene: Scene = ron::from_str(&ron)?;
                let res = serde_json::to_string(&scene)?;
                
                Ok(res)
            })();

            CliResult::from_result(result).expose();
        }
    }
}

enum CliResult {
    Success(String),
    Err(AnyError),
}


impl Serialize for CliResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        match self {
            CliResult::Success(content) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("result", "success")?;
                map.serialize_entry("content", content)?;
                map.end()
            },
            CliResult::Err(err) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("result", "error")?;
                map.serialize_entry("content", &format!("{err:#}"))?;
                map.end()
            },
        }
    }
}

impl CliResult {
    pub fn from_result(result: Result<String, AnyError>) -> CliResult {
        match result {
            Ok(success) => CliResult::Success(success),
            Err(e) => CliResult::Err(e),
        }
    }

    pub fn expose(&self) {
        println!("{}", serde_json::to_string(self).expect("Cli result is invalid"));
    }
}

fn main() {
    match Cli::parse().command {
        Command::Convert(args) => args.convert(),
    };
}