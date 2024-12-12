
use clap::Parser;

mod watch;

#[derive(Debug, Parser)]
enum Commands {
    Watch {
        api_key: String,
        #[arg(short, long)]
        progress: bool,
        #[arg(short, long, default_value_t = 60)]
        interval: u64,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Commands::parse() {
        Commands::Watch { api_key, progress, interval } => watch::watch(&api_key, progress, interval).await,
    }
}
