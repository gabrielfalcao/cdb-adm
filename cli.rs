use cdb_adm::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "ADM CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[arg(about = "CoreData Backup")]
    Cdb(Cdb),

    #[arg(about = "Agents and Daemons Manager")]
    Adm(Adm),
}

#[derive(Parser, Debug)]
pub struct Cdb {
    #[command(subcommand)]
    pub cdb: CdbCommand,

    #[command(subcommand)]
    pub adm: AdmCommand,
}

#[derive(Parser, Debug)]
pub struct Adm {
    #[arg()]
    pub name: String
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::List(op) => {
            println!("list of agents and daemons:")
        },
        Command::TurnOff(op) => {
            println!("tur")
        },
    }

    Ok(())
}
