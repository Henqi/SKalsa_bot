use clap::Parser;
use skalsa_bot::{CourtName, Weekday};

#[derive(Parser)]
#[command(author, about, version)]
struct Args {
    /// Optional court name to check
    court: Option<CourtName>,

    /// Weekday to check when specifying court name [1-7]
    #[arg(value_enum, short, long)]
    day: Option<Weekday>,

    /// Hour to check when specifying court name [00-23]
    #[arg(short, long, value_name = "HOUR")]
    time: Option<u32>,

    /// Print verbose information
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.court {
        Some(CourtName::Delsu) => {
            println!(
                "{}",
                skalsa_bot::check_delsu(args.day, args.time, args.verbose).await?
            );
        }
        Some(CourtName::Hakis) => {
            println!(
                "{}",
                skalsa_bot::check_hakis(args.day, args.time, args.verbose).await?
            );
        }
        None => {
            println!(
                "{}: {}",
                CourtName::Delsu,
                skalsa_bot::check_delsu(args.day, args.time, args.verbose).await?
            );
            println!(
                "{}: {}",
                CourtName::Hakis,
                skalsa_bot::check_hakis(args.day, args.time, args.verbose).await?
            );
        }
    }

    Ok(())
}
